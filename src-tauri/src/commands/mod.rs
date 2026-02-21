use crate::database::{get_app_data_path, Database};
use crate::error::{AppError, Result};
use crate::models::{CreateRuleInput, Rule, SyncHistoryEntry, SyncResult, UpdateRuleInput};
use crate::sync::SyncEngine;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
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
    validate_rule_input(&input.name, &input.content)?;
    db.create_rule(input)
}

#[tauri::command]
pub fn update_rule(id: String, input: UpdateRuleInput, db: State<'_, Database>) -> Result<Rule> {
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
    let validated_path = validate_path(&path)?;
    let content = fs::read_to_string(validated_path)?;
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
    let validated_path =
        std::fs::canonicalize(&path).map_err(|e| crate::error::AppError::InvalidInput {
            message: format!("Invalid path: {}", e),
        })?;

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
