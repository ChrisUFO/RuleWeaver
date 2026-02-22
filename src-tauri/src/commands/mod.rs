use crate::database::{get_app_data_path, Database, ExecutionLogInput};
use crate::error::{AppError, Result};
use crate::execution::{
    argument_env_var_name, execute_shell_with_timeout_env, replace_template_with_env_ref,
    sanitize_argument_value, slugify,
};
use crate::file_storage;
use crate::mcp::{McpConnectionInstructions, McpManager, McpStatus};
use crate::models::{
    Command, CreateCommandInput, CreateRuleInput, CreateSkillInput, ExecutionLog, Rule, Skill,
    SyncError, SyncHistoryEntry, SyncResult, TestCommandResult, UpdateCommandInput,
    UpdateRuleInput, UpdateSkillInput,
};
use crate::sync::SyncEngine;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tauri::State;

fn validate_path(path: &str) -> Result<PathBuf> {
    let canonical_path = std::fs::canonicalize(path).map_err(|e| AppError::InvalidInput {
        message: format!("Invalid path: {}", e),
    })?;

    let home_dir = dirs::home_dir()
        .ok_or_else(|| AppError::Path("Could not determine home directory".to_string()))?;
    let canonical_home = std::fs::canonicalize(&home_dir).unwrap_or(home_dir);

    if !canonical_path.starts_with(&canonical_home) {
        return Err(AppError::InvalidInput {
            message: "Path must be within user's home directory".to_string(),
        });
    }

    Ok(canonical_path)
}

const MAX_RULE_NAME_LENGTH: usize = 200;
const MAX_RULE_CONTENT_LENGTH: usize = 1_000_000;
const MAX_COMMAND_NAME_LENGTH: usize = 120;
const MAX_COMMAND_SCRIPT_LENGTH: usize = 10_000;
const MAX_SKILL_NAME_LENGTH: usize = 160;
const MAX_SKILL_INSTRUCTIONS_LENGTH: usize = 200_000;

fn validate_rule_input(name: &str, content: &str) -> Result<()> {
    let trimmed_name = name.trim();
    if trimmed_name.is_empty() {
        return Err(AppError::InvalidInput {
            message: "Rule name cannot be empty".to_string(),
        });
    }
    if trimmed_name.len() > MAX_RULE_NAME_LENGTH {
        return Err(AppError::InvalidInput {
            message: format!(
                "Rule name too long (max {} characters)",
                MAX_RULE_NAME_LENGTH
            ),
        });
    }
    if content.len() > MAX_RULE_CONTENT_LENGTH {
        return Err(AppError::InvalidInput {
            message: format!(
                "Rule content too large (max {} characters)",
                MAX_RULE_CONTENT_LENGTH
            ),
        });
    }
    Ok(())
}

fn validate_command_input(name: &str, script: &str) -> Result<()> {
    let trimmed_name = name.trim();
    if trimmed_name.is_empty() {
        return Err(AppError::InvalidInput {
            message: "Command name cannot be empty".to_string(),
        });
    }
    if trimmed_name.len() > MAX_COMMAND_NAME_LENGTH {
        return Err(AppError::InvalidInput {
            message: format!(
                "Command name too long (max {} characters)",
                MAX_COMMAND_NAME_LENGTH
            ),
        });
    }
    if script.trim().is_empty() {
        return Err(AppError::InvalidInput {
            message: "Command script cannot be empty".to_string(),
        });
    }
    if script.len() > MAX_COMMAND_SCRIPT_LENGTH {
        return Err(AppError::InvalidInput {
            message: format!(
                "Command script too long (max {} characters)",
                MAX_COMMAND_SCRIPT_LENGTH
            ),
        });
    }
    Ok(())
}

fn validate_skill_input(name: &str, instructions: &str) -> Result<()> {
    let trimmed_name = name.trim();
    if trimmed_name.is_empty() {
        return Err(AppError::InvalidInput {
            message: "Skill name cannot be empty".to_string(),
        });
    }
    if trimmed_name.len() > MAX_SKILL_NAME_LENGTH {
        return Err(AppError::InvalidInput {
            message: format!(
                "Skill name too long (max {} characters)",
                MAX_SKILL_NAME_LENGTH
            ),
        });
    }
    if instructions.trim().is_empty() {
        return Err(AppError::InvalidInput {
            message: "Skill instructions cannot be empty".to_string(),
        });
    }
    if instructions.len() > MAX_SKILL_INSTRUCTIONS_LENGTH {
        return Err(AppError::InvalidInput {
            message: format!(
                "Skill instructions too large (max {} characters)",
                MAX_SKILL_INSTRUCTIONS_LENGTH
            ),
        });
    }
    Ok(())
}

