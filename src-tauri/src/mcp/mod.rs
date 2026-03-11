use axum::{
    extract::State,
    http::{HeaderMap, HeaderValue, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::Emitter;
use tokio::sync::{broadcast, Mutex};
use tokio::task::JoinHandle;
use tower_http::cors::CorsLayer;

pub mod watcher;

use crate::constants::{
    limits::{LOG_LIMIT, MAX_OUTPUT_SIZE, MCP_RATE_LIMIT_MAX_CALLS, MCP_SERVER_RETRY_COUNT},
    timing::{
        CMD_EXEC_TIMEOUT, MCP_RATE_LIMIT_WINDOW, MCP_SERVER_BACKOFF_INITIAL_MS, SKILL_EXEC_TIMEOUT,
    },
};
use crate::database::{Database, ExecutionLogInput};
use crate::error::{AppError, Result};
use crate::execution::{
    argument_env_var_name, contains_disallowed_pattern, execute_and_log,
    execute_shell_with_timeout_env_dir, replace_template_with_env_ref, sanitize_argument_value,
    slugify, ExecuteAndLogInput,
};
use crate::models::{Command, Skill, SkillParameterType};
use crate::secrets;

fn mcp_error_response(id: serde_json::Value, code: i64, message: &str) -> serde_json::Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message
        }
    })
}

fn truncate_output_custom(s: String, max_size: usize) -> String {
    if s.len() > max_size {
        let original_len = s.len();
        let mut truncated = s;
        truncated.truncate(max_size);
        truncated.push_str(&format!(
            "\n\n[Output truncated from {} bytes due to size limit]",
            original_len
        ));
        truncated
    } else {
        s
    }
}

fn truncate_output(s: String) -> String {
    truncate_output_custom(s, MAX_OUTPUT_SIZE)
}

const MCP_AUTH_HEADER_NAME: &str = "X-API-Key";
pub const MCP_TOKEN_ENV_VAR: &str = "RULEWEAVER_MCP_TOKEN";

fn mcp_endpoint_url(port: u16) -> String {
    format!("http://127.0.0.1:{port}")
}

