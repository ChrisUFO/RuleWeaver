use crate::database::{get_app_data_path, Database};
use crate::error::Result;
use crate::models::{CreateRuleInput, Rule, SyncHistoryEntry, SyncResult, UpdateRuleInput};
use crate::sync::SyncEngine;
use std::collections::HashMap;
use std::fs;
use tauri::State;

#[tauri::command]
pub fn get_all_rules(db: State<'_, Database>) -> Result<Vec<Rule>> {
    db.get_all_rules()
}

#[tauri::command]
pub fn get_rule_by_id(id: String, db: State<'_, Database>) -> Result<Rule> {
    db.get_rule_by_id(&id)
}

#[tauri::command]
pub fn create_rule(input: CreateRuleInput, db: State<'_, Database>) -> Result<Rule> {
    db.create_rule(input)
}

#[tauri::command]
pub fn update_rule(id: String, input: UpdateRuleInput, db: State<'_, Database>) -> Result<Rule> {
    db.update_rule(&id, input)
}

#[tauri::command]
pub fn delete_rule(id: String, db: State<'_, Database>) -> Result<()> {
    db.delete_rule(&id)
}

#[tauri::command]
pub fn toggle_rule(id: String, enabled: bool, db: State<'_, Database>) -> Result<Rule> {
    db.toggle_rule(&id, enabled)
}

#[tauri::command]
pub fn sync_rules(db: State<'_, Database>) -> Result<SyncResult> {
    let rules = db.get_all_rules()?;
    let engine = SyncEngine::new(&db);
    Ok(engine.sync_all(rules))
}

#[tauri::command]
pub fn preview_sync(db: State<'_, Database>) -> Result<SyncResult> {
    let rules = db.get_all_rules()?;
    let engine = SyncEngine::new(&db);
    Ok(engine.preview(rules))
}

#[tauri::command]
pub fn get_sync_history(
    limit: Option<u32>,
    db: State<'_, Database>,
) -> Result<Vec<SyncHistoryEntry>> {
    db.get_sync_history(limit.unwrap_or(50))
}

#[tauri::command]
pub fn read_file_content(path: String) -> Result<String> {
    let content = fs::read_to_string(&path)?;
    Ok(content)
}

#[tauri::command]
pub fn resolve_conflict(
    file_path: String,
    resolution: String,
    db: State<'_, Database>,
) -> Result<()> {
    match resolution.as_str() {
        "overwrite" => {
            let rules = db.get_all_rules()?;
            let engine = SyncEngine::new(&db);
            engine.sync_file_by_path(&rules, &file_path)?;
        }
        "keep-remote" => {
            let content = fs::read_to_string(&file_path)?;
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
pub fn get_setting(key: String, db: State<'_, Database>) -> Result<Option<String>> {
    db.get_setting(&key)
}

#[tauri::command]
pub fn set_setting(key: String, value: String, db: State<'_, Database>) -> Result<()> {
    db.set_setting(&key, &value)
}

#[tauri::command]
pub fn get_all_settings(db: State<'_, Database>) -> Result<HashMap<String, String>> {
    db.get_all_settings()
}

#[tauri::command]
pub fn get_app_data_path_cmd(app: tauri::AppHandle) -> Result<String> {
    let path = get_app_data_path(&app)?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn open_in_explorer(path: String) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .args(["/select,", &path])
            .spawn()
            .map_err(crate::error::AppError::Io)?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .args(["-R", &path])
            .spawn()
            .map_err(crate::error::AppError::Io)?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(crate::error::AppError::Io)?;
    }

    Ok(())
}