fn markdown_escape_inline(input: &str) -> String {
    input.replace('`', "\\`")
}

fn command_file_targets() -> Result<Vec<(String, String)>> {
    let home = dirs::home_dir()
        .ok_or_else(|| AppError::Path("Could not determine home directory".to_string()))?;
    Ok(vec![
        (
            home.join(".gemini")
                .join("COMMANDS.toml")
                .to_string_lossy()
                .to_string(),
            "gemini".to_string(),
        ),
        (
            home.join(".opencode")
                .join("COMMANDS.md")
                .to_string_lossy()
                .to_string(),
            "opencode".to_string(),
        ),
        (
            home.join(".claude")
                .join("COMMANDS.md")
                .to_string_lossy()
                .to_string(),
            "claude-code".to_string(),
        ),
    ])
}

fn format_commands_toml(commands: &[Command]) -> String {
    let mut out = String::from("# Generated by RuleWeaver - Do not edit manually\n\n");
    for cmd in commands.iter().filter(|c| c.expose_via_mcp) {
        out.push_str("[[command]]\n");
        out.push_str(&format!("name = \"{}\"\n", slugify(&cmd.name)));
        out.push_str(&format!(
            "description = \"{}\"\n",
            cmd.description.replace('"', "\\\"")
        ));
        out.push_str(&format!(
            "script = \"{}\"\n",
            cmd.script.replace('"', "\\\"")
        ));
        if !cmd.arguments.is_empty() {
            out.push_str("[command.arguments]\n");
            for arg in &cmd.arguments {
                out.push_str(&format!(
                    "{} = {{ type = \"string\", required = {} }}\n",
                    arg.name, arg.required
                ));
            }
        }
        out.push('\n');
    }
    out
}

fn format_commands_markdown(commands: &[Command], title: &str) -> String {
    let mut out = format!("# {}\n\n", title);
    out.push_str("<!-- Generated by RuleWeaver - Do not edit manually -->\n\n");
    for cmd in commands.iter().filter(|c| c.expose_via_mcp) {
        out.push_str(&format!("## {}\n\n", cmd.name));
        out.push_str(&format!("{}\n\n", cmd.description));
        out.push_str(&format!(
            "**Command:** `{}`\n\n",
            markdown_escape_inline(&cmd.script)
        ));
        if !cmd.arguments.is_empty() {
            out.push_str("**Arguments:**\n");
            for arg in &cmd.arguments {
                out.push_str(&format!(
                    "- `{}` (string, {}): {}\n",
                    arg.name,
                    if arg.required { "required" } else { "optional" },
                    arg.description
                ));
            }
            out.push('\n');
        }
    }
    out
}

fn use_file_storage(db: &Database) -> bool {
    db.get_storage_mode()
        .map(|mode| mode == "file")
        .unwrap_or(false)
}

const LOCAL_RULE_PATHS_KEY: &str = "local_rule_paths";

fn get_local_rule_roots(db: &Database) -> Vec<PathBuf> {
    db.get_setting(LOCAL_RULE_PATHS_KEY)
        .ok()
        .flatten()
        .and_then(|json| serde_json::from_str::<Vec<String>>(&json).ok())
        .map(|items| items.into_iter().map(PathBuf::from).collect())
        .unwrap_or_default()
}

fn register_local_rule_paths(db: &Database, rule: &Rule) -> Result<()> {
    if !matches!(rule.scope, crate::models::Scope::Local) {
        return Ok(());
    }

    let mut roots = get_local_rule_roots(db)
        .into_iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect::<Vec<_>>();

    if let Some(paths) = &rule.target_paths {
        for path in paths {
            if !roots.iter().any(|p| p == path) {
                roots.push(path.clone());
            }
        }
    }

    let encoded = serde_json::to_string(&roots)?;
    db.set_setting(LOCAL_RULE_PATHS_KEY, &encoded)
}

