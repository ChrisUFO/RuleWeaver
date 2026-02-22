use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::Command as ProcessCommand;
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::database::Database;
use crate::error::{AppError, Result};
use crate::models::{Command, Skill};

const MAX_SKILL_STEPS: usize = 10;
const MAX_STEP_LENGTH: usize = 4000;

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
    stop_tx: Option<Sender<()>>,
    commands: Vec<Command>,
    skills: Vec<Skill>,
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

    fn snapshot(&self) -> Result<(Vec<Command>, Vec<Skill>)> {
        let state = self.inner.lock().map_err(|_| AppError::LockError)?;
        Ok((state.commands.clone(), state.skills.clone()))
    }

    pub fn start(&self, db: &Database) -> Result<()> {
        self.refresh_commands(db)?;

        let port = {
            let mut state = self.inner.lock().map_err(|_| AppError::LockError)?;
            if state.running {
                return Ok(());
            }
            state.running = true;
            state.started_at = Some(Instant::now());
            state.logs.push("Starting MCP server".to_string());
            state.port
        };

        let (stop_tx, stop_rx): (Sender<()>, Receiver<()>) = channel();
        {
            let mut state = self.inner.lock().map_err(|_| AppError::LockError)?;
            state.stop_tx = Some(stop_tx);
        }

        let manager = self.clone();
        thread::spawn(move || {
            let bind_addr = format!("127.0.0.1:{}", port);
            let listener = match TcpListener::bind(&bind_addr) {
                Ok(listener) => listener,
                Err(e) => {
                    let _ = manager.log(format!("Failed to bind MCP server {}: {}", bind_addr, e));
                    let _ = manager.mark_stopped();
                    return;
                }
            };
            let _ = listener.set_nonblocking(true);
            let _ = manager.log(format!("MCP server listening on {}", bind_addr));

            loop {
                match stop_rx.try_recv() {
                    Ok(()) | Err(TryRecvError::Disconnected) => break,
                    Err(TryRecvError::Empty) => {}
                }

                match listener.accept() {
                    Ok((mut stream, addr)) => {
                        let _ = manager.log(format!("Client connected: {}", addr));
                        let _ = handle_client(&mut stream, &manager);
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(50));
                    }
                    Err(e) => {
                        let _ = manager.log(format!("Listener error: {}", e));
                        thread::sleep(Duration::from_millis(200));
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
}

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    id: serde_json::Value,
    method: String,
    params: Option<serde_json::Value>,
}

fn handle_client(stream: &mut TcpStream, manager: &McpManager) -> Result<()> {
    let mut buffer = vec![0u8; 64 * 1024];
    let bytes_read = stream.read(&mut buffer)?;
    if bytes_read == 0 {
        return Ok(());
    }
    let raw = String::from_utf8_lossy(&buffer[..bytes_read]);

    let body = if let Some(idx) = raw.find("\r\n\r\n") {
        &raw[idx + 4..]
    } else {
        raw.as_ref()
    };

    let request: JsonRpcRequest = match serde_json::from_str(body.trim()) {
        Ok(req) => req,
        Err(e) => {
            let _ = manager.log(format!("Invalid JSON-RPC request: {}", e));
            return Ok(());
        }
    };

    let (commands, skills) = manager.snapshot()?;

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
                    for arg in &cmd.arguments {
                        let token = format!("{{{{{}}}}}", arg.name);
                        let value = args_map
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
                        rendered = rendered.replace(&token, &value);
                    }

                    #[cfg(target_os = "windows")]
                    let output = std::process::Command::new("cmd")
                        .args(["/C", &rendered])
                        .output();

                    #[cfg(not(target_os = "windows"))]
                    let output = std::process::Command::new("sh")
                        .args(["-c", &rendered])
                        .output();

                    match output {
                        Ok(output) => {
                            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                            let exit_code = output.status.code().unwrap_or(-1);
                            let _ = manager.log(format!(
                                "MCP tools/call '{}' exit code {}",
                                cmd.name, exit_code
                            ));
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

                        match execute_shell(step) {
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

    stream.write_all(http_response.as_bytes())?;
    stream.flush()?;
    Ok(())
}

fn slugify(input: &str) -> String {
    let mut out = String::new();
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if ch.is_whitespace() || ch == '-' || ch == '_' {
            out.push('-');
        }
    }
    while out.contains("--") {
        out = out.replace("--", "-");
    }
    out.trim_matches('-').to_string()
}

fn extract_skill_steps(instructions: &str) -> Vec<String> {
    let re = match Regex::new(r"(?s)```(?:bash|sh|shell|powershell|pwsh|cmd)?\n(.*?)```") {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    re.captures_iter(instructions)
        .filter_map(|caps| caps.get(1).map(|m| m.as_str().trim().to_string()))
        .filter(|s| !s.is_empty())
        .collect()
}

fn contains_disallowed_pattern(step: &str) -> Option<&'static str> {
    let lower = step.to_lowercase();
    let patterns: [(&str, &str); 8] = [
        ("rm -rf", "rm -rf"),
        ("del /f", "del /f"),
        ("format ", "format"),
        ("mkfs", "mkfs"),
        ("shutdown", "shutdown"),
        ("reboot", "reboot"),
        ("curl |", "curl pipe"),
        ("wget |", "wget pipe"),
    ];

    for (needle, name) in patterns {
        if lower.contains(needle) {
            return Some(name);
        }
    }

    None
}

fn execute_shell(script: &str) -> Result<(i32, String, String)> {
    #[cfg(target_os = "windows")]
    let output = ProcessCommand::new("cmd").args(["/C", script]).output()?;

    #[cfg(not(target_os = "windows"))]
    let output = ProcessCommand::new("sh").args(["-c", script]).output()?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    Ok((output.status.code().unwrap_or(-1), stdout, stderr))
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
