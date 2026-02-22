use axum::{
    extract::State,
    http::{HeaderValue, Method},
    routing::post,
    Json, Router,
};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tower_http::cors::CorsLayer;

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

use crate::constants::{
    CMD_EXEC_TIMEOUT, LOG_LIMIT, MAX_OUTPUT_SIZE, MAX_SKILL_STEPS, MAX_STEP_LENGTH,
    MCP_RATE_LIMIT_MAX_CALLS, MCP_RATE_LIMIT_WINDOW, MCP_SERVER_BACKOFF_INITIAL_MS,
    MCP_SERVER_RETRY_COUNT, SKILL_EXEC_TIMEOUT,
};
use crate::database::{Database, ExecutionLogInput};
use crate::error::{AppError, Result};
use crate::execution::{
    argument_env_var_name, contains_disallowed_pattern, execute_and_log,
    execute_shell_with_timeout_env, replace_template_with_env_ref, sanitize_argument_value, slugify,
};
use crate::models::{Command, Skill};

fn truncate_output(s: String) -> String {
    if s.len() > MAX_OUTPUT_SIZE {
        let original_len = s.len();
        let mut truncated = s;
        truncated.truncate(MAX_OUTPUT_SIZE);
        truncated.push_str(&format!(
            "\n\n[Output truncated from {} bytes due to 10MB size limit]",
            original_len
        ));
        truncated
    } else {
        s
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct McpStatus {
    pub running: bool,
    pub port: u16,
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct McpConnectionInstructions {
    pub claude_code_json: String,
    pub opencode_json: String,
    pub standalone_command: String,
}

#[derive(Debug)]
struct McpRuntime {
    running: bool,
    port: u16,
    started_at: Option<Instant>,
    logs: Vec<String>,
    stop_tx: Option<broadcast::Sender<()>>,
    task_handle: Option<JoinHandle<()>>,
    commands: Vec<Command>,
    skills: Vec<Skill>,
    invocation_timestamps: VecDeque<Instant>,
    db: Option<Arc<Database>>,
}

#[derive(Clone, Debug)]
pub struct McpManager {
    inner: Arc<Mutex<McpRuntime>>,
}

pub struct McpSnapshot {
    pub commands: Vec<Command>,
    pub skills: Vec<Skill>,
    pub db: Option<Arc<Database>>,
}

impl McpManager {
    pub fn new(port: u16) -> Self {
        Self {
            inner: Arc::new(Mutex::new(McpRuntime {
                running: false,
                port,
                started_at: None,
                logs: Vec::new(),
                stop_tx: None,
                task_handle: None,
                commands: Vec::new(),
                skills: Vec::new(),
                invocation_timestamps: VecDeque::new(),
                db: None,
            })),
        }
    }

    pub fn refresh_commands(&self, db: &Database) -> Result<()> {
        let (commands, skills) = db.get_mcp_data()?;
        let mut state = self.inner.lock().map_err(|_| AppError::LockError)?;
        state.commands = commands;
        state.skills = skills;
        Ok(())
    }

    fn snapshot(&self) -> Result<McpSnapshot> {
        let state = self.inner.lock().map_err(|_| AppError::LockError)?;
        Ok(McpSnapshot {
            commands: state.commands.clone(),
            skills: state.skills.clone(),
            db: state.db.clone(),
        })
    }

    pub fn start(&self, db: &Arc<Database>) -> Result<()> {
        self.refresh_commands(db)?;

        let port = {
            let mut state = self.inner.lock().map_err(|_| AppError::LockError)?;
            if state.running {
                return Ok(());
            }
            state.running = true;
            state.started_at = Some(Instant::now());
            state.logs.push("Starting MCP server".to_string());
            state.db = Some(Arc::clone(db));
            state.port
        };

        let (stop_tx, _) = broadcast::channel(1);
        {
            let mut state = self.inner.lock().map_err(|_| AppError::LockError)?;
            state.stop_tx = Some(stop_tx.clone());
        }

        let manager = self.clone();
        let mut stop_rx = stop_tx.subscribe();
        let handle = tokio::spawn(async move {
            let app = Router::new()
                .route("/", post(mcp_handler))
                // Support root and any other path for flexibility
                .fallback(post(mcp_handler))
                .layer(
                    CorsLayer::new()
                        .allow_origin([
                            "http://localhost".parse::<HeaderValue>().unwrap(),
                            "http://127.0.0.1".parse::<HeaderValue>().unwrap(),
                        ])
                        .allow_methods([Method::POST])
                        .allow_headers([axum::http::header::CONTENT_TYPE]),
                )
                .with_state(manager.clone());

            let addr = format!("127.0.0.1:{}", port);
            
            // Port binding with retry/backoff
            let mut retry_count = 0;
            let mut backoff_ms = MCP_SERVER_BACKOFF_INITIAL_MS;
            let listener = loop {
                match tokio::net::TcpListener::bind(&addr).await {
                    Ok(l) => break Some(l),
                    Err(e) => {
                        if retry_count >= MCP_SERVER_RETRY_COUNT {
                            let _ = manager.log(format!("Failed to bind MCP server after {} attempts {}: {}", MCP_SERVER_RETRY_COUNT, addr, e));
                            break None;
                        }
                        let _ = manager.log(format!("Port {} busy, retrying in {}ms...", port, backoff_ms));
                        tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                        retry_count += 1;
                        backoff_ms *= 2;
                    }
                }
            };

            let listener = match listener {
                Some(l) => l,
                None => {
                    let _ = manager.mark_stopped();
                    return;
                }
            };

            let _ = manager.log(format!("MCP server listening on {}", addr));

            if let Err(e) = axum::serve(listener, app)
                .with_graceful_shutdown(async move {
                    let _ = stop_rx.recv().await;
                })
                .await
            {
                let _ = manager.log(format!("MCP server error: {}", e));
            }

            let _ = manager.log("MCP server stopped".to_string());
            let _ = manager.mark_stopped();
        });

        {
            let mut state = self.inner.lock().map_err(|_| AppError::LockError)?;
            state.task_handle = Some(handle);
        }

        Ok(())
    }

    pub fn stop(&self) -> Result<()> {
        let tx = {
            let mut state = self.inner.lock().map_err(|_| AppError::LockError)?;
            if !state.running {
                return Ok(());
            }
            state.stop_tx.take()
        };

        if let Some(tx) = tx {
            let _ = tx.send(());
        }

        Ok(())
    }

    pub async fn wait_until_stopped(&self) -> Result<()> {
        let handle = {
            let mut state = self.inner.lock().map_err(|_| AppError::LockError)?;
            state.task_handle.take()
        };

        if let Some(handle) = handle {
            let _ = handle.await;
        }
        Ok(())
    }

    pub fn status(&self) -> Result<McpStatus> {
        let state = self.inner.lock().map_err(|_| AppError::LockError)?;
        let uptime_seconds = state.started_at.map(|t| t.elapsed().as_secs()).unwrap_or(0);
        Ok(McpStatus {
            running: state.running,
            port: state.port,
            uptime_seconds,
        })
    }

    pub fn logs(&self, limit: usize) -> Result<Vec<String>> {
        let state = self.inner.lock().map_err(|_| AppError::LockError)?;
        let len = state.logs.len();
        let start = len.saturating_sub(limit);
        Ok(state.logs[start..].to_vec())
    }

    pub fn instructions(&self) -> Result<McpConnectionInstructions> {
        let status = self.status()?;
        let port = status.port;

        let claude_code_json = serde_json::to_string_pretty(&json!({
            "mcpServers": {
                "ruleweaver": {
                    "url": format!("http://127.0.0.1:{}", port)
                }
            }
        }))
        .map_err(AppError::Serialization)?;

        let opencode_json = serde_json::to_string_pretty(&json!({
            "mcp": {
                "servers": [
                    {
                        "name": "ruleweaver",
                        "url": format!("http://127.0.0.1:{}", port)
                    }
                ]
            }
        }))
        .map_err(AppError::Serialization)?;

        Ok(McpConnectionInstructions {
            claude_code_json,
            opencode_json,
            standalone_command: format!("ruleweaver-mcp --port {}", port),
        })
    }

    fn log(&self, message: String) -> Result<()> {
        let mut state = self.inner.lock().map_err(|_| AppError::LockError)?;
        state.logs.push(message);
        if state.logs.len() > LOG_LIMIT {
            let drain_to = state.logs.len() - LOG_LIMIT;
            state.logs.drain(0..drain_to);
        }
        Ok(())
    }

    fn mark_stopped(&self) -> Result<()> {
        let mut state = self.inner.lock().map_err(|_| AppError::LockError)?;
        state.running = false;
        state.stop_tx = None;
        state.started_at = None;
        Ok(())
    }

    fn allow_invocation(&self) -> Result<bool> {
        let mut state = self.inner.lock().map_err(|_| AppError::LockError)?;
        let cutoff = Instant::now() - MCP_RATE_LIMIT_WINDOW;
        
        while let Some(t) = state.invocation_timestamps.front() {
            if *t < cutoff {
                state.invocation_timestamps.pop_front();
            } else {
                break;
            }
        }

        if state.invocation_timestamps.len() >= MCP_RATE_LIMIT_MAX_CALLS {
            return Ok(false);
        }

        state.invocation_timestamps.push_back(Instant::now());
        Ok(true)
    }
}

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    id: serde_json::Value,
    method: String,
    params: Option<serde_json::Value>,
}

async fn mcp_handler(
    State(manager): State<McpManager>,
    Json(request): Json<JsonRpcRequest>,
) -> Json<serde_json::Value> {
    let McpSnapshot {
        commands,
        skills,
        db: shared_db,
    } = match manager.snapshot() {
        Ok(s) => s,
        Err(e) => {
            return Json(json!({
                "jsonrpc": "2.0",
                "id": request.id,
                "error": {
                    "code": -32603,
                    "message": format!("Internal server error: {}", e)
                }
            }));
        }
    };

    let response = match request.method.as_str() {
        "initialize" => handle_initialize(request.id),
        "tools/list" => handle_tools_list(request.id, &commands, &skills),
        "tools/call" => {
            handle_tools_call(&manager, request.id, request.params, &commands, &skills, &shared_db).await
        }
        _ => json!({
            "jsonrpc": "2.0",
            "id": request.id,
            "error": {
                "code": -32601,
                "message": format!("Method not found: {}", request.method)
            }
        }),
    };

    Json(response)
}

fn handle_initialize(id: serde_json::Value) -> serde_json::Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "serverInfo": {
                "name": "RuleWeaver MCP",
                "version": "0.1.0"
            }
        }
    })
}

