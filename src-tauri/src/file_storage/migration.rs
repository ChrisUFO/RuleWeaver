use sha2::Digest;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Mutex, OnceLock};

use crate::database::Database;
use crate::error::{AppError, Result};
use crate::file_storage::{save_rule_to_disk, StorageLocation};

static MIGRATION_PROGRESS: AtomicU32 = AtomicU32::new(0);
static MIGRATION_TOTAL: AtomicU32 = AtomicU32::new(0);
static MIGRATION_STATE: OnceLock<Mutex<MigrationState>> = OnceLock::new();

#[derive(Debug, Clone)]
struct MigrationState {
    current_rule: Option<String>,
    status: MigrationStatus,
}

fn migration_state() -> &'static Mutex<MigrationState> {
    MIGRATION_STATE.get_or_init(|| {
        Mutex::new(MigrationState {
            current_rule: None,
            status: MigrationStatus::NotStarted,
        })
    })
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationProgress {
    pub total: u32,
    pub migrated: u32,
    pub current_rule: Option<String>,
    pub status: MigrationStatus,
}

#[derive(Debug, Clone, serde::Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum MigrationStatus {
    NotStarted,
    InProgress,
    Completed,
    Failed,
    RolledBack,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationResult {
    pub success: bool,
    pub rules_migrated: u32,
    pub rules_skipped: u32,
    pub errors: Vec<MigrationError>,
    pub backup_path: Option<String>,
    pub storage_dir: String,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationError {
    pub rule_id: String,
    pub rule_name: String,
    pub error: String,
}

pub async fn migrate_to_file_storage(db: &Database) -> Result<MigrationResult> {
    MIGRATION_PROGRESS.store(0, Ordering::Relaxed);
    MIGRATION_TOTAL.store(0, Ordering::Relaxed);
    if let Ok(mut state) = migration_state().lock() {
        state.current_rule = None;
        state.status = MigrationStatus::InProgress;
    }

    let rules = db.get_all_rules().await?;
    let total = rules.len() as u32;
    MIGRATION_TOTAL.store(total, Ordering::Relaxed);

    if total == 0 {
        if let Ok(mut state) = migration_state().lock() {
            state.status = MigrationStatus::Completed;
        }
        return Ok(MigrationResult {
            success: true,
            rules_migrated: 0,
            rules_skipped: 0,
            errors: Vec::new(),
            backup_path: None,
            storage_dir: crate::file_storage::get_global_rules_dir()?
                .to_string_lossy()
                .to_string(),
        });
    }

    let backup_path = create_backup(db).await?;

    let storage_dir = crate::file_storage::get_global_rules_dir()?;
    fs::create_dir_all(&storage_dir)?;

    let mut rules_migrated = 0u32;
    let mut rules_skipped = 0u32;
    let mut errors = Vec::new();
    let mut local_rule_paths: Vec<String> = Vec::new();

    for rule in &rules {
        if let Ok(mut state) = migration_state().lock() {
            state.current_rule = Some(rule.name.clone());
        }
        MIGRATION_PROGRESS.fetch_add(1, Ordering::Relaxed);

        let location = match rule.scope {
            crate::models::Scope::Global => StorageLocation::Global,
            crate::models::Scope::Local => {
                if let Some(ref paths) = rule.target_paths {
                    if let Some(first_path) = paths.first() {
                        StorageLocation::Local(PathBuf::from(first_path))
                    } else {
                        StorageLocation::Global
                    }
                } else {
                    StorageLocation::Global
                }
            }
        };

        match save_rule_to_disk(rule, &location) {
            Ok(_) => {
                rules_migrated += 1;
                if let Some(paths) = &rule.target_paths {
                    for path in paths {
                        if !local_rule_paths.iter().any(|p| p == path) {
                            local_rule_paths.push(path.clone());
                        }
                    }
                }
                if let Err(e) = db.update_rule_file_index(&rule.id, &location).await {
                    errors.push(MigrationError {
                        rule_id: rule.id.clone(),
                        rule_name: rule.name.clone(),
                        error: format!("Failed to update index: {}", e),
                    });
                }
            }
            Err(e) => {
                rules_skipped += 1;
                errors.push(MigrationError {
                    rule_id: rule.id.clone(),
                    rule_name: rule.name.clone(),
                    error: e.to_string(),
                });
            }
        }
    }

    let success = errors.is_empty();
    if success {
        let local_paths_json = serde_json::to_string(&local_rule_paths)?;
        let _ = db.set_setting("local_rule_paths", &local_paths_json).await;
    }
    if let Ok(mut state) = migration_state().lock() {
        state.current_rule = None;
        state.status = if success {
            MigrationStatus::Completed
        } else {
            MigrationStatus::Failed
        };
    }

    Ok(MigrationResult {
        success,
        rules_migrated,
        rules_skipped,
        errors,
        backup_path: Some(backup_path),
        storage_dir: storage_dir.to_string_lossy().to_string(),
    })
}

async fn create_backup(db: &Database) -> Result<String> {
    let db_path = db.get_database_path().await?;
    let now = chrono::Local::now().format("%Y%m%d%H%M%S");
    let backup_path = format!("{}.{}.migration-backup", db_path, now);

    fs::copy(&db_path, &backup_path).map_err(|e| AppError::InvalidInput {
        message: format!("Failed to create database backup: {}", e),
    })?;

    // Create checksum
    let content = fs::read(&backup_path)?;
    let mut hasher = sha2::Sha256::new();
    sha2::Digest::update(&mut hasher, &content);
    let checksum = format!("{:x}", hasher.finalize());
    fs::write(format!("{}.checksum", backup_path), checksum)?;

    Ok(backup_path)
}

pub async fn rollback_migration(backup_path: &str, db: Option<&Database>) -> Result<()> {
    // backup_path: /path/to/db.timestamp.migration-backup
    let db_path_buf = if let Some(d) = db {
        PathBuf::from(d.get_database_path().await?)
    } else {
        let path = if let Some(stripped) = backup_path.strip_suffix(".migration-backup") {
            if let Some(last_dot_idx) = stripped.rfind('.') {
                &stripped[..last_dot_idx]
            } else {
                stripped
            }
        } else {
            backup_path
        };
        PathBuf::from(path)
    };

    if !PathBuf::from(backup_path).exists() {
        return Err(AppError::InvalidInput {
            message: "Backup file not found".to_string(),
        });
    }

    // Verify checksum
    let checksum_path = format!("{}.checksum", backup_path);
    if PathBuf::from(&checksum_path).exists() {
        let stored_checksum = fs::read_to_string(&checksum_path)?;
        let content = fs::read(backup_path)?;
        let mut hasher = sha2::Sha256::new();
        sha2::Digest::update(&mut hasher, &content);
        let current_checksum = format!("{:x}", hasher.finalize());

        if stored_checksum.trim() != current_checksum {
            return Err(AppError::InvalidInput {
                message: "Backup integrity check failed: checksum mismatch".to_string(),
            });
        }
    } else {
        return Err(AppError::InvalidInput {
            message: "Backup checksum file missing. Restoration aborted for safety.".to_string(),
        });
    }

    fs::copy(backup_path, &db_path_buf).map_err(|e| AppError::InvalidInput {
        message: format!("Failed to restore database from backup: {}", e),
    })?;

    fs::remove_file(backup_path).ok();

    let storage_dir = crate::file_storage::get_global_rules_dir()?;
    if storage_dir.exists() {
        // Only remove if it's empty to avoid deleting new rules
        if let Ok(entries) = fs::read_dir(&storage_dir) {
            if entries.count() == 0 {
                fs::remove_dir(&storage_dir).ok();
            }
        }
    }

    if let Ok(mut state) = migration_state().lock() {
        state.current_rule = None;
        state.status = MigrationStatus::RolledBack;
    }

    Ok(())
}

pub fn get_migration_progress() -> MigrationProgress {
    let migrated = MIGRATION_PROGRESS.load(Ordering::Relaxed);
    let total = MIGRATION_TOTAL.load(Ordering::Relaxed);
    let (current_rule, status) = migration_state()
        .lock()
        .map(|s| (s.current_rule.clone(), s.status.clone()))
        .unwrap_or((None, MigrationStatus::NotStarted));

    MigrationProgress {
        total,
        migrated,
        current_rule,
        status,
    }
}

pub async fn verify_migration(db: &Database) -> Result<VerificationResult> {
    let db_rules = db.get_all_rules().await?;

    let load_result = crate::file_storage::load_rules_from_disk()?;

    let mut missing_rules = Vec::new();
    let mut extra_rules = Vec::new();
    let mut mismatched_rules = Vec::new();

    for db_rule in &db_rules {
        let found = load_result.rules.iter().find(|r| r.id == db_rule.id);
        if found.is_none() {
            missing_rules.push(db_rule.id.clone());
        } else if let Some(file_rule) = found {
            if file_rule.name != db_rule.name || file_rule.content != db_rule.content {
                mismatched_rules.push(db_rule.id.clone());
            }
        }
    }

    for file_rule in &load_result.rules {
        let found = db_rules.iter().find(|r| r.id == file_rule.id);
        if found.is_none() {
            extra_rules.push(file_rule.id.clone());
        }
    }

    Ok(VerificationResult {
        is_valid: missing_rules.is_empty() && mismatched_rules.is_empty(),
        db_rule_count: db_rules.len() as u32,
        file_rule_count: load_result.rules.len() as u32,
        missing_rules,
        extra_rules,
        mismatched_rules,
        load_errors: load_result.errors.len() as u32,
    })
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct VerificationResult {
    pub is_valid: bool,
    pub db_rule_count: u32,
    pub file_rule_count: u32,
    pub missing_rules: Vec<String>,
    pub extra_rules: Vec<String>,
    pub mismatched_rules: Vec<String>,
    pub load_errors: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_status_default() {
        let progress = get_migration_progress();
        assert_eq!(progress.migrated, 0);
        assert_eq!(progress.status, MigrationStatus::NotStarted);
    }
}
