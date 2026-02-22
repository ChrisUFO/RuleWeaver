use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;

use crate::constants::timing::TEST_CMD_TIMEOUT;
use crate::database::Database;
use crate::error::{AppError, Result};
use crate::execution::{
    argument_env_var_name, execute_and_log, replace_template_with_env_ref,
    sanitize_argument_value, ExecuteAndLogInput,
};
use crate::mcp::McpManager;
use crate::models::{
    Command, CreateCommandInput, SyncError, SyncResult, TestCommandResult, UpdateCommandInput,
};

use super::{
    command_file_targets, validate_command_arguments, validate_command_input, RUNNING_TESTS,
};

#[tauri::command]
pub fn get_all_commands(db: State<'_, Arc<Database>>) -> Result<Vec<Command>> {
    db.get_all_commands()
}

#[tauri::command]
pub fn get_command_by_id(id: String, db: State<'_, Arc<Database>>) -> Result<Command> {
    db.get_command_by_id(&id)
}

#[tauri::command]
pub async fn create_command(
    input: CreateCommandInput,
    db: State<'_, Arc<Database>>,
    mcp: State<'_, McpManager>,
) -> Result<Command> {
    validate_command_input(&input.name, &input.script)?;
    validate_command_arguments(&input.arguments)?;
    let created = db.create_command(input)?;
    mcp.refresh_commands(&db).await?;
    Ok(created)
}

#[tauri::command]
pub async fn update_command(
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

    if let Some(args) = &input.arguments {
        validate_command_arguments(args)?;
    }

    let updated = db.update_command(&id, input)?;
    mcp.refresh_commands(&db).await?;
    Ok(updated)
}

#[tauri::command]
pub async fn delete_command(
    id: String,
    db: State<'_, Arc<Database>>,
    mcp: State<'_, McpManager>,
) -> Result<()> {
    db.delete_command(&id)?;
    mcp.refresh_commands(&db).await
}

#[tauri::command]
pub async fn test_command(
    id: String,
    args: HashMap<String, String>,
    db: State<'_, Arc<Database>>,
) -> Result<TestCommandResult> {
    // Concurrency control: prevent multiple simultaneous tests of the same command
    {
        let mut running = RUNNING_TESTS.lock().map_err(|_| AppError::LockError)?;
        if running.contains(&id) {
            return Err(AppError::InvalidInput {
                message: "A test is already running for this command. Please wait for it to complete.".to_string(),
            });
        }
        running.insert(id.clone());
    }

    let result = test_command_internal(&id, args, &db).await;

    // Clean up regardless of success or failure
    {
        let mut running = RUNNING_TESTS.lock().map_err(|_| AppError::LockError)?;
        running.remove(&id);
    }

    result
}

async fn test_command_internal(
    id: &str,
    args: HashMap<String, String>,
    db: &State<'_, Arc<Database>>,
) -> Result<TestCommandResult> {
    let cmd = db.get_command_by_id(id)?;
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
        
        // Enum validation
        if matches!(arg.arg_type, crate::models::ArgumentType::Enum) {
            if let Some(ref options) = arg.options {
                if !options.contains(&raw_value) {
                    return Err(AppError::InvalidInput {
                        message: format!("Argument '{}' must be one of: {}", arg.name, options.join(", ")),
                    });
                }
            }
        }

        envs.push((argument_env_var_name(&arg.name), safe_value));
    }

    let (exit_code, stdout, stderr, duration_ms) = execute_and_log(ExecuteAndLogInput {
        db: Some(db),
        command_id: &cmd.id,
        command_name: &cmd.name,
        script: &script,
        timeout_dur: TEST_CMD_TIMEOUT,
        envs: &envs,
        arguments_json: &serde_json::to_string(&args).unwrap_or_default(),
        triggered_by: "test",
    })
    .await?;

    let success = exit_code == 0;

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

    // Two-phase commit:
    // 1. Prepare all contents and targets
    let mut pending_writes = Vec::new();
    for (path_str, adapter) in command_file_targets()? {
        let path = PathBuf::from(&path_str);
        let content = adapter.format(&commands);
        pending_writes.push((path, adapter.name().to_string(), content));
    }

    // 2. Execute all writes
    for (path, adapter_name, content) in pending_writes {
        if let Some(parent) = path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                errors.push(SyncError {
                    file_path: path.to_string_lossy().to_string(),
                    adapter_name: adapter_name.clone(),
                    message: format!("Failed to create directory: {}", e),
                });
                continue;
            }
        }

        let temp_path = path.with_extension(format!("tmp-{}", uuid::Uuid::new_v4()));
        let write_result = (|| -> std::io::Result<()> {
            {
                let mut file = fs::File::create(&temp_path)?;
                file.write_all(content.as_bytes())?;
                file.sync_all()?;
            }
            fs::rename(&temp_path, &path)?;
            
            if let Some(parent) = path.parent() {
                if let Ok(dir) = fs::File::open(parent) {
                    let _ = dir.sync_all();
                }
            }
            Ok(())
        })();

        match write_result {
            Ok(()) => files_written.push(path.to_string_lossy().to_string()),
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