fn handle_tools_list(
    id: serde_json::Value,
    commands: &[Command],
    skills: &[Skill],
) -> serde_json::Value {
    let tools: Vec<serde_json::Value> = commands
        .iter()
        .filter(|c| c.expose_via_mcp)
        .map(|c| {
            let mut props = serde_json::Map::new();
            let mut required: Vec<String> = Vec::new();

            for arg in &c.arguments {
                props.insert(
                    arg.name.clone(),
                    json!({
                        "type": "string",
                        "description": arg.description,
                    }),
                );
                if arg.required {
                    required.push(arg.name.clone());
                }
            }

                                json!({
                                    "name": format!("{}-{}", slugify(&c.name), &c.id[..8]),
                                    "description": c.description,
                                    "inputSchema": {
                                        "type": "object",
                                        "properties": props,
                                        "required": required,
                                    }
                                })
            
        })
        .collect();

    let skill_tools: Vec<serde_json::Value> = skills
        .iter()
        .filter(|s| s.enabled)
        .map(|s| {
            json!({
                "name": format!("skill_{}-{}", slugify(&s.name), &s.id[..8]),
                "description": s.description,
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "input": {
                            "type": "string",
                            "description": "Optional free-form input for the skill"
                        }
                    },
                    "required": []
                }
            })
        })
        .collect();

    let mut all_tools = tools;
    all_tools.extend(skill_tools);

    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": { "tools": all_tools }
    })
}

