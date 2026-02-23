use std::path::PathBuf;
use std::sync::Arc;

use tauri::State;

use crate::database::Database;
use crate::error::Result;
use crate::slash_commands::{
    get_all_adapters, SlashCommandSyncEngine, SlashCommandSyncResult, SyncStatus,
};

/// Sync slash commands for a specific command
#[tauri::command]
pub async fn sync_slash_command(
    command_id: String,
    is_global: bool,
    database: State<'_, Arc<Database>>,
) -> Result<SlashCommandSyncResult> {
    let engine = SlashCommandSyncEngine::new(Arc::clone(&database));
    
    // Get the command from database
    let command = database.get_command_by_id(&command_id)?;
    
    // Sync the command
    let result = engine.sync_command(&command, is_global)?;
    
    Ok(result)
}

/// Sync all commands that have slash commands enabled
#[tauri::command]
pub async fn sync_all_slash_commands(
    is_global: bool,
    database: State<'_, Arc<Database>>,
) -> Result<SlashCommandSyncResult> {
    let engine = SlashCommandSyncEngine::new(Arc::clone(&database));
    
    // Sync all commands
    let result = engine.sync_all_commands(is_global)?;
    
    Ok(result)
}

/// Get sync status for a command
#[tauri::command]
pub async fn get_slash_command_status(
    command_id: String,
    database: State<'_, Arc<Database>>,
) -> Result<std::collections::HashMap<String, SyncStatus>> {
    let engine = SlashCommandSyncEngine::new(Arc::clone(&database));
    
    // Get the command from database
    let command = database.get_command_by_id(&command_id)?;
    
    // Get sync status
    let status = engine.get_command_sync_status(&command)?;
    
    Ok(status)
}

/// Clean up slash command files for a given adapter
#[tauri::command]
pub async fn cleanup_slash_commands(
    adapter_name: String,
    is_global: bool,
    database: State<'_, Arc<Database>>,
) -> Result<usize> {
    let engine = SlashCommandSyncEngine::new(Arc::clone(&database));
    
    // Cleanup the adapter
    let count = engine.cleanup_adapter(&adapter_name, is_global)?;
    
    Ok(count)
}

/// Remove slash command files for a deleted command
#[tauri::command]
pub async fn remove_slash_command_files(
    command_name: String,
    adapters: Vec<String>,
    database: State<'_, Arc<Database>>,
) -> Result<SlashCommandSyncResult> {
    let engine = SlashCommandSyncEngine::new(Arc::clone(&database));
    
    // Remove command files
    let result = engine.remove_command(&command_name, &adapters)?;
    
    Ok(result)
}

/// Get all available slash command adapters
#[tauri::command]
pub async fn get_slash_command_adapters() -> Result<Vec<AdapterInfo>> {
    let adapters = get_all_adapters();
    
    let info: Vec<AdapterInfo> = adapters
        .iter()
        .map(|adapter| AdapterInfo {
            name: adapter.name().to_string(),
            supports_argument_substitution: adapter.supports_argument_substitution(),
            argument_pattern: adapter.argument_pattern().map(|s| s.to_string()),
            global_path: adapter.global_dir().to_string(),
            local_path: adapter.local_dir().to_string(),
        })
        .collect();
    
    Ok(info)
}

/// Info about a slash command adapter
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AdapterInfo {
    pub name: String,
    pub supports_argument_substitution: bool,
    pub argument_pattern: Option<String>,
    pub global_path: String,
    pub local_path: String,
}

/// Test slash command generation without writing files
#[tauri::command]
pub async fn test_slash_command_generation(
    adapter_name: String,
    command_id: String,
    database: State<'_, Arc<Database>>,
) -> Result<String> {
    use crate::slash_commands::get_adapter;
    
    // Get the command from database
    let command = database.get_command_by_id(&command_id)?;
    
    // Get the adapter
    let adapter = get_adapter(&adapter_name)
        .ok_or_else(|| crate::error::AppError::InvalidInput {
            message: format!("Unknown adapter: {}", adapter_name),
        })?;
    
    // Generate the content
    let content = adapter.format_command(&command);
    
    Ok(content)
}

/// Get the file path for a slash command
#[tauri::command]
pub async fn get_slash_command_path(
    adapter_name: String,
    command_name: String,
    is_global: bool,
) -> Result<PathBuf> {
    use crate::slash_commands::get_adapter;
    
    // Get the adapter
    let adapter = get_adapter(&adapter_name)
        .ok_or_else(|| crate::error::AppError::InvalidInput {
            message: format!("Unknown adapter: {}", adapter_name),
        })?;
    
    // Get the path
    let path = adapter.get_command_path(&command_name, is_global);
    
    Ok(path)
}
