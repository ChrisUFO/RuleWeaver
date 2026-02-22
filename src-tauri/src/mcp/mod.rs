use axum::{
    extract::{State},
    routing::post,
    Json, Router,
};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tower_http::cors::CorsLayer;

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::broadcast;

use crate::database::{Database, ExecutionLogInput};
use crate::error::{AppError, Result};
use crate::execution::{
    argument_env_var_name, contains_disallowed_pattern,
    execute_shell_with_timeout_env, replace_template_with_env_ref, sanitize_argument_value,
    slugify,
};
use crate::models::{Command, Skill};

const MAX_SKILL_STEPS: usize = 10;
const MAX_STEP_LENGTH: usize = 4000;
const CMD_EXEC_TIMEOUT_SECS: u64 = 60;
const SKILL_EXEC_TIMEOUT_SECS: u64 = 60;
const MCP_RATE_LIMIT_WINDOW_SECS: u64 = 10;
const MCP_RATE_LIMIT_MAX_CALLS: usize = 30;
const MAX_OUTPUT_SIZE: usize = 10 * 1024 * 1024; // 10MB

fn truncate_output(s: String) -> String {
    if s.len() > MAX_OUTPUT_SIZE {
        let mut truncated = s;
        truncated.truncate(MAX_OUTPUT_SIZE);
        truncated.push_str("\n\n[Output truncated due to size limit]");
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
    commands: Vec<Command>,
    skills: Vec<Skill>,
    invocation_timestamps: Vec<Instant>,
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
                commands: Vec::new(),
                skills: Vec::new(),
                invocation_timestamps: Vec::new(),
                db: None,
            })),
        }
    }

    pub fn refresh_commands(&self, db: &Database) -> Result<()> {
        let commands = db.get_all_commands()?;
        let skills = db.get_all_skills()?;
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
        tokio::spawn(async move {
            let app = Router::new()
                .route("/", post(mcp_handler))
                // Support root and any other path for flexibility
                .fallback(post(mcp_handler))
                .layer(CorsLayer::permissive())
                .with_state(manager.clone());

            let addr = format!("127.0.0.1:{}", port);
            let listener = match tokio::net::TcpListener::bind(&addr).await {
                Ok(l) => l,
                Err(e) => {
                    let _ = manager.log(format!("Failed to bind MCP server {}: {}", addr, e));
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
        if state.logs.len() > 500 {
            let drain_to = state.logs.len() - 500;
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
        let cutoff = Instant::now() - Duration::from_secs(MCP_RATE_LIMIT_WINDOW_SECS);
        state.invocation_timestamps.retain(|t| *t >= cutoff);

        if state.invocation_timestamps.len() >= MCP_RATE_LIMIT_MAX_CALLS {
            return Ok(false);
        }

        state.invocation_timestamps.push(Instant::now());
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
        "initialize" => json!({
            "jsonrpc": "2.0",
            "id": request.id,
            "result": {
                "serverInfo": {
                    "name": "RuleWeaver MCP",
                    "version": "0.1.0"
                }
            }
        }),
        "tools/list" => {
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
                        "name": slugify(&c.name),
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
                        "name": format!("skill_{}", slugify(&s.name)),
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
                "id": request.id,
                "result": { "tools": all_tools }
            })
        }
        "tools/call" => {
            let allow = match manager.allow_invocation() {
                Ok(a) => a,
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

            if !allow {
                json!({
                    "jsonrpc": "2.0",
                    "id": request.id,
                    "error": {
                        "code": -32029,
                        "message": "Rate limit exceeded. Please retry shortly."
                    }
                })
            } else {
                let params = request.params.unwrap_or_else(|| json!({}));
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

                if let Some(cmd) = commands.iter().find(|c| slugify(&c.name) == name && c.expose_via_mcp) {
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
                        json!({
                            "jsonrpc": "2.0",
                            "id": request.id,
                            "error": {
                                "code": -32602,
                                "message": format!("Missing required arguments: {}", missing_required.join(", "))
                            }
                        })
                    } else {
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
                                    envs.push((argument_env_var_name(&arg.name), safe_value));
                                }
                                Err(e) => {
                                    invalid_arg_message = Some(e.to_string());
                                    break;
                                }
                            }
                        }

                        if let Some(message) = invalid_arg_message {
                            json!({
                                "jsonrpc": "2.0",
                                "id": request.id,
                                "error": {
                                    "code": -32602,
                                    "message": format!("Invalid argument value: {}", message)
                                }
                            })
                        } else {
                            let start = Instant::now();
                            match execute_shell_with_timeout_env(
                                &rendered,
                                Duration::from_secs(CMD_EXEC_TIMEOUT_SECS),
                                &envs,
                            )
                            .await
                            {
                                Ok((exit_code, stdout, stderr)) => {
                                    let duration_ms = start.elapsed().as_millis() as u64;
                                    let _ = manager.log(format!(
                                        "MCP tools/call '{}' exit code {} ({}ms)",
                                        cmd.name, exit_code, duration_ms
                                    ));
                                    if let Some(db) = &shared_db {
                                        let args_json =
                                            serde_json::to_string(&args_map).unwrap_or_default();
                                        let _ = db.add_execution_log(&ExecutionLogInput {
                                            command_id: &cmd.id,
                                            command_name: &cmd.name,
                                            arguments_json: &args_json,
                                            stdout: &stdout,
                                            stderr: &stderr,
                                            exit_code,
                                            duration_ms,
                                            triggered_by: "mcp",
                                        });
                                    }
                                    json!({
                                        "jsonrpc": "2.0",
                                        "id": request.id,
                                        "result": {
                                            "content": [{
                                                "type": "text",
                                                "text": format!("exit_code: {}\n\nstdout:\n{}\n\nstderr:\n{}", exit_code, truncate_output(stdout), truncate_output(stderr))
                                            }],
                                            "is_error": exit_code != 0
                                        }
                                    })
                                }
                                Err(e) => json!({
                                    "jsonrpc": "2.0",
                                    "id": request.id,
                                    "result": {
                                        "content": [{
                                            "type": "text",
                                            "text": format!("Failed to execute command: {}", e)
                                        }],
                                        "is_error": true
                                    }
                                }),
                            }
                        }
                    }
                } else if let Some(skill) = skills
                    .iter()
                    .find(|s| s.enabled && format!("skill_{}", slugify(&s.name)) == name)
                {
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
                            "id": request.id,
                            "result": {
                                "content": [{
                                    "type": "text",
                                    "text": display_text
                                }],
                                "is_error": false
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
                                Duration::from_secs(SKILL_EXEC_TIMEOUT_SECS),
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

                        if let Some(db) = &shared_db {
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
                            "id": request.id,
                            "result": {
                                "content": [{
                                    "type": "text",
                                    "text": truncate_output(output)
                                }],
                                "is_error": is_error
                            }
                        })
                    }
                } else {
                    json!({
                        "jsonrpc": "2.0",
                        "id": request.id,
                        "error": {
                            "code": -32602,
                            "message": format!("Unknown or disabled tool: {}", name)
                        }
                    })
                }
            }
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