async fn handle_tools_call(
    manager: &McpManager,
    id: serde_json::Value,
    params: Option<serde_json::Value>,
    commands: &[Command],
    skills: &[Skill],
    shared_db: &Option<Arc<Database>>,
) -> serde_json::Value {
    let allow = match manager.allow_invocation() {
        Ok(a) => a,
        Err(e) => {
            return json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32603,
                    "message": format!("Internal server error: {}", e)
                }
            });
        }
    };

    if !allow {
        return json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": -32029,
                "message": "Rate limit exceeded. Please retry shortly."
            }
        });
    }

    let params = params.unwrap_or_else(|| json!({}));
    let name = params
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();

    let args_map = params
        .get("arguments")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();

    if let Some(cmd) = commands
        .iter()
        .find(|c| format!("{}-{}", slugify(&c.name), &c.id[..8]) == name && c.expose_via_mcp)
    {
        handle_command_call(manager, id, cmd, args_map, shared_db).await
    } else if let Some(skill) = skills
        .iter()
        .find(|s| s.enabled && format!("skill_{}-{}", slugify(&s.name), &s.id[..8]) == name)
    {
        handle_skill_call(manager, id, skill, args_map, shared_db).await
    } else {
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": -32602,
                "message": format!("Unknown or disabled tool: {}", name)
            }
        })
    }
}

