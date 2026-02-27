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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpStatus {
    pub running: bool,
    pub port: u16,
    pub uptime_seconds: u64,
    pub api_token: Option<String>,
    pub is_watching: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpConnectionInstructions {
    pub claude_code_json: String,
    pub opencode_json: String,
    pub standalone_command: String,
    pub api_token: String,
}

#[derive(Debug)]
pub struct McpRuntime {
    running: bool,
    port: u16,
    api_token: String,
    started_at: Option<Instant>,
    logs: Vec<String>,
    stop_tx: Option<broadcast::Sender<()>>,
    task_handle: Option<JoinHandle<()>>,
    commands: Vec<Command>,
    skills: Vec<Skill>,
    invocation_timestamps: VecDeque<Instant>,
    db: Option<Arc<Database>>,
    watcher: watcher::WatcherManager,
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

impl McpManager {
    pub fn new(port: u16) -> Self {
        let api_token = uuid::Uuid::new_v4().to_string();
        Self {
            inner: Arc::new(Mutex::new(McpRuntime {
                running: false,
                port,
                api_token,
                started_at: None,
                logs: Vec::new(),
                stop_tx: None,
                task_handle: None,
                commands: Vec::new(),
                skills: Vec::new(),
                invocation_timestamps: VecDeque::new(),
                db: None,
                watcher: watcher::WatcherManager::new(),
            })),
        }
    }

    pub async fn set_api_token(&self, token: String) {
        let mut state = self.inner.lock().await;
        state.api_token = token;
    }

    pub async fn refresh_commands(&self, db: &Database) -> Result<()> {
        let (commands, skills) = db.get_mcp_data().await?;
        let mut state = self.inner.lock().await;
        state.commands = commands;
        state.skills = skills;
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

    pub async fn start(&self, db: &Arc<Database>) -> Result<()> {
        self.refresh_commands(db).await?;

        let (port, paths_to_watch) = {
            let mut state = self.inner.lock().await;
            if state.running {
                return Ok(());
            }

            // Collect paths to watch from skills and commands
            let mut paths = std::collections::HashSet::new();
            for skill in &state.skills {
                if skill.enabled {
                    paths.insert(std::path::PathBuf::from(&skill.directory_path));
                }
            }
            for cmd in &state.commands {
                for path in &cmd.target_paths {
                    paths.insert(std::path::PathBuf::from(path));
                }
            }

            state.running = true;
            state.started_at = Some(Instant::now());
            state.logs.push("Starting MCP server".to_string());
            state.db = Some(Arc::clone(db));
            (state.port, paths.into_iter().collect::<Vec<_>>())
        };

        // Start watcher
        {
            let manager_clone = self.clone();
            let db_clone = Arc::clone(db);
            let mut state = self.inner.lock().await;
            state.watcher.start(paths_to_watch, move || {
                let m = manager_clone.clone();
                let d = Arc::clone(&db_clone);
                tokio::spawn(async move {
                    let _ = m.log("Detected artifact changes, refreshing tools...".to_string()).await;
                    let _ = m.refresh_commands(&d).await;
                });
            })?;
        }

        let (stop_tx, _) = broadcast::channel(1);
        {
            let mut state = self.inner.lock().await;
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
                            let _ = manager
                                .log(format!(
                                    "Failed to bind MCP server after {} attempts {}: {}",
                                    MCP_SERVER_RETRY_COUNT, addr, e
                                ))
                                .await;
                            break None;
                        }
                        let _ = manager
                            .log(format!(
                                "Port {} busy, retrying in {}ms...",
                                port, backoff_ms
                            ))
                            .await;
                        tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                        retry_count += 1;
                        backoff_ms *= 2;
                    }
                }
            };

            let listener = match listener {
                Some(l) => l,
                None => {
                    let _ = manager.mark_stopped().await;
                    return;
                }
            };

            let _ = manager
                .log(format!("MCP server listening on {}", addr))
                .await;

            if let Err(e) = axum::serve(listener, app)
                .with_graceful_shutdown(async move {
                    let _ = stop_rx.recv().await;
                })
                .await
            {
                let _ = manager.log(format!("MCP server error: {}", e)).await;
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
        let tx = {
            let mut state = self.inner.lock().await;
            if !state.running {
                return Ok(());
            }
            state.watcher.stop();
            state.stop_tx.take()
        };

        if let Some(tx) = tx {
            let _ = tx.send(());
        }

        Ok(())
    }

    pub fn port(&self) -> u16 {
        // We use a blocking lock here for simplicity as it's just a u16 read
        // In a real high-concurrency app we'd use a separate atomic or RWLock
        let state = self.inner.blocking_lock();
        state.port
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
        Ok(McpStatus {
            running: state.running,
            port: state.port,
            uptime_seconds,
            api_token: Some(state.api_token.clone()),
            is_watching: state.watcher.is_watching(),
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
        let token = status.api_token.clone().unwrap_or_default();

        let claude_code_json = serde_json::to_string_pretty(&json!({
            "mcpServers": {
                "ruleweaver": {
                    "url": format!("http://127.0.0.1:{}", port),
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
                        "url": format!("http://127.0.0.1:{}", port),
                        "headers": {
                            "X-API-Key": token
                        }
                    }
                ]
            }
        }))
        .map_err(AppError::Serialization)?;

        Ok(McpConnectionInstructions {
            claude_code_json,
            opencode_json,
            standalone_command: format!("ruleweaver-mcp --port {} --token {}", port, token),
            api_token: token,
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

    // Inject filtered secrets as SKILL_SECRET_*
    if let Some(db) = shared_db {
        let allowlist = db
            .get_setting("mcp_secrets_allowlist")
            .await
            .ok()
            .flatten()
            .unwrap_or_default();
        let allowed_keys: Vec<String> = allowlist
            .split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect();

        if let Ok(settings) = db.get_all_settings().await {
            for (k, v) in settings {
                let low_k = k.to_lowercase();
                if allowed_keys.contains(&low_k) {
                    let env_name = format!(
                        "{}{}",
                        crate::constants::skills::SKILL_SECRET_PREFIX,
                        k.replace('-', "_").to_uppercase()
                    );
                    final_envs.push((env_name, v));
                }
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

    let dir = std::path::PathBuf::from(&skill.directory_path);
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
}
