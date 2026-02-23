use std::sync::Arc;
use tauri::{Manager, State};

use crate::database::Database;
use crate::error::Result;
use crate::file_storage;
use crate::sync::SyncEngine;

use super::validate_path;

#[tauri::command]
pub fn migrate_to_file_storage(
    db: State<'_, Arc<Database>>,
) -> Result<file_storage::MigrationResult> {
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
    file_storage::rollback_migration(&backup_path, Some(&db))?;
    db.set_storage_mode("sqlite")?;
    db.set_setting("file_storage_backup_path", "")
}

#[tauri::command]
pub fn verify_file_migration(
    db: State<'_, Arc<Database>>,
) -> Result<file_storage::VerificationResult> {
    file_storage::verify_migration(&db)
}

#[tauri::command]
pub fn get_file_migration_progress() -> file_storage::MigrationProgress {
    file_storage::get_migration_progress()
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
            let content = std::fs::read_to_string(validated_path)?;
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
pub fn get_storage_info() -> Result<std::collections::HashMap<String, String>> {
    let info = file_storage::get_storage_info()?;
    let mut out = std::collections::HashMap::new();
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
pub async fn export_configuration(path: String, db: State<'_, Arc<Database>>) -> Result<()> {
    let rules = db.get_all_rules()?;
    let commands = db.get_all_commands()?;
    let skills = db.get_all_skills()?;

    let config = crate::models::ExportConfiguration::new(rules, commands, skills);

    let content = if path.ends_with(".yaml") || path.ends_with(".yml") {
        serde_yaml::to_string(&config).map_err(|e| crate::error::AppError::InvalidInput {
            message: e.to_string(),
        })?
    } else {
        serde_json::to_string_pretty(&config)?
    };

    tokio::task::spawn_blocking(move || {
        std::fs::write(path, content).map_err(crate::error::AppError::Io)
    })
    .await
    .map_err(|e| crate::error::AppError::InvalidInput {
        message: e.to_string(),
    })??;

    Ok(())
}

fn validate_config_version(config: &crate::models::ExportConfiguration) -> Result<()> {
    if config.version != "1.0" {
        return Err(crate::error::AppError::InvalidInput {
            message: format!(
                "Unsupported configuration version: {}. Only 1.0 is supported.",
                config.version
            ),
        });
    }
    Ok(())
}

fn validate_config_data(config: &crate::models::ExportConfiguration) -> Result<()> {
    for rule in &config.rules {
        if rule.name.trim().is_empty() {
            return Err(crate::error::AppError::Validation(
                "Imported rule name cannot be empty".to_string(),
            ));
        }
        if rule.content.trim().is_empty() {
            return Err(crate::error::AppError::Validation(format!(
                "Imported rule '{}' has no content",
                rule.name
            )));
        }
    }

    for cmd in &config.commands {
        if cmd.name.trim().is_empty() {
            return Err(crate::error::AppError::Validation(
                "Imported command name cannot be empty".to_string(),
            ));
        }
        if cmd.script.trim().is_empty() {
            return Err(crate::error::AppError::Validation(format!(
                "Imported command '{}' has no script",
                cmd.name
            )));
        }
    }

    for skill in &config.skills {
        if skill.name.trim().is_empty() {
            return Err(crate::error::AppError::Validation(
                "Imported skill name cannot be empty".to_string(),
            ));
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn preview_import(path: String) -> Result<crate::models::ExportConfiguration> {
    let path_clone = path.clone();
    let content = tokio::task::spawn_blocking(move || {
        std::fs::read_to_string(path_clone).map_err(crate::error::AppError::Io)
    })
    .await
    .map_err(|e| crate::error::AppError::InvalidInput {
        message: e.to_string(),
    })??;

    let config: crate::models::ExportConfiguration = if path.ends_with(".yaml") || path.ends_with(".yml")
    {
        serde_yaml::from_str(&content).map_err(|e| crate::error::AppError::InvalidInput {
            message: e.to_string(),
        })?
    } else {
        serde_json::from_str(&content)?
    };

    validate_config_version(&config)?;
    validate_config_data(&config)?;

    Ok(config)
}

#[tauri::command]
pub async fn import_configuration(
    path: String,
    mode: crate::models::ImportMode,
    db: State<'_, Arc<Database>>,
    _status: State<'_, crate::GlobalStatus>,
    app: tauri::AppHandle,
) -> Result<()> {
    let path_clone = path.clone();
    let content = tokio::task::spawn_blocking(move || {
        std::fs::read_to_string(path_clone).map_err(crate::error::AppError::Io)
    })
    .await
    .map_err(|e| crate::error::AppError::InvalidInput {
        message: e.to_string(),
    })??;

    let config: crate::models::ExportConfiguration =
        if path.ends_with(".yaml") || path.ends_with(".yml") {
            serde_yaml::from_str(&content).map_err(|e| crate::error::AppError::InvalidInput {
                message: e.to_string(),
            })?
        } else {
            serde_json::from_str(&content)?
        };

    validate_config_version(&config)?;
    validate_config_data(&config)?;

    let db_clone = Arc::clone(&db);

    // Perform DB operations in a blocking task to keep UI responsive
    tokio::task::spawn_blocking(move || -> Result<()> {
        for rule in config.rules {
            db_clone.import_rule(rule, mode)?;
        }

        for command in config.commands {
            db_clone.import_command(command, mode)?;
        }

        for skill in config.skills {
            db_clone.import_skill(skill, mode)?;
        }
        Ok(())
    })
    .await
    .map_err(|e| crate::error::AppError::InvalidInput {
        message: e.to_string(),
    })??;

    // Trigger sync after import
    {
        if let Some(s) = app.try_state::<crate::GlobalStatus>() {
            *s.sync_status.lock() = "Syncing...".to_string();
            s.update_tray();
        }
    }

    let engine = SyncEngine::new(&db);
    let rules = db.get_all_rules()?;
    let _ = engine.sync_all(rules);

    {
        if let Some(s) = app.try_state::<crate::GlobalStatus>() {
            *s.sync_status.lock() = "Idle".to_string();
            s.update_tray();
        }
    }

    Ok(())
}