fn storage_location_for_rule(rule: &Rule) -> file_storage::StorageLocation {
    match rule.scope {
        crate::models::Scope::Global => file_storage::StorageLocation::Global,
        crate::models::Scope::Local => {
            if let Some(paths) = &rule.target_paths {
                if let Some(first) = paths.first() {
                    return file_storage::StorageLocation::Local(PathBuf::from(first));
                }
            }
            file_storage::StorageLocation::Global
        }
    }
}

#[tauri::command]
pub fn get_all_rules(db: State<'_, Arc<Database>>) -> Result<Vec<Rule>> {
    if use_file_storage(&db) {
        let local_roots = get_local_rule_roots(&db);
        let loaded = file_storage::load_rules_from_locations(&local_roots)?;
        Ok(loaded.rules)
    } else {
        db.get_all_rules()
    }
}

#[tauri::command]
pub fn get_rule_by_id(id: String, db: State<'_, Arc<Database>>) -> Result<Rule> {
    if use_file_storage(&db) {
        let local_roots = get_local_rule_roots(&db);
        let loaded = file_storage::load_rules_from_locations(&local_roots)?;
        loaded
            .rules
            .into_iter()
            .find(|r| r.id == id)
            .ok_or_else(|| AppError::RuleNotFound { id })
    } else {
        db.get_rule_by_id(&id)
    }
}

#[tauri::command]
pub fn create_rule(input: CreateRuleInput, db: State<'_, Arc<Database>>) -> Result<Rule> {
    validate_rule_input(&input.name, &input.content)?;

    if matches!(input.scope, crate::models::Scope::Local) {
        if let Some(ref paths) = input.target_paths {
            if paths.is_empty() {
                return Err(AppError::InvalidInput {
                    message: "Local rules must have at least one target path".to_string(),
                });
            }
        } else {
            return Err(AppError::InvalidInput {
                message: "Local rules must have target paths specified".to_string(),
            });
        }
    }

    let created = db.create_rule(input)?;

    if use_file_storage(&db) {
        let location = storage_location_for_rule(&created);
        file_storage::save_rule_to_disk(&created, &location)?;
        db.update_rule_file_index(&created.id, &location)?;
        register_local_rule_paths(&db, &created)?;
    }

    Ok(created)
}

#[tauri::command]
pub fn update_rule(id: String, input: UpdateRuleInput, db: State<'_, Arc<Database>>) -> Result<Rule> {
    if let Some(ref name) = input.name {
        if let Some(ref content) = input.content {
            validate_rule_input(name, content)?;
        } else {
            let existing = db.get_rule_by_id(&id)?;
            validate_rule_input(name, &existing.content)?;
        }
    } else if let Some(ref content) = input.content {
        let existing = db.get_rule_by_id(&id)?;
        validate_rule_input(&existing.name, content)?;
    }

    // Validate scope change or local rule path requirement
    if let Some(scope) = input.scope {
        if matches!(scope, crate::models::Scope::Local) {
            if let Some(ref p) = input.target_paths {
                if p.is_empty() {
                    return Err(AppError::InvalidInput {
                        message: "Local rules must have at least one target path".to_string(),
                    });
                }
            } else {
                // If we're changing scope to Local, we MUST provide target_paths
                let existing = db.get_rule_by_id(&id)?;
                if existing.target_paths.as_ref().map(|p| p.is_empty()).unwrap_or(true) {
                    return Err(AppError::InvalidInput {
                        message: "Local rules must have at least one target path".to_string(),
                    });
                }
            }
        }
    } else {
        // If scope is not changing, but we're updating target_paths for an existing local rule
        let existing = db.get_rule_by_id(&id)?;
        if matches!(existing.scope, crate::models::Scope::Local) {
            if let Some(ref p) = input.target_paths {
                if p.is_empty() {
                    return Err(AppError::InvalidInput {
                        message: "Local rules must have at least one target path".to_string(),
                    });
                }
            }
        }
    }

    let updated = db.update_rule(&id, input)?;

    if use_file_storage(&db) {
        let location = storage_location_for_rule(&updated);
        file_storage::save_rule_to_disk(&updated, &location)?;
        db.update_rule_file_index(&updated.id, &location)?;
        register_local_rule_paths(&db, &updated)?;
    }

    Ok(updated)
}