async fn handle_command_call(
    manager: &McpManager,
    id: serde_json::Value,
    cmd: &Command,
    args_map: serde_json::Map<String, serde_json::Value>,
    shared_db: &Option<Arc<Database>>,
) -> serde_json::Value {
    let missing_required: Vec<String> = cmd
        .arguments
        .iter()
        .filter(|arg| {
            arg.required
                && !args_map.contains_key(&arg.name)
                && arg
                    .default_value
                    .as_ref()
                    .map(|v| v.is_empty())
                    .unwrap_or(true)
        })
        .map(|arg| arg.name.clone())
        .collect();

    if !missing_required.is_empty() {
        return json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": -32602,
                "message": format!("Missing required arguments: {}", missing_required.join(", "))
            }
        });
    }

    let mut rendered = cmd.script.clone();
    let mut envs: Vec<(String, String)> = Vec::new();
    let mut invalid_arg_message: Option<String> = None;

    for arg in &cmd.arguments {
        rendered = replace_template_with_env_ref(&rendered, &arg.name);

        let raw_value = args_map
            .get(&arg.name)
            .map(|v| {
                if let Some(s) = v.as_str() {
                    s.to_string()
                } else {
                    v.to_string()
                }
            })
            .or_else(|| arg.default_value.clone())
            .unwrap_or_default();

                                    match sanitize_argument_value(&raw_value) {
                                        Ok(safe_value) => {
                                            // Enum validation
                                            if matches!(arg.arg_type, crate::models::ArgumentType::Enum) {
                                                if let Some(ref options) = arg.options {
                                                    if !options.contains(&raw_value) {
                                                        invalid_arg_message = Some(format!(
                                                            "Argument '{}' must be one of: {}",
                                                            arg.name,
                                                            options.join(", ")
                                                        ));
                                                        break;
                                                    }
                                                }
                                            }
                                            envs.push((argument_env_var_name(&arg.name), safe_value));
                                        }
                                        Err(e) => {
                                            invalid_arg_message = Some(e.to_string());
                                            break;
                                        }
                                    }
        
    }

    if let Some(message) = invalid_arg_message {
        return json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": -32602,
                "message": format!("Invalid argument value: {}", message)
            }
        });
    }

    match execute_and_log(
        shared_db.as_ref().map(|arc| arc.as_ref()),
        &cmd.id,
        &cmd.name,
        &rendered,
        CMD_EXEC_TIMEOUT,
        &envs,
        &serde_json::to_string(&args_map).unwrap_or_default(),
        "mcp",
    )
    .await
    {
        Ok((exit_code, stdout, stderr, duration_ms)) => {
            let _ = manager.log(format!(
                "MCP tools/call '{}' exit code {} ({}ms)",
                cmd.name, exit_code, duration_ms
            ));
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "content": [{
                        "type": "text",
                        "text": format!("exit_code: {}\n\nstdout:\n{}\n\nstderr:\n{}", exit_code, truncate_output(stdout), truncate_output(stderr))
                    }],
                    "isError": exit_code != 0
                }
            })
        }
        Err(e) => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "content": [{
                    "type": "text",
                    "text": format!("Failed to execute command: {}", e)
                }],
                "isError": true
            }
        }),
    }
}

