use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use tauri::State;

use crate::database::{get_app_data_path, Database};
use crate::error::Result;
use crate::models::{ExecutionLog, SyncHistoryEntry};

use super::validate_path;

#[tauri::command]
pub async fn get_execution_history(
    limit: Option<u32>,
    db: State<'_, Arc<Database>>,
) -> Result<Vec<ExecutionLog>> {
    db.get_execution_history(limit.unwrap_or(100)).await
}

#[tauri::command]
pub async fn get_sync_history(
    limit: Option<u32>,
    db: State<'_, Arc<Database>>,
) -> Result<Vec<SyncHistoryEntry>> {
    db.get_sync_history(limit.unwrap_or(50)).await
}

#[tauri::command]
pub async fn read_file_content(path: String) -> Result<String> {
    let validated_path = validate_path(&path)?;
    let content = tokio::task::spawn_blocking(move || {
        fs::read_to_string(validated_path).map_err(crate::error::AppError::Io)
    })
    .await
    .map_err(|e| crate::error::AppError::InvalidInput {
        message: e.to_string(),
    })??;
    Ok(content)
}

#[tauri::command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
pub async fn get_setting(key: String, db: State<'_, Arc<Database>>) -> Result<Option<String>> {
    db.get_setting(&key).await
}

#[tauri::command]
pub async fn set_setting(key: String, value: String, db: State<'_, Arc<Database>>) -> Result<()> {
    db.set_setting(&key, &value).await
}

#[tauri::command]
pub async fn get_all_settings(db: State<'_, Arc<Database>>) -> Result<HashMap<String, String>> {
    db.get_all_settings().await
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