#[tauri::command]
pub fn delete_rule(id: String, db: State<'_, Arc<Database>>) -> Result<()> {
    if use_file_storage(&db) {
        if let Ok(existing) = db.get_rule_by_id(&id) {
            let location = storage_location_for_rule(&existing);
            file_storage::delete_rule_file(&id, &location, Some(&db))?;
            db.remove_rule_file_index(&id)?;
        }
    }
    db.delete_rule(&id)
}

#[tauri::command]
pub fn toggle_rule(id: String, enabled: bool, db: State<'_, Arc<Database>>) -> Result<Rule> {
    let toggled = db.toggle_rule(&id, enabled)?;

    if use_file_storage(&db) {
        let location = storage_location_for_rule(&toggled);
        file_storage::save_rule_to_disk(&toggled, &location)?;
        db.update_rule_file_index(&toggled.id, &location)?;
        register_local_rule_paths(&db, &toggled)?;
    }

    Ok(toggled)
}

#[tauri::command]
pub fn migrate_to_file_storage(db: State<'_, Arc<Database>>) -> Result<file_storage::MigrationResult> {
    let result = file_storage::migrate_to_file_storage(&db)?;
    if result.success {
        db.set_storage_mode("file")?;
        if let Some(path) = &result.backup_path {
            db.set_setting("file_storage_backup_path", path)?;
        }
    }
    Ok(result)
}

#[tauri::command]
pub fn rollback_file_migration(backup_path: String, db: State<'_, Arc<Database>>) -> Result<()> {
    file_storage::rollback_migration(&backup_path)?;
    db.set_storage_mode("sqlite")?;
    db.set_setting("file_storage_backup_path", "")
}

#[tauri::command]
pub fn verify_file_migration(db: State<'_, Arc<Database>>) -> Result<file_storage::VerificationResult> {
    file_storage::verify_migration(&db)
}

#[tauri::command]
pub fn get_file_migration_progress() -> file_storage::MigrationProgress {
    file_storage::get_migration_progress()
}

#[tauri::command]
pub fn get_storage_info() -> Result<HashMap<String, String>> {
    let info = file_storage::get_storage_info()?;
    let mut out = HashMap::new();
    out.insert(
        "global_dir".to_string(),
        info.global_dir.to_string_lossy().to_string(),
    );
    out.insert("exists".to_string(), info.exists.to_string());
    out.insert("rule_count".to_string(), info.rule_count.to_string());
    out.insert(
        "total_size_bytes".to_string(),
        info.total_size_bytes.to_string(),
    );
    Ok(out)
}

#[tauri::command]
pub fn get_storage_mode(db: State<'_, Arc<Database>>) -> Result<String> {
    db.get_storage_mode()
}

#[tauri::command]
pub fn get_all_commands(db: State<'_, Arc<Database>>) -> Result<Vec<Command>> {
    db.get_all_commands()
}

#[tauri::command]
pub fn get_command_by_id(id: String, db: State<'_, Arc<Database>>) -> Result<Command> {
    db.get_command_by_id(&id)
}

#[tauri::command]
pub fn create_command(
    input: CreateCommandInput,
    db: State<'_, Arc<Database>>,
    mcp: State<'_, McpManager>,
) -> Result<Command> {
    validate_command_input(&input.name, &input.script)?;
    let created = db.create_command(input)?;
    let _ = mcp.refresh_commands(&db);
    Ok(created)
}

#[tauri::command]
pub fn update_command(
    id: String,
    input: UpdateCommandInput,
    db: State<'_, Arc<Database>>,
    mcp: State<'_, McpManager>,
) -> Result<Command> {
    if let Some(name) = &input.name {
        if let Some(script) = &input.script {
            validate_command_input(name, script)?;
        } else {
            let existing = db.get_command_by_id(&id)?;
            validate_command_input(name, &existing.script)?;
        }
    } else if let Some(script) = &input.script {
        let existing = db.get_command_by_id(&id)?;
        validate_command_input(&existing.name, script)?;
    }

    let updated = db.update_command(&id, input)?;
    let _ = mcp.refresh_commands(&db);
    Ok(updated)
}

#[tauri::command]
pub fn delete_command(
    id: String,
    db: State<'_, Arc<Database>>,
    mcp: State<'_, McpManager>,
) -> Result<()> {
    db.delete_command(&id)?;
    let _ = mcp.refresh_commands(&db);
    Ok(())
}

