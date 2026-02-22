use std::sync::Arc;
use tauri::State;

use crate::database::Database;
use crate::error::Result;
use crate::file_storage;
use crate::sync::SyncEngine;

use super::validate_path;

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
    file_storage::rollback_migration(&backup_path, Some(&db))?;
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