async fn handle_skill_call(
    manager: &McpManager,
    id: serde_json::Value,
    skill: &Skill,
    args_map: serde_json::Map<String, serde_json::Value>,
    shared_db: &Option<Arc<Database>>,
) -> serde_json::Value {
    let input = args_map
        .get("input")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();

    // Security: DO NOT use string replacement for {{input}} as it leads to command injection.
    // Instead, we pass the input via environment variables.
    #[cfg(target_os = "windows")]
    let rendered = skill.instructions.replace("{{input}}", "%RW_SKILL_INPUT%");
    #[cfg(not(target_os = "windows"))]
    let rendered = skill.instructions.replace("{{input}}", "$RW_SKILL_INPUT");

    let steps = extract_skill_steps(&rendered);

    if steps.is_empty() {
        // If no shell steps, just return the instructions (with input placeholder)
        // Note: If we really want to return the text with the input replaced,
        // we should do it safely here ONLY for the returned text, not for execution.
        let display_text = skill.instructions.replace("{{input}}", &input);
                                json!({
                                    "jsonrpc": "2.0",
                                    "id": id,
                                    "result": {
                                        "content": [{
                                            "type": "text",
                                            "text": display_text
                                        }],
                                        "isError": false
                                    }
                                })
                            } else {
                                let mut output = String::new();
                                let mut is_error = false;
        
        let start = Instant::now();
        let skill_envs = vec![("RW_SKILL_INPUT".to_string(), input.clone())];

        for (idx, step) in steps.iter().take(MAX_SKILL_STEPS).enumerate() {
            if step.len() > MAX_STEP_LENGTH {
                is_error = true;
                output.push_str(&format!("Step {} rejected: too long\n", idx + 1));
                break;
            }

            if let Some(pattern) = contains_disallowed_pattern(step) {
                is_error = true;
                output.push_str(&format!(
                    "Step {} rejected due to unsafe pattern: {}\n",
                    idx + 1,
                    pattern
                ));
                break;
            }

                                        match execute_shell_with_timeout_env(
                                            step,
                                            SKILL_EXEC_TIMEOUT,
                                            &skill_envs,
                                        )
                                        .await
            
            {
                Ok((exit_code, stdout, stderr)) => {
                    output.push_str(&format!(
                        "[step {}] exit_code: {}\nstdout:\n{}\nstderr:\n{}\n\n",
                        idx + 1,
                        exit_code,
                        stdout,
                        stderr
                    ));
                    if exit_code != 0 {
                        is_error = true;
                        break;
                    }
                }
                Err(e) => {
                    is_error = true;
                    output.push_str(&format!(
                        "[step {}] execution error: {}\n",
                        idx + 1,
                        e
                    ));
                    break;
                }
            }
        }

        let duration_ms = start.elapsed().as_millis() as u64;
        let _ = manager.log(format!(
            "MCP tools/call '{}' skill execution {} ({}ms)",
            skill.name,
            if is_error { "failed" } else { "succeeded" },
            duration_ms
        ));

        if let Some(db) = shared_db {
            let args_json = serde_json::to_string(&args_map).unwrap_or_default();
            let skill_name = format!("skill:{}", skill.name);
            let _ = db.add_execution_log(&ExecutionLogInput {
                command_id: &skill.id,
                command_name: &skill_name,
                arguments_json: &args_json,
                stdout: &output,
                stderr: "",
                exit_code: if is_error { 1 } else { 0 },
                duration_ms,
                triggered_by: "mcp-skill",
            });
        }

                                json!({
                                    "jsonrpc": "2.0",
                                    "id": id,
                                    "result": {
                                        "content": [{
                                            "type": "text",
                                            "text": truncate_output(output)
                                        }],
                                        "isError": is_error
                                    }
                                })
        
    }
}

pub fn extract_skill_steps(instructions: &str) -> Vec<String> {
    static RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let re = RE.get_or_init(|| {
        Regex::new(r"(?s)```(?:bash|sh|shell|powershell|pwsh|cmd)?\n(.*?)```")
            .expect("Invalid skill steps regex")
    });

    re.captures_iter(instructions)
        .filter_map(|caps| caps.get(1).map(|m| m.as_str().trim().to_string()))
        .filter(|s| !s.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("My Skill"), "my-skill");
        assert_eq!(slugify("Skill__Name"), "skill-name");
    }

    #[test]
    fn test_extract_skill_steps() {
        let text = "before\n```bash\necho one\n```\nmid\n```sh\necho two\n```";
        let steps = extract_skill_steps(text);
        assert_eq!(steps.len(), 2);
        assert!(steps[0].contains("echo one"));
    }

    #[test]
    fn test_disallowed_patterns() {
        assert!(contains_disallowed_pattern("rm -rf /").is_some());
        assert!(contains_disallowed_pattern("echo hi").is_none());
    }
}
