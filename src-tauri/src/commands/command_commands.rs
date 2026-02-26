use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;

use crate::commands::{RUNNING_TESTS, TEST_INVOCATION_TIMESTAMPS};
use crate::constants::limits::TEST_CMD_RATE_LIMIT_MAX;
use crate::constants::timing::{TEST_CMD_RATE_LIMIT_WINDOW, TEST_CMD_TIMEOUT};
use crate::database::Database;
use crate::error::{AppError, Result};
use crate::execution::{
    argument_env_var_name, execute_and_log, replace_template_with_env_ref, sanitize_argument_value,
    validate_enum_argument, ExecuteAndLogInput,
};
use crate::mcp::McpManager;
use crate::models::{
    Command, CreateCommandInput, SyncError, SyncResult, TestCommandResult, UpdateCommandInput,
};
use crate::slash_commands::SlashCommandSyncEngine;

use crate::templates::commands::{get_bundled_command_templates, TemplateCommand};
use std::time::Instant;

use super::{
    command_file_targets, command_file_targets_for_root, reconcile_after_mutation,
    register_local_paths, validate_command_arguments, validate_command_input, validate_path,
    validate_paths_within_registered_roots,
};

#[tauri::command]
pub async fn get_all_commands(db: State<'_, Arc<Database>>) -> Result<Vec<Command>> {
    db.get_all_commands().await
}

#[tauri::command]
pub async fn get_command_by_id(id: String, db: State<'_, Arc<Database>>) -> Result<Command> {
    db.get_command_by_id(&id).await
}

#[tauri::command]
pub async fn create_command(
    input: CreateCommandInput,
    db: State<'_, Arc<Database>>,
    mcp: State<'_, McpManager>,
) -> Result<Command> {
    validate_command_input(&input.name, &input.script)?;
    validate_command_arguments(&input.arguments)?;
    for path in &input.target_paths {
        validate_path(path)?;
    }
    validate_paths_within_registered_roots(&db, &input.target_paths).await?;
    let created = db.create_command(input).await?;
    register_local_paths(&db, &created.target_paths).await?;
    mcp.refresh_commands(&db).await?;

    // Autosync slash commands on save when the command opts in to slash generation.
    if created.generate_slash_commands && !created.slash_command_adapters.is_empty() {
        let engine = SlashCommandSyncEngine::new(Arc::clone(&db));
        // Sync global and local (per target_paths) slash files; errors are non-fatal.
        let _ = engine.sync_command(&created, true);
        if !created.target_paths.is_empty() {
            let _ = engine.sync_command(&created, false);
        }
    }

    Ok(created)
}

#[tauri::command]
pub async fn update_command(
    id: String,
    input: UpdateCommandInput,
    db: State<'_, Arc<Database>>,
    mcp: State<'_, McpManager>,
) -> Result<Command> {
    // Capture the pre-update state before making any changes so we can detect
    // renames and adapter deselections that would leave orphan slash files.
    let existing = db.get_command_by_id(&id).await?;

    if let Some(name) = &input.name {
        if let Some(script) = &input.script {
            validate_command_input(name, script)?;
        } else {
            validate_command_input(name, &existing.script)?;
        }
    } else if let Some(script) = &input.script {
        validate_command_input(&existing.name, script)?;
    }

    if let Some(args) = &input.arguments {
        validate_command_arguments(args)?;
    }

    if let Some(paths) = &input.target_paths {
        for path in paths {
            validate_path(path)?;
        }
        validate_paths_within_registered_roots(&db, paths).await?;
    }

    let updated = db.update_command(&id, input).await?;
    register_local_paths(&db, &updated.target_paths).await?;
    mcp.refresh_commands(&db).await?;

    let engine = SlashCommandSyncEngine::new(Arc::clone(&db));

    // Orphan prevention: if the command was renamed, remove stale slash files for
    // the old name across all adapters and all target paths.
    let name_changed = existing.name != updated.name;
    if name_changed && !existing.slash_command_adapters.is_empty() {
        let _ = engine.remove_command(
            &existing.name,
            &existing.slash_command_adapters,
            &existing.target_paths,
        );
    }

    // Orphan prevention: if adapters were deselected, remove files only for
    // the adapters that are no longer in the updated list.
    let deselected: Vec<String> = existing
        .slash_command_adapters
        .iter()
        .filter(|a| !updated.slash_command_adapters.contains(a))
        .cloned()
        .collect();
    if !deselected.is_empty() {
        let _ = engine.remove_command(&existing.name, &deselected, &existing.target_paths);
    }

    // Autosync slash commands on save when the command opts in to slash generation.
    if updated.generate_slash_commands && !updated.slash_command_adapters.is_empty() {
        let _ = engine.sync_command(&updated, true);
        if !updated.target_paths.is_empty() {
            let _ = engine.sync_command(&updated, false);
        }
    }

    Ok(updated)
}