#[tauri::command]
pub async fn test_command(
    id: String,
    args: HashMap<String, String>,
    db: State<'_, Arc<Database>>,
) -> Result<TestCommandResult> {
    let cmd = db.get_command_by_id(&id)?;
    let mut script = cmd.script.clone();
    let mut envs: Vec<(String, String)> = Vec::new();

    for arg in &cmd.arguments {
        script = replace_template_with_env_ref(&script, &arg.name);

        let raw_value = args
            .get(&arg.name)
            .cloned()
            .or_else(|| arg.default_value.clone())
            .unwrap_or_default();
        let safe_value = sanitize_argument_value(&raw_value)?;
        envs.push((argument_env_var_name(&arg.name), safe_value));
    }

    let start = std::time::Instant::now();
    let (exit_code, stdout, stderr) =
        execute_shell_with_timeout_env(&script, Duration::from_secs(30), &envs).await?;
    let duration_ms = start.elapsed().as_millis() as u64;
    let success = exit_code == 0;

    let args_json = serde_json::to_string(&args)?;
    let _ = db.add_execution_log(&ExecutionLogInput {
        command_id: &cmd.id,
        command_name: &cmd.name,
        arguments_json: &args_json,
        stdout: &stdout,
        stderr: &stderr,
        exit_code,
        duration_ms,
        triggered_by: "test",
    });

    Ok(TestCommandResult {
        success,
        stdout,
        stderr,
        exit_code,
        duration_ms,
    })
}

#[tauri::command]
pub fn sync_commands(db: State<'_, Arc<Database>>) -> Result<SyncResult> {
    let commands = db.get_all_commands()?;
    let mut files_written = Vec::new();
    let mut errors = Vec::new();

    for (path_str, adapter_name) in command_file_targets()? {
        let path = PathBuf::from(&path_str);
        if let Some(parent) = path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                errors.push(SyncError {
                    file_path: path_str.clone(),
                    adapter_name: adapter_name.clone(),
                    message: e.to_string(),
                });
                continue;
            }
        }

        let content = if adapter_name == "gemini" {
            format_commands_toml(&commands)
        } else if adapter_name == "opencode" {
            format_commands_markdown(&commands, "RuleWeaver Commands")
        } else {
            format_commands_markdown(&commands, "RuleWeaver Commands (Claude Code)")
        };

        match fs::write(&path, content) {
            Ok(()) => files_written.push(path_str),
            Err(e) => errors.push(SyncError {
                file_path: path.to_string_lossy().to_string(),
                adapter_name,
                message: e.to_string(),
            }),
        }
    }

    Ok(SyncResult {
        success: errors.is_empty(),
        files_written,
        errors,
        conflicts: Vec::new(),
    })
}

#[tauri::command]
pub fn get_all_skills(db: State<'_, Arc<Database>>) -> Result<Vec<Skill>> {
    db.get_all_skills()
}

#[tauri::command]
pub fn get_skill_by_id(id: String, db: State<'_, Arc<Database>>) -> Result<Skill> {
    db.get_skill_by_id(&id)
}

#[tauri::command]
pub fn create_skill(input: CreateSkillInput, db: State<'_, Arc<Database>>) -> Result<Skill> {
    validate_skill_input(&input.name, &input.instructions)?;
    db.create_skill(input)
}

#[tauri::command]
pub fn update_skill(id: String, input: UpdateSkillInput, db: State<'_, Arc<Database>>) -> Result<Skill> {
    if let Some(ref name) = input.name {
        if let Some(ref instructions) = input.instructions {
            validate_skill_input(name, instructions)?;
        } else {
            let existing = db.get_skill_by_id(&id)?;
            validate_skill_input(name, &existing.instructions)?;
        }
    } else if let Some(ref instructions) = input.instructions {
        let existing = db.get_skill_by_id(&id)?;
        validate_skill_input(&existing.name, instructions)?;
    }

    db.update_skill(&id, input)
}

#[tauri::command]
pub fn delete_skill(id: String, db: State<'_, Arc<Database>>) -> Result<()> {
    db.delete_skill(&id)
}

#[tauri::command]
pub fn get_mcp_status(mcp: State<'_, McpManager>) -> Result<McpStatus> {
    mcp.status()
}