fn mcp_standalone_command(port: u16) -> String {
    format!("ruleweaver-mcp --port {port}")
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum McpHealthState {
    Stopped,
    Starting,
    Ready,
    Degraded,
    Error,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum McpDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpDiagnostic {
    pub code: String,
    pub severity: McpDiagnosticSeverity,
    pub title: String,
    pub message: String,
    pub hint: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpStatus {
    pub running: bool,
    pub port: u16,
    pub uptime_seconds: u64,
    pub api_token: Option<String>,
    pub is_watching: bool,
    pub endpoint_url: String,
    pub health_state: McpHealthState,
    pub status_message: String,
    pub diagnostics: Vec<McpDiagnostic>,
    pub available_commands: usize,
    pub available_skills: usize,
    pub watch_target_count: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpConnectionInstructions {
    pub claude_code_json: String,
    pub opencode_json: String,
    pub standalone_command: String,
    pub api_token: String,
    pub endpoint_url: String,
    pub auth_header_name: String,
    pub token_env_var_name: String,
}

#[derive(Debug)]
pub struct McpRuntime {
    running: bool,
    health_state: McpHealthState,
    port: u16,
    api_token: String,
    started_at: Option<Instant>,
    logs: Vec<String>,
    status_message: String,
    last_error_code: Option<String>,
    last_error_message: Option<String>,
    stop_tx: Option<broadcast::Sender<()>>,
    task_handle: Option<JoinHandle<()>>,
    commands: Vec<Command>,
    skills: Vec<Skill>,
    watch_target_count: usize,
    invocation_timestamps: VecDeque<Instant>,
    db: Option<Arc<Database>>,
    watcher: watcher::WatcherManager,
    app_handle: Option<tauri::AppHandle>,
}

#[derive(Clone, Debug)]
pub struct McpManager {
    pub inner: Arc<Mutex<McpRuntime>>,
}

pub struct McpSnapshot {
    pub commands: Vec<Command>,
    pub skills: Vec<Skill>,
    pub db: Option<Arc<Database>>,
}

fn spawn_refresh_task(manager: McpManager, db: Arc<Database>) {
    tokio::spawn(async move {
        let _ = manager
            .log("Detected artifact changes, refreshing tools...".to_string())
            .await;
        let _ = manager.refresh_commands(&db).await;
    });
}

impl McpManager {
    pub fn new(port: u16) -> Self {
        let api_token = uuid::Uuid::new_v4().to_string();
        Self {
            inner: Arc::new(Mutex::new(McpRuntime {
                running: false,
                health_state: McpHealthState::Stopped,
                port,
                api_token,
                started_at: None,
                logs: Vec::new(),
                status_message: "MCP server is stopped".to_string(),
                last_error_code: None,
                last_error_message: None,
                stop_tx: None,
                task_handle: None,
                commands: Vec::new(),
                skills: Vec::new(),
                watch_target_count: 0,
                invocation_timestamps: VecDeque::new(),
                db: None,
                watcher: watcher::WatcherManager::new(),
                app_handle: None,
            })),
        }
    }

    pub async fn set_api_token(&self, token: String) {
        let mut state = self.inner.lock().await;
        state.api_token = token;
    }

    pub async fn refresh_commands(&self, db: &Database) -> Result<()> {
        let (commands, skills) = db.get_mcp_data().await?;

        let app_handle = {
            let mut state = self.inner.lock().await;
            state.commands = commands;
            state.skills = skills;

            let mut paths = std::collections::HashSet::new();
            for skill in &state.skills {
                if skill.enabled && !skill.directory_path.is_empty() {
                    paths.insert(std::path::PathBuf::from(&skill.directory_path));
                }
            }
            for cmd in &state.commands {
                for path in &cmd.target_paths {
                    if !path.is_empty() {
                        paths.insert(std::path::PathBuf::from(path));
                    }
                }
            }
            let paths_vec = paths.into_iter().collect::<Vec<_>>();
            state.watch_target_count = paths_vec.len();

            if !matches!(
                state.health_state,
                McpHealthState::Stopped | McpHealthState::Error
            ) {
                let manager_clone = self.clone();
                if let Some(db_arc) = state.db.clone() {
                    if let Err(e) = state.watcher.start(paths_vec, move || {
                        spawn_refresh_task(manager_clone.clone(), Arc::clone(&db_arc));
                    }) {
                        log::error!("Failed to start artifact watcher: {}", e);
                    }
                }
            }

            state.app_handle.clone()
        };

        if let Some(app) = app_handle {
            let _ = app.emit("mcp-artifacts-refreshed", ());
        }

        Ok(())
    }

    async fn snapshot(&self) -> Result<McpSnapshot> {
        let state = self.inner.lock().await;
        Ok(McpSnapshot {
            commands: state.commands.clone(),
            skills: state.skills.clone(),
            db: state.db.clone(),
        })
    }

    pub async fn set_app_handle(&self, handle: tauri::AppHandle) {
        let mut state = self.inner.lock().await;
        state.app_handle = Some(handle);
    }

    async fn set_error_state(&self, code: &str, message: String) {
        let mut state = self.inner.lock().await;
        state.running = false;
        state.health_state = McpHealthState::Error;
        state.status_message = message.clone();
        state.last_error_code = Some(code.to_string());
        state.last_error_message = Some(message);
        state.started_at = None;
        state.stop_tx = None;
        state.watcher.stop();
    }

    async fn bind_listener(&self, port: u16) -> Result<tokio::net::TcpListener> {
        let addr = format!("127.0.0.1:{port}");
        let mut retry_count = 0;
        let mut backoff_ms = MCP_SERVER_BACKOFF_INITIAL_MS;

        loop {
            match tokio::net::TcpListener::bind(&addr).await {
                Ok(listener) => return Ok(listener),
                Err(error) => {
                    if retry_count >= MCP_SERVER_RETRY_COUNT {
                        let code = if error.kind() == std::io::ErrorKind::AddrInUse {
                            "port_conflict"
                        } else {
                            "startup_failed"
                        };
                        let message = format!(
                            "Failed to bind MCP server on {addr} after {} attempts: {error}",
                            MCP_SERVER_RETRY_COUNT + 1
                        );
                        self.log(message.clone()).await?;
                        self.set_error_state(code, message.clone()).await;
                        return Err(AppError::Mcp(message));
                    }

                    self.log(format!("Port {port} busy, retrying in {backoff_ms}ms..."))
                        .await?;
                    tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                    retry_count += 1;
                    backoff_ms *= 2;
                }
            }
        }
    }

    pub async fn start(&self, db: &Arc<Database>) -> Result<()> {
        let port = {
            let mut state = self.inner.lock().await;
            if matches!(
                state.health_state,
                McpHealthState::Starting | McpHealthState::Ready | McpHealthState::Degraded
            ) {
                return Ok(());
            }

            state.running = false;
            state.health_state = McpHealthState::Starting;
            state.status_message =
                format!("Starting MCP server on {}", mcp_endpoint_url(state.port));
            state.last_error_code = None;
            state.last_error_message = None;
            state.started_at = None;
            state.logs.push("Starting MCP server".to_string());
            state.db = Some(Arc::clone(db));
            state.port
        };

        if let Err(error) = self.refresh_commands(db).await {
            let message = format!("Failed to refresh MCP artifacts before start: {error}");
            self.log(message.clone()).await?;
            self.set_error_state("refresh_failed", message).await;
            return Err(error);
        }

        let listener = self.bind_listener(port).await?;

        let (stop_tx, _) = broadcast::channel(1);
        {
            let mut state = self.inner.lock().await;
            state.stop_tx = Some(stop_tx.clone());
            state.running = true;
            state.health_state = McpHealthState::Ready;
            state.status_message =
                format!("Listening for MCP clients on {}", mcp_endpoint_url(port));
            state.started_at = Some(Instant::now());
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

            let _ = manager
                .log(format!(
                    "MCP server listening on {}",
                    mcp_endpoint_url(port)
                ))
                .await;

            if let Err(e) = axum::serve(listener, app)
                .with_graceful_shutdown(async move {
                    let _ = stop_rx.recv().await;
                })
                .await
            {
                let message = format!("MCP server error: {}", e);
                let _ = manager.log(message.clone()).await;
                manager.set_error_state("server_error", message).await;
                return;
            }

            let _ = manager.log("MCP server stopped".to_string()).await;
            let _ = manager.mark_stopped().await;
        });

        {
            let mut state = self.inner.lock().await;
            state.task_handle = Some(handle);
        }

        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        let (tx, should_mark_stopped_immediately) = {
            let mut state = self.inner.lock().await;
            if !state.running && !matches!(state.health_state, McpHealthState::Starting) {
                return Ok(());
            }
            state.watcher.stop();
            state.status_message = "Stopping MCP server".to_string();
            let tx = state.stop_tx.take();
            (tx, !state.running)
        };

        if let Some(tx) = tx {
            let _ = tx.send(());
        } else if should_mark_stopped_immediately {
            self.mark_stopped().await?;
        }

        Ok(())
    }

    pub async fn wait_until_stopped(&self) -> Result<()> {
        let handle = {
            let mut state = self.inner.lock().await;
            state.task_handle.take()
        };

        if let Some(handle) = handle {
            let _ = handle.await;
        }
        Ok(())
    }

    pub async fn status(&self) -> Result<McpStatus> {
        let state = self.inner.lock().await;
        let uptime_seconds = state.started_at.map(|t| t.elapsed().as_secs()).unwrap_or(0);
        let endpoint_url = mcp_endpoint_url(state.port);
        let has_exposed_artifacts = !state.commands.is_empty() || !state.skills.is_empty();
        let watcher_expected = state.watch_target_count > 0;
        let is_watching = state.watcher.is_watching();

        let health_state = if matches!(state.health_state, McpHealthState::Ready)
            && ((!has_exposed_artifacts) || (watcher_expected && !is_watching))
        {
            McpHealthState::Degraded
        } else {
            state.health_state.clone()
        };

        let status_message = match health_state {
            McpHealthState::Degraded if !has_exposed_artifacts => {
                "MCP server is reachable, but no commands or skills are exposed yet".to_string()
            }
            McpHealthState::Degraded if watcher_expected && !is_watching => {
                "MCP server is reachable, but artifact watching is unavailable".to_string()
            }
            _ => state.status_message.clone(),
        };

        let mut diagnostics = Vec::new();

        match health_state {
            McpHealthState::Stopped => diagnostics.push(McpDiagnostic {
                code: "server_stopped".to_string(),
                severity: McpDiagnosticSeverity::Info,
                title: "Server not running".to_string(),
                message: "Start the MCP server before connecting standalone clients.".to_string(),
                hint: Some("Use Start Server, then copy one of the configurations below.".to_string()),
            }),
            McpHealthState::Starting => diagnostics.push(McpDiagnostic {
                code: "server_starting".to_string(),
                severity: McpDiagnosticSeverity::Info,
                title: "Server is starting".to_string(),
                message: format!("RuleWeaver is preparing MCP on {endpoint_url}."),
                hint: Some("Wait for the status to switch to Ready before retrying your client.".to_string()),
            }),
            McpHealthState::Error => diagnostics.push(McpDiagnostic {
                code: state
                    .last_error_code
                    .clone()
                    .unwrap_or_else(|| "startup_failed".to_string()),
                severity: McpDiagnosticSeverity::Error,
                title: if state.last_error_code.as_deref() == Some("port_conflict") {
                    "Port conflict"
                } else {
                    "Startup failure"
                }
                .to_string(),
                message: state
                    .last_error_message
                    .clone()
                    .unwrap_or_else(|| "The MCP server could not start.".to_string()),
                hint: Some(if state.last_error_code.as_deref() == Some("port_conflict") {
                    format!(
                        "Another process is already using port {}. Stop that process or change the MCP port, then retry.",
                        state.port
                    )
                } else {
                    "Review the recent logs below, then retry starting the server.".to_string()
                }),
            }),
            McpHealthState::Ready | McpHealthState::Degraded => {}
        }

        if !has_exposed_artifacts {
            diagnostics.push(McpDiagnostic {
                code: "no_tools_exposed".to_string(),
                severity: McpDiagnosticSeverity::Warning,
                title: "No MCP tools or skills exposed".to_string(),
                message: "Clients can connect, but they will not see any callable tools until a command is exposed via MCP or a skill is added.".to_string(),
                hint: Some("Enable 'Expose via MCP' on at least one command or create a skill.".to_string()),
            });
        }

        if watcher_expected && !is_watching {
            diagnostics.push(McpDiagnostic {
                code: "watcher_inactive".to_string(),
                severity: McpDiagnosticSeverity::Warning,
                title: "Artifact watcher inactive".to_string(),
                message: "RuleWeaver cannot currently watch local MCP artifacts for changes.".to_string(),
                hint: Some("Use Refresh after changing tools, and restart MCP if automatic updates stay stale.".to_string()),
            });
        }

        diagnostics.push(McpDiagnostic {
            code: "client_configuration".to_string(),
            severity: McpDiagnosticSeverity::Info,
            title: "Verify client configuration".to_string(),
            message: format!(
                "Standalone clients should target {endpoint_url} and send the {MCP_AUTH_HEADER_NAME} header with the current API token."
            ),
            hint: Some(
                "If a client still fails after updating the config, fully restart the client to clear stale protocol or version state."
                    .to_string(),
            ),
        });

        Ok(McpStatus {
            running: matches!(
                health_state,
                McpHealthState::Ready | McpHealthState::Degraded
            ),
            port: state.port,
            uptime_seconds,
            api_token: Some(state.api_token.clone()),
            is_watching,
            endpoint_url,
            health_state,
            status_message,
            diagnostics,
            available_commands: state.commands.len(),
            available_skills: state.skills.len(),
            watch_target_count: state.watch_target_count,
        })
    }

    pub async fn logs(&self, limit: usize) -> Result<Vec<String>> {
        let state = self.inner.lock().await;
        let len = state.logs.len();
        let start = len.saturating_sub(limit);
        Ok(state.logs[start..].to_vec())
    }

    pub async fn instructions(&self) -> Result<McpConnectionInstructions> {
        let status = self.status().await?;
        let port = status.port;
        let endpoint_url = status.endpoint_url.clone();
        let token = status.api_token.clone().unwrap_or_default();

        let claude_code_json = serde_json::to_string_pretty(&json!({
            "mcpServers": {
                "ruleweaver": {
                    "url": endpoint_url,
                    "env": {
                        "X_API_KEY": token
                    }
                }
            }
        }))
        .map_err(AppError::Serialization)?;

        let opencode_json = serde_json::to_string_pretty(&json!({
            "mcp": {
                "servers": [
                    {
                        "name": "ruleweaver",
                        "url": mcp_endpoint_url(port),
                        "headers": {
                            MCP_AUTH_HEADER_NAME: token
                        }
                    }
                ]
            }
        }))
        .map_err(AppError::Serialization)?;

        Ok(McpConnectionInstructions {
            claude_code_json,
            opencode_json,
            standalone_command: mcp_standalone_command(port),
            api_token: token,
            endpoint_url: mcp_endpoint_url(port),
            auth_header_name: MCP_AUTH_HEADER_NAME.to_string(),
            token_env_var_name: MCP_TOKEN_ENV_VAR.to_string(),
        })
    }

    async fn log(&self, message: String) -> Result<()> {
        let mut state = self.inner.lock().await;
        state.logs.push(message);
        if state.logs.len() > LOG_LIMIT {
            let drain_to = state.logs.len() - LOG_LIMIT;
            state.logs.drain(0..drain_to);
        }
        Ok(())
    }

    async fn mark_stopped(&self) -> Result<()> {
        let mut state = self.inner.lock().await;
        state.running = false;
        state.health_state = McpHealthState::Stopped;
        state.status_message = "MCP server is stopped".to_string();
        state.stop_tx = None;
        state.started_at = None;
        state.watcher.stop();
        Ok(())
    }

    async fn allow_invocation(&self) -> Result<bool> {
        let mut state = self.inner.lock().await;
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
    headers: HeaderMap,
    Json(request): Json<JsonRpcRequest>,
) -> Response {
    let auth_valid = {
        let state = manager.inner.lock().await;

        let provided_key = headers.get("X-API-Key").and_then(|v| v.to_str().ok());

        provided_key == Some(&state.api_token)
    };

    if !auth_valid {
        return (
            StatusCode::UNAUTHORIZED,
            "Unauthorized: Invalid or missing X-API-Key header",
        )
            .into_response();
    }

    let McpSnapshot {
        commands,
        skills,
        db: shared_db,
    } = match manager.snapshot().await {
        Ok(s) => s,
        Err(_e) => {
            return Json(json!({
                "jsonrpc": "2.0",
                "id": request.id,
                "error": {
                    "code": -32603,
                    "message": "Internal server error"
                }
            }))
            .into_response();
        }
    };

    let response = match request.method.as_str() {
        "initialize" => handle_initialize(request.id),
        "tools/list" => handle_tools_list(request.id, &commands, &skills),
        "tools/call" => {
            handle_tools_call(
                &manager,
                request.id,
                request.params,
                &commands,
                &skills,
                &shared_db,
            )
            .await
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

    Json(response).into_response()
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

struct McpToolParameter {
    name: String,
    description: String,
    required: bool,
    enum_values: Option<Vec<String>>,
    param_type: SkillParameterType,
}

fn handle_tools_list(
    id: serde_json::Value,
    commands: &[Command],
    skills: &[Skill],
) -> serde_json::Value {
    let mut tools: Vec<serde_json::Value> = commands
        .iter()
        .filter(|c| c.expose_via_mcp)
        .map(|c| {
            let params: Vec<_> = c
                .arguments
                .iter()
                .map(|a| {
                    let p_type = if let Some(ref opts) = a.options {
                        if !opts.is_empty() {
                            SkillParameterType::Enum
                        } else {
                            SkillParameterType::String
                        }
                    } else {
                        SkillParameterType::String
                    };

                    McpToolParameter {
                        name: a.name.clone(),
                        description: a.description.clone(),
                        required: a.required,
                        enum_values: a.options.clone(),
                        param_type: p_type,
                    }
                })
                .collect();

            build_mcp_tool_schema(
                &format!("{}-{}", slugify(&c.name), &c.id[..8]),
                &c.description,
                &params,
            )
        })
        .collect();

    let skill_tools: Vec<serde_json::Value> = skills
        .iter()
        .filter(|s| s.enabled)
        .map(|s| {
            let params: Vec<_> = s
                .input_schema
                .iter()
                .map(|p| McpToolParameter {
                    name: p.name.clone(),
                    description: p.description.clone(),
                    required: p.required,
                    enum_values: p.enum_values.clone(),
                    param_type: p.param_type.clone(),
                })
                .collect();

            build_mcp_tool_schema(
                &format!("skill_{}-{}", slugify(&s.name), &s.id[..8]),
                &s.description,
                &params,
            )
        })
        .collect();

    tools.extend(skill_tools);

    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": { "tools": tools }
    })
}

fn build_mcp_tool_schema(
    name: &str,
    description: &str,
    params: &[McpToolParameter],
) -> serde_json::Value {
    let mut props = serde_json::Map::new();
    let mut required: Vec<String> = Vec::new();

    for param in params {
        let type_str = match param.param_type {
            SkillParameterType::Number => "number",
            SkillParameterType::Boolean => "boolean",
            SkillParameterType::Array => "array",
            SkillParameterType::Object => "object",
            _ => "string",
        };

        let mut prop_schema = json!({
            "type": type_str,
            "description": param.description,
        });

        if let Some(ref enum_vals) = param.enum_values {
            prop_schema
                .as_object_mut()
                .unwrap()
                .insert("enum".to_string(), json!(enum_vals));
        }

        props.insert(param.name.clone(), prop_schema);
        if param.required {
            required.push(param.name.clone());
        }
    }

    json!({
        "name": name,
        "description": description,
        "inputSchema": {
            "type": "object",
            "properties": props,
            "required": required,
        }
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
    let allow = match manager.allow_invocation().await {
        Ok(a) => a,
        Err(_) => {
            return json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32603,
                    "message": "Internal server error"
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
    if let Some(pattern) = contains_disallowed_pattern(&cmd.script) {
        return mcp_error_response(
            id,
            -32602,
            &format!("Command script contains a disallowed pattern: {}", pattern),
        );
    }

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
        return mcp_error_response(
            id,
            -32602,
            &format!(
                "Missing required arguments: {}",
                missing_required.join(", ")
            ),
        );
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
        return mcp_error_response(id, -32602, &format!("Invalid argument value: {}", message));
    }

    let args_json = match serde_json::to_string(&args_map) {
        Ok(s) => s,
        Err(e) => {
            return json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32603,
                    "message": format!("Serialization error: {}", e)
                }
            });
        }
    };

    if let Some(db) = shared_db {
        match secrets::resolve_command_secret_envs(db.as_ref(), cmd).await {
            Ok(secret_envs) => envs.extend(secret_envs),
            Err(e) => {
                return json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "content": [{
                            "type": "text",
                            "text": format!("Failed to resolve command secrets: {}", e)
                        }],
                        "isError": true
                    }
                });
            }
        }
    }

    match execute_and_log(ExecuteAndLogInput {
        db: shared_db.as_ref().map(|arc| arc.as_ref()),
        command_id: &cmd.id,
        command_name: &cmd.name,
        script: &rendered,
        timeout_dur: CMD_EXEC_TIMEOUT,
        envs: &envs,
        arguments_json: &args_json,
        triggered_by: "mcp",
        max_retries: cmd.max_retries,
        adapter_context: Some("mcp"),
    })
    .await
    {
        Ok((exit_code, stdout, stderr, duration_ms)) => {
            let _ = manager
                .log(format!(
                    "MCP tools/call '{}' exit code {} ({}ms)",
                    cmd.name, exit_code, duration_ms
                ))
                .await;
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
    let final_envs = match skill.validate_payload(&args_map) {
        Ok(envs) => envs,
        Err(e) => {
            return mcp_error_response(id, -32602, &e.to_string());
        }
    };

    let mut final_envs = final_envs;

    final_envs.push((
        crate::constants::skills::RULEWEAVER_SKILL_ID.to_string(),
        skill.id.clone(),
    ));
    final_envs.push((
        crate::constants::skills::RULEWEAVER_SKILL_NAME.to_string(),
        skill.name.clone(),
    ));
    final_envs.push((
        crate::constants::skills::RULEWEAVER_SKILL_DIR.to_string(),
        skill.directory_path.clone(),
    ));

    // Inject filtered scoped secrets and preserve legacy allowlist fallback
    if let Some(db) = shared_db {
        let allowlist = db
            .get_setting("mcp_secrets_allowlist")
            .await
            .ok()
            .flatten()
            .unwrap_or_default();
        let allowed_keys = allowlist
            .split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect::<std::collections::HashSet<_>>();

        match secrets::resolve_skill_secret_envs(db.as_ref(), skill, &allowed_keys).await {
            Ok(secret_envs) => {
                for env in secret_envs {
                    final_envs.push(env);
                }
            }
            Err(e) => {
                return mcp_error_response(
                    id,
                    -32603,
                    &format!("Failed to resolve skill secrets: {}", e),
                );
            }
        }
    }

    let start = Instant::now();
    let mut output = String::new();
    let mut is_error = false;

    if let Err(e) = crate::models::validate_skill_input(&skill.name, &skill.instructions) {
        return mcp_error_response(id, -32602, &e.to_string());
    }
    if let Err(e) = crate::models::validate_skill_schema(&skill.input_schema) {
        return mcp_error_response(id, -32602, &e.to_string());
    }
    if let Err(e) = crate::models::validate_skill_entry_point(&skill.entry_point) {
        return mcp_error_response(id, -32602, &e.to_string());
    }

    let entry_point = if skill.entry_point.is_empty() {
        return mcp_error_response(id, -32603, "Skill has no entry point defined");
    } else {
        skill.entry_point.clone()
    };

    let resolved_path = crate::path_resolver::resolve_workspace_path(
        &skill.directory_path,
        skill.base_path.as_deref(),
    );
    let dir = std::path::PathBuf::from(&resolved_path);
    if !dir.exists() || !dir.is_dir() {
        return json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": -32603,
                "message": format!("Skill directory does not exist: {}", skill.directory_path)
            }
        });
    }

    if let Some(pattern) = contains_disallowed_pattern(&entry_point) {
        return json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": -32602,
                "message": format!("Entry point rejected due to unsafe pattern: {}", pattern)
            }
        });
    }

    // Security: Canonicalize entry point to prevent directory traversal
    let canonical_skill_dir = match std::fs::canonicalize(&dir) {
        Ok(p) => p,
        Err(e) => {
            return json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32603, "message": format!("Failed to canonicalize skill directory: {}", e) }
            })
        }
    };

    let full_entry_path = dir.join(&entry_point);
    let canonical_entry_path = match std::fs::canonicalize(&full_entry_path) {
        Ok(p) => p,
        Err(e) => {
            return json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32603, "message": format!("Entry point not found or invalid: {}", e) }
            })
        }
    };

    if !canonical_entry_path.starts_with(&canonical_skill_dir) {
        return json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": -32603,
                "message": "Security Violation: Entry point must be within the skill directory"
            }
        });
    }

    match execute_shell_with_timeout_env_dir(
        &entry_point,
        SKILL_EXEC_TIMEOUT,
        &final_envs,
        Some(dir),
    )
    .await
    {
        Ok((exit_code, stdout, stderr)) => {
            let step_stdout = truncate_output_custom(
                stdout,
                crate::constants::limits::MAX_SKILL_OUTPUT_PER_STREAM,
            );
            let step_stderr = truncate_output_custom(
                stderr,
                crate::constants::limits::MAX_SKILL_OUTPUT_PER_STREAM,
            );

            output.push_str(&format!(
                "exit_code: {}\nstdout:\n{}\nstderr:\n{}",
                exit_code, step_stdout, step_stderr
            ));
            if exit_code != 0 {
                is_error = true;
            }
        }
        Err(e) => {
            is_error = true;
            output.push_str(&format!("execution error: {}\n", e));
        }
    }

    let duration_ms = start.elapsed().as_millis() as u64;
    let _ = manager
        .log(format!(
            "MCP tools/call '{}' skill execution {} ({}ms)",
            skill.name,
            if is_error { "failed" } else { "succeeded" },
            duration_ms
        ))
        .await;

    if let Some(db) = shared_db {
        let args_json = match serde_json::to_string(&args_map) {
            Ok(s) => s,
            Err(e) => {
                let _ = manager
                    .log(format!("Skill execution serialization error: {}", e))
                    .await;
                String::new()
            }
        };
        let skill_name = format!("skill:{}", skill.name);
        let (stdout_redacted, was_redacted) = crate::redaction::redact(&output);
        let _ = db
            .add_execution_log(&ExecutionLogInput {
                command_id: &skill.id,
                command_name: &skill_name,
                arguments_json: &args_json,
                stdout: &stdout_redacted,
                stderr: "",
                exit_code: if is_error { 1 } else { 0 },
                duration_ms,
                triggered_by: "mcp-skill",
                failure_class: None,
                adapter_context: Some("mcp-skill"),
                is_redacted: was_redacted,
                attempt_number: 1,
            })
            .await;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("My Skill"), "my-skill");
        assert_eq!(slugify("Skill__Name"), "skill-name");
    }

    #[test]
    fn test_disallowed_patterns() {
        assert!(contains_disallowed_pattern("rm -rf /").is_some());
        assert!(contains_disallowed_pattern("echo hi").is_none());
    }

    #[test]
    fn test_standalone_command_omits_token() {
        assert_eq!(mcp_standalone_command(4545), "ruleweaver-mcp --port 4545");
        assert!(!mcp_standalone_command(4545).contains("token"));
    }
}