#[tauri::command]
pub async fn delete_command(
    id: String,
    db: State<'_, Arc<Database>>,
    mcp: State<'_, McpManager>,
) -> Result<()> {
    // Read before deleting so we have adapter and target_path info for cleanup.
    let command = db.get_command_by_id(&id).await?;

    db.delete_command(&id).await?;
    mcp.refresh_commands(&db).await?;

    // Remove slash command files for this command across all adapters and repo roots.
    if !command.slash_command_adapters.is_empty() {
        let engine = SlashCommandSyncEngine::new(Arc::clone(&db));
        let _ = engine.remove_command(
            &command.name,
            &command.slash_command_adapters,
            &command.target_paths,
        );
    }

    // Run reconciliation to clean up any remaining orphaned artifacts.
    reconcile_after_mutation(db.inner().clone()).await;

    Ok(())
}

#[tauri::command]
pub async fn test_command(
    id: String,
    args: HashMap<String, String>,
    db: State<'_, Arc<Database>>,
) -> Result<TestCommandResult> {
    // 1. Global Rate Limiting
    {
        let mut timestamps = TEST_INVOCATION_TIMESTAMPS.lock();
        let now = Instant::now();
        let cutoff = now - TEST_CMD_RATE_LIMIT_WINDOW;

        while let Some(t) = timestamps.front() {
            if *t < cutoff {
                timestamps.pop_front();
            } else {
                break;
            }
        }

        if timestamps.len() >= TEST_CMD_RATE_LIMIT_MAX {
            return Err(AppError::InvalidInput {
                message: format!(
                    "Rate limit exceeded. Max {} tests per minute. Please try again later.",
                    TEST_CMD_RATE_LIMIT_MAX
                ),
            });
        }
        timestamps.push_back(now);
    }

    // 2. Concurrency control: prevent multiple simultaneous tests of the same command
    {
        let mut running = RUNNING_TESTS.lock();
        if running.contains(&id) {
            return Err(AppError::InvalidInput {
                message:
                    "A test is already running for this command. Please wait for it to complete."
                        .to_string(),
            });
        }
        running.insert(id.clone());
    }

    let result = test_command_internal(&id, args, &db).await;

    // Clean up regardless of success or failure
    {
        let mut running = RUNNING_TESTS.lock();
        running.remove(&id);
    }

    result
}

async fn test_command_internal(
    id: &str,
    args: HashMap<String, String>,
    db: &State<'_, Arc<Database>>,
) -> Result<TestCommandResult> {
    let cmd = db.get_command_by_id(id).await?;
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

        // Enum validation (shared helper)
        if matches!(arg.arg_type, crate::models::ArgumentType::Enum) {
            validate_enum_argument(&arg.name, &raw_value, &arg.options)?;
        }

        envs.push((argument_env_var_name(&arg.name), safe_value));
    }

    let args_json = serde_json::to_string(&args).map_err(AppError::Serialization)?;

    let (exit_code, stdout, stderr, duration_ms) = execute_and_log(ExecuteAndLogInput {
        db: Some(db),
        command_id: &cmd.id,
        command_name: &cmd.name,
        script: &script,
        timeout_dur: TEST_CMD_TIMEOUT,
        envs: &envs,
        arguments_json: &args_json,
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
pub async fn sync_commands(db: State<'_, Arc<Database>>) -> Result<SyncResult> {
    // ... existing sync code ...
    let commands = db.get_all_commands().await?;
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

    let local_roots: std::collections::HashSet<_> = commands
        .iter()
        .flat_map(|command| &command.target_paths)
        .cloned()
        .collect();

    for local_root in local_roots {
        let local_commands = commands
            .iter()
            .filter(|c| c.target_paths.iter().any(|p| p == &local_root))
            .cloned()
            .collect::<Vec<_>>();

        if local_commands.is_empty() {
            continue;
        }

        for (path_str, adapter) in command_file_targets_for_root(&PathBuf::from(&local_root)) {
            let path = PathBuf::from(&path_str);
            let content = adapter.format(&local_commands);
            pending_writes.push((path, format!("{} (local)", adapter.name()), content));
        }
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

#[tauri::command]
pub fn get_command_templates() -> Result<Vec<TemplateCommand>> {
    Ok(get_bundled_command_templates())
}

#[tauri::command]
pub async fn install_command_template(
    template_id: String,
    db: State<'_, Arc<Database>>,
    mcp: State<'_, McpManager>,
) -> Result<Command> {
    // 1. Check idempotency: is it already installed?
    if let Ok(existing) = db.get_command_by_id(&template_id).await {
        return Ok(existing);
    }

    // 2. Find template
    let templates = get_bundled_command_templates();
    let template = templates
        .into_iter()
        .find(|t| t.template_id == template_id)
        .ok_or_else(|| AppError::Validation(format!("Template '{}' not found", template_id)))?;

    // 3. Check for name collisions
    if db.command_exists_with_name(&template.metadata.name).await? {
        return Err(AppError::Validation(format!(
            "A command with the name '{}' already exists. Please rename or delete it before installing this template.",
            template.metadata.name
        )));
    }

    let mut input = template.metadata.clone();
    input.id = Some(template_id.clone());

    // 4. Create in DB
    let created = db.create_command(input).await?;

    // 5. Registration
    // If registration fails, we must rollback the DB entry
    if let Err(e) = register_local_paths(&db, &created.target_paths).await {
        let _ = db.delete_command(&created.id).await;
        return Err(e);
    }

    if let Err(e) = mcp.refresh_commands(&db).await {
        let _ = db.delete_command(&created.id).await;
        return Err(e);
    }

    Ok(created)
}