#[tauri::command]
pub fn start_mcp_server(db: State<'_, Arc<Database>>, mcp: State<'_, McpManager>) -> Result<()> {
    mcp.start(&db)
}

#[tauri::command]
pub fn stop_mcp_server(mcp: State<'_, McpManager>) -> Result<()> {
    mcp.stop()
}

#[tauri::command]
pub fn restart_mcp_server(db: State<'_, Arc<Database>>, mcp: State<'_, McpManager>) -> Result<()> {
    mcp.stop()?;
    mcp.start(&db)
}

#[tauri::command]
pub fn get_mcp_connection_instructions(
    mcp: State<'_, McpManager>,
) -> Result<McpConnectionInstructions> {
    mcp.instructions()
}

#[tauri::command]
pub fn get_mcp_logs(limit: Option<u32>, mcp: State<'_, McpManager>) -> Result<Vec<String>> {
    mcp.logs(limit.unwrap_or(50) as usize)
}

#[tauri::command]
pub fn get_execution_history(
    limit: Option<u32>,
    db: State<'_, Arc<Database>>,
) -> Result<Vec<ExecutionLog>> {
    db.get_execution_history(limit.unwrap_or(100))
}

#[tauri::command]
pub fn sync_rules(db: State<'_, Arc<Database>>) -> Result<SyncResult> {
    let rules = db.get_all_rules()?;
    let engine = SyncEngine::new(&db);
    Ok(engine.sync_all(rules))
}

#[tauri::command]
pub fn preview_sync(db: State<'_, Arc<Database>>) -> Result<SyncResult> {
    let rules = db.get_all_rules()?;
    let engine = SyncEngine::new(&db);
    Ok(engine.preview(rules))
}

#[tauri::command]
pub fn get_sync_history(
    limit: Option<u32>,
    db: State<'_, Arc<Database>>,
) -> Result<Vec<SyncHistoryEntry>> {
    db.get_sync_history(limit.unwrap_or(50))
}

#[tauri::command]
pub fn read_file_content(path: String) -> Result<String> {
    let validated_path = validate_path(&path)?;
    let content = fs::read_to_string(validated_path)?;
    Ok(content)
}

#[tauri::command]
pub fn resolve_conflict(
    file_path: String,
    resolution: String,
    db: State<'_, Arc<Database>>,
) -> Result<()> {
    match resolution.as_str() {
        "overwrite" => {
            let rules = db.get_all_rules()?;
            let engine = SyncEngine::new(&db);
            engine.sync_file_by_path(&rules, &file_path)?;
        }
        "keep-remote" => {
            let validated_path = validate_path(&file_path)?;
            let content = fs::read_to_string(validated_path)?;
            let hash = crate::sync::compute_content_hash_public(&content);
            db.set_file_hash(&file_path, &hash)?;
        }
        _ => {
            return Err(crate::error::AppError::InvalidInput {
                message: format!("Unknown resolution: {}", resolution),
            });
        }
    }
    Ok(())
}

#[tauri::command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
pub fn get_setting(key: String, db: State<'_, Arc<Database>>) -> Result<Option<String>> {
    db.get_setting(&key)
}

#[tauri::command]
pub fn set_setting(key: String, value: String, db: State<'_, Arc<Database>>) -> Result<()> {
    db.set_setting(&key, &value)
}

#[tauri::command]
pub fn get_all_settings(db: State<'_, Arc<Database>>) -> Result<HashMap<String, String>> {
    db.get_all_settings()
}

#[tauri::command]
pub fn get_app_data_path_cmd(app: tauri::AppHandle) -> Result<String> {
    let path = get_app_data_path(&app)?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn open_in_explorer(path: String) -> Result<()> {
    let validated_path = validate_path(&path)?;

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .args(["/select,", &validated_path.to_string_lossy()])
            .spawn()
            .map_err(crate::error::AppError::Io)?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .args(["-R", &validated_path.to_string_lossy()])
            .spawn()
            .map_err(crate::error::AppError::Io)?;
    }

    #[cfg(target_os = "linux")]
    {
        let parent_dir = validated_path.parent().unwrap_or(std::path::Path::new("/"));
        std::process::Command::new("xdg-open")
            .arg(parent_dir)
            .spawn()
            .map_err(crate::error::AppError::Io)?;
    }

    Ok(())
}
