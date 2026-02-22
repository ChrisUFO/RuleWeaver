use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;

use crate::database::{Database, ExecutionLogInput};
use crate::error::{AppError, Result};
use crate::execution::{
    argument_env_var_name, contains_disallowed_pattern, execute_shell_with_timeout,
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

    fn snapshot(&self) -> Result<(Vec<Command>, Vec<Skill>, Option<Arc<Database>>)> {
        let state = self.inner.lock().map_err(|_| AppError::LockError)?;
        Ok((
            state.commands.clone(),
            state.skills.clone(),
            state.db.clone(),
        ))
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
            let bind_addr = format!("127.0.0.1:{}", port);
            let listener = match TcpListener::bind(&bind_addr).await {
                Ok(listener) => listener,
                Err(e) => {
                    let _ = manager.log(format!("Failed to bind MCP server {}: {}", bind_addr, e));
                    let _ = manager.mark_stopped();
                    return;
                }
            };
            let _ = manager.log(format!("MCP server listening on {}", bind_addr));

            loop {
                tokio::select! {
                    _ = stop_rx.recv() => {
                        break;
                    }
                    accept_res = listener.accept() => {
                        match accept_res {
                            Ok((stream, addr)) => {
                                let _ = manager.log(format!("Client connected: {}", addr));
                                let manager_clone = manager.clone();
                                tokio::spawn(async move {
                                    if let Err(e) = handle_client(stream, &manager_clone).await {
                                        let _ = manager_clone.log(format!("MCP handler error: {}", e));
                                    }
                                });
                            }
                            Err(e) => {
                                let _ = manager.log(format!("Listener error: {}", e));
                                tokio::time::sleep(Duration::from_millis(200)).await;
                            }
                        }
                    }
                }
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

async fn handle_client(mut stream: TcpStream, manager: &McpManager) -> Result<()> {
    let body = read_http_body(&mut stream).await?;

    let request: JsonRpcRequest = match serde_json::from_str(body.trim()) {
        Ok(req) => req,
        Err(e) => {
            let _ = manager.log(format!("Invalid JSON-RPC request: {}", e));
            return Ok(());
        }
    };

    let (commands, skills, shared_db) = manager.snapshot()?;

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
                        "name": c.name,
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
            if !manager.allow_invocation()? {
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

                if let Some(cmd) = commands.iter().find(|c| c.name == name && c.expose_via_mcp) {
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
                            match execute_shell_with_timeout_env(
                                &rendered,
                                Duration::from_secs(CMD_EXEC_TIMEOUT_SECS),
                                &envs,
                            )
                            .await
                            {
                                Ok((exit_code, stdout, stderr)) => {
                                    let _ = manager.log(format!(
                                        "MCP tools/call '{}' exit code {}",
                                        cmd.name, exit_code
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
                                            duration_ms: 0,
                                            triggered_by: "mcp",
                                        });
                                    }
                                    json!({
                                        "jsonrpc": "2.0",
                                        "id": request.id,
                                        "result": {
                                            "content": [{
                                                "type": "text",
                                                "text": format!("exit_code: {}\n\nstdout:\n{}\n\nstderr:\n{}", exit_code, stdout, stderr)
                                            }],
                                            "is_error": exit_code != 0
                                        }
                                    })
                                }
                                Err(e) => json!({
                                    "jsonrpc": "2.0",
                                    "id": request.id,
                                    "error": {
                                        "code": -32000,
                                        "message": format!("Failed to execute command: {}", e)
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

                    let rendered = skill.instructions.replace("{{input}}", &input);
                    let steps = extract_skill_steps(&rendered);

                    if steps.is_empty() {
                        json!({
                            "jsonrpc": "2.0",
                            "id": request.id,
                            "result": {
                                "content": [{
                                    "type": "text",
                                    "text": rendered
                                }],
                                "is_error": false
                            }
                        })
                    } else {
                        let mut output = String::new();
                        let mut is_error = false;

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

                            match execute_shell_with_timeout(
                                step,
                                Duration::from_secs(SKILL_EXEC_TIMEOUT_SECS),
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

                        let _ = manager.log(format!(
                            "MCP tools/call '{}' skill execution {}",
                            skill.name,
                            if is_error { "failed" } else { "succeeded" }
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
                                duration_ms: 0,
                                triggered_by: "mcp-skill",
                            });
                        }

                        json!({
                            "jsonrpc": "2.0",
                            "id": request.id,
                            "result": {
                                "content": [{
                                    "type": "text",
                                    "text": output
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

    let payload = serde_json::to_string(&response)?;
    let http_response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        payload.len(),
        payload
    );

    stream.write_all(http_response.as_bytes()).await?;
    stream.flush().await?;
    Ok(())
}

fn extract_skill_steps(instructions: &str) -> Vec<String> {
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

async fn read_http_body(stream: &mut TcpStream) -> Result<String> {
    let mut headers_buf = Vec::new();
    let mut temp = [0u8; 4096];

    loop {
        let n = stream.read(&mut temp).await?;
        if n == 0 {
            break;
        }
        headers_buf.extend_from_slice(&temp[..n]);
        if headers_buf.windows(4).any(|w| w == b"\r\n\r\n") {
            break;
        }
        if headers_buf.len() > 1024 * 1024 {
            return Err(AppError::InvalidInput {
                message: "Request headers too large".to_string(),
            });
        }
    }

    let raw = String::from_utf8_lossy(&headers_buf);
    let split_idx = raw.find("\r\n\r\n").ok_or_else(|| AppError::InvalidInput {
        message: "Malformed HTTP request".to_string(),
    })?;
    let headers = &raw[..split_idx];
    let mut body = headers_buf[(split_idx + 4)..].to_vec();

    let content_length = headers
        .lines()
        .find_map(|line| {
            let lower = line.to_lowercase();
            if lower.starts_with("content-length:") {
                line.split(':')
                    .nth(1)
                    .and_then(|v| v.trim().parse::<usize>().ok())
            } else {
                None
            }
        })
        .unwrap_or(body.len());

    while body.len() < content_length {
        let n = stream.read(&mut temp).await?;
        if n == 0 {
            break;
        }
        body.extend_from_slice(&temp[..n]);
        if body.len() > 2 * 1024 * 1024 {
            return Err(AppError::InvalidInput {
                message: "Request body too large".to_string(),
            });
        }
    }

    Ok(String::from_utf8_lossy(&body[..std::cmp::min(body.len(), content_length)]).to_string())
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
