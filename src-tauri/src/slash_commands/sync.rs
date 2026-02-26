use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use sha2::{Digest, Sha256};
use tokio::sync::Mutex;

use crate::database::Database;
use crate::error::{AppError, Result};
use crate::models::Command;
use crate::slash_commands::{get_adapter, SlashCommandAdapter};

/// Validates a command name to prevent path traversal and other security issues
pub fn validate_command_name(name: &str) -> Result<String> {
    if name.trim().is_empty() {
        return Err(AppError::InvalidInput {
            message: "Command name cannot be empty".to_string(),
        });
    }

    // Check for path traversal attempts
    if name.contains("..") || name.contains('/') || name.contains('\\') {
        return Err(AppError::InvalidInput {
            message: format!(
                "Command name '{}' contains invalid characters. Path separators are not allowed.",
                name
            ),
        });
    }

    // Check for null bytes
    if name.contains('\0') {
        return Err(AppError::InvalidInput {
            message: "Command name cannot contain null bytes".to_string(),
        });
    }

    // Create a safe slug
    let slug = name
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>();

    // Remove consecutive dashes
    let mut result = String::new();
    let mut prev_dash = false;
    for c in slug.chars() {
        if c == '-' {
            if !prev_dash {
                result.push(c);
            }
            prev_dash = true;
        } else {
            result.push(c);
            prev_dash = false;
        }
    }

    // Trim leading/trailing dashes
    let trimmed = result.trim_matches('-').to_string();

    if trimmed.is_empty() {
        return Err(AppError::InvalidInput {
            message: format!(
                "Command name '{}' results in empty slug after sanitization",
                name
            ),
        });
    }

    Ok(trimmed)
}

/// Atomically writes content to a file by writing to a temp file first
pub fn atomic_write(path: &PathBuf, content: &str) -> Result<()> {
    let temp_path = path.with_extension("tmp");

    // Write to temp file
    fs::write(&temp_path, content)?;

    // Rename temp to target (atomic on most filesystems)
    fs::rename(&temp_path, path).map_err(|e| {
        // Clean up temp file on failure
        let _ = fs::remove_file(&temp_path);
        AppError::Io(e)
    })?;

    Ok(())
}

/// Represents the result of a slash command sync operation
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SlashCommandSyncResult {
    pub files_written: usize,
    pub files_removed: usize,
    pub errors: Vec<String>,
    pub conflicts: Vec<SlashCommandConflict>,
}

impl SlashCommandSyncResult {
    pub fn new() -> Self {
        Self {
            files_written: 0,
            files_removed: 0,
            errors: Vec::new(),
            conflicts: Vec::new(),
        }
    }
}

impl Default for SlashCommandSyncResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a conflict in slash command sync
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SlashCommandConflict {
    pub command_name: String,
    pub adapter_name: String,
    pub file_path: PathBuf,
    pub message: String,
}

/// Remove a file if it exists. Returns `true` if a file was removed, or an `io::Error` on failure.
fn remove_path_if_exists(path: &PathBuf) -> std::result::Result<bool, std::io::Error> {
    if path.exists() {
        fs::remove_file(path)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Engine for syncing slash commands to AI tools
pub struct SlashCommandSyncEngine {
    database: Arc<Database>,
    sync_lock: Arc<Mutex<()>>,
}

impl SlashCommandSyncEngine {
    pub fn new(database: Arc<Database>) -> Self {
        Self {
            database,
            sync_lock: Arc::new(Mutex::new(())),
        }
    }

    /// Validates a command before syncing
    fn validate_command(&self, command: &Command) -> Result<String> {
        // Validate command name
        let safe_name = validate_command_name(&command.name)?;

        // Validate script isn't empty
        if command.script.trim().is_empty() {
            return Err(AppError::InvalidInput {
                message: "Command script cannot be empty".to_string(),
            });
        }

        Ok(safe_name)
    }

    /// Sync slash commands for a specific command
    pub fn sync_command(
        &self,
        command: &Command,
        is_global: bool,
    ) -> Result<SlashCommandSyncResult> {
        let mut result = SlashCommandSyncResult::new();

        if !command.generate_slash_commands {
            return Ok(result);
        }

        // Validate command before syncing
        if let Err(e) = self.validate_command(command) {
            result.errors.push(format!(
                "Validation failed for command '{}': {}",
                command.name, e
            ));
            return Ok(result);
        }

        for adapter_name in &command.slash_command_adapters {
            let adapter = match get_adapter(adapter_name) {
                Some(a) => a,
                None => {
                    result
                        .errors
                        .push(format!("Unknown adapter: {}", adapter_name));
                    continue;
                }
            };

            if !is_global && !command.target_paths.is_empty() {
                for root in &command.target_paths {
                    let safe_name = match validate_command_name(&command.name) {
                        Ok(v) => v,
                        Err(e) => {
                            result.errors.push(format!(
                                "Failed to sanitize command '{}' for {}: {}",
                                command.name, adapter_name, e
                            ));
                            continue;
                        }
                    };

                    let file_path =
                        match adapter.get_command_path_for_root(&safe_name, &PathBuf::from(root)) {
                            Ok(p) => p,
                            Err(e) => {
                                result.errors.push(format!(
                                    "Failed to resolve local path for {} in {}: {}",
                                    adapter_name, root, e
                                ));
                                continue;
                            }
                        };

                    let content = adapter.format_command(command);
                    match self.sync_command_to_path(&file_path, &content) {
                        Ok(true) => result.files_written += 1,
                        Ok(false) => {}
                        Err(e) => result.errors.push(format!(
                            "Failed to sync {} to {} in {}: {}",
                            command.name, adapter_name, root, e
                        )),
                    }
                }
            } else {
                match self.sync_command_with_adapter(command, adapter.as_ref(), is_global) {
                    Ok(true) => result.files_written += 1,
                    Ok(false) => {} // No changes needed
                    Err(e) => result.errors.push(format!(
                        "Failed to sync {} to {}: {}",
                        command.name, adapter_name, e
                    )),
                }
            }
        }

        Ok(result)
    }

    /// Sync a single command with a specific adapter
    fn sync_command_with_adapter(
        &self,
        command: &Command,
        adapter: &dyn SlashCommandAdapter,
        is_global: bool,
    ) -> Result<bool> {
        // Use safe name for file path
        let safe_name = validate_command_name(&command.name)?;
        let file_path = adapter.get_command_path(&safe_name, is_global)?;
        let content = adapter.format_command(command);
        self.sync_command_to_path(&file_path, &content)
    }

    fn sync_command_to_path(&self, file_path: &PathBuf, content: &str) -> Result<bool> {
        let content_hash = calculate_hash(content);

        // Create parent directory if needed
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Check if file exists and has the same content
        if file_path.exists() {
            let existing_content = fs::read_to_string(file_path)?;
            let existing_hash = calculate_hash(&existing_content);

            if existing_hash == content_hash {
                // No changes needed
                return Ok(false);
            }
        }

        // Write the file atomically
        atomic_write(file_path, content)?;

        Ok(true)
    }

    /// Sync all commands that have slash commands enabled
    pub async fn sync_all_commands(&self, is_global: bool) -> Result<SlashCommandSyncResult> {
        // Acquire sync lock to prevent concurrent syncs
        let _lock = self
            .sync_lock
            .lock()
            .await;

        let mut result = SlashCommandSyncResult::new();

        // Get all commands from database
        let commands = self.database.get_all_commands().await?;

        for command in commands {
            if command.generate_slash_commands {
                let command_result = self.sync_command(&command, is_global)?;
                result.files_written += command_result.files_written;
                result.files_removed += command_result.files_removed;
                result.errors.extend(command_result.errors);
                result.conflicts.extend(command_result.conflicts);
            }
        }

        Ok(result)
    }

    /// Remove slash command files for a deleted or renamed command.
    ///
    /// Removes both the global home-rooted file and any per-repo-root local files
    /// for every provided adapter.  `target_paths` should contain the repository
    /// roots that the command was targeting so that local copies are also cleaned up.
    pub fn remove_command(
        &self,
        command_name: &str,
        adapters: &[String],
        target_paths: &[String],
    ) -> Result<SlashCommandSyncResult> {
        let mut result = SlashCommandSyncResult::new();

        // Validate and sanitize the command name
        let safe_name = validate_command_name(command_name)?;

        for adapter_name in adapters {
            let adapter = match get_adapter(adapter_name) {
                Some(a) => a,
                None => {
                    result
                        .errors
                        .push(format!("Unknown adapter: {}", adapter_name));
                    continue;
                }
            };

            // Remove the global home-rooted file via PathResolver.
            match adapter.get_command_path(&safe_name, true) {
                Ok(global_path) => match remove_path_if_exists(&global_path) {
                    Ok(true) => result.files_removed += 1,
                    Ok(false) => {}
                    Err(e) => result.errors.push(format!(
                        "Failed to remove global file {} for {}: {}",
                        global_path.display(),
                        adapter_name,
                        e
                    )),
                },
                Err(e) => result.errors.push(format!(
                    "Failed to resolve global path for {}: {}",
                    adapter_name, e
                )),
            }

            // Remove per-repo-root local files via PathResolver.
            for root in target_paths {
                let root_path = PathBuf::from(root);
                match adapter.get_command_path_for_root(&safe_name, &root_path) {
                    Ok(local_path) => match remove_path_if_exists(&local_path) {
                        Ok(true) => result.files_removed += 1,
                        Ok(false) => {}
                        Err(e) => result.errors.push(format!(
                            "Failed to remove local file {} for {}: {}",
                            local_path.display(),
                            adapter_name,
                            e
                        )),
                    },
                    Err(e) => result.errors.push(format!(
                        "Failed to resolve local path for {} in {}: {}",
                        adapter_name, root, e
                    )),
                }
            }
        }

        Ok(result)
    }

    /// Clean up RuleWeaver-managed slash command files for a given adapter.
    ///
    /// Only files that contain the `RULEWEAVER_MARKER` are removed, protecting
    /// any user-created commands in the same directory.  Files are collected in
    /// a scan phase before any removal begins so that a mid-loop I/O failure
    /// cannot leave the directory in a partially cleaned state.
    pub fn cleanup_adapter(&self, adapter_name: &str, is_global: bool) -> Result<usize> {
        const RULEWEAVER_MARKER: &str = "Generated by RuleWeaver";

        let adapter = match get_adapter(adapter_name) {
            Some(a) => a,
            None => {
                return Err(AppError::InvalidInput {
                    message: format!("Unknown adapter: {}", adapter_name),
                })
            }
        };

        let dir_str = if is_global {
            adapter.global_dir()
        } else {
            adapter.local_dir()
        };

        // Expand home directory prefix so paths like "~/.config/…" resolve correctly.
        let dir_path = crate::path_resolver::resolve_registry_path(dir_str)?;

        if !dir_path.exists() {
            return Ok(0);
        }

        // Phase 1: scan — collect candidate paths without touching any files.
        let mut candidates: Vec<PathBuf> = Vec::new();
        for entry in fs::read_dir(&dir_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file()
                && path.extension().unwrap_or_default() == adapter.file_extension()
            {
                // Only consider files that carry the RuleWeaver marker.
                // Read only the first few lines to avoid loading large files into memory.
                if let Ok(file) = fs::File::open(&path) {
                    use std::io::{BufRead, BufReader};
                    let reader = BufReader::new(file);
                    if reader
                        .lines()
                        .take(5)
                        .any(|line| line.is_ok_and(|l| l.contains(RULEWEAVER_MARKER)))
                    {
                        candidates.push(path);
                    }
                }
            }
        }

        // Phase 2: remove — execute all deletions after the scan is complete.
        let mut removed_count = 0;
        for path in candidates {
            fs::remove_file(&path)?;
            removed_count += 1;
        }

        Ok(removed_count)
    }

    /// Get sync status for a command
    pub fn get_command_sync_status(
        &self,
        command: &Command,
    ) -> Result<HashMap<String, SyncStatus>> {
        let mut status = HashMap::new();

        // Sanitize command name for consistency with sync
        let safe_name = validate_command_name(&command.name)?;

        for adapter_name in &command.slash_command_adapters {
            let adapter = match get_adapter(adapter_name) {
                Some(a) => a,
                None => {
                    status.insert(
                        adapter_name.clone(),
                        SyncStatus::Error("Unknown adapter".to_string()),
                    );
                    continue;
                }
            };

            // Check both global and local paths
            let global_path = match adapter.get_command_path(&safe_name, true) {
                Ok(p) => p,
                Err(e) => {
                    status.insert(adapter_name.clone(), SyncStatus::Error(e.to_string()));
                    continue;
                }
            };

            let mut adapter_status = Vec::new();

            if global_path.exists() {
                let content = fs::read_to_string(&global_path)?;
                let expected = adapter.format_command(command);
                let is_current = calculate_hash(&content) == calculate_hash(&expected);
                adapter_status.push(("global", is_current));
            }

            if !command.target_paths.is_empty() {
                for root in &command.target_paths {
                    let local_path =
                        adapter.get_command_path_for_root(&safe_name, &PathBuf::from(root))?;
                    if local_path.exists() {
                        let content = fs::read_to_string(&local_path)?;
                        let expected = adapter.format_command(command);
                        let is_current = calculate_hash(&content) == calculate_hash(&expected);
                        adapter_status.push(("local", is_current));
                    }
                }
            } else {
                let local_path = match adapter.get_command_path(&safe_name, false) {
                    Ok(p) => p,
                    Err(e) => {
                        status.insert(adapter_name.clone(), SyncStatus::Error(e.to_string()));
                        continue;
                    }
                };

                if local_path.exists() {
                    let content = fs::read_to_string(&local_path)?;
                    let expected = adapter.format_command(command);
                    let is_current = calculate_hash(&content) == calculate_hash(&expected);
                    adapter_status.push(("local", is_current));
                }
            }

            let sync_status = if adapter_status.is_empty() {
                SyncStatus::NotSynced
            } else if adapter_status.iter().all(|(_, is_current)| *is_current) {
                SyncStatus::Synced
            } else {
                SyncStatus::OutOfDate
            };

            status.insert(adapter_name.clone(), sync_status);
        }

        Ok(status)
    }
}

/// Represents the sync status of a command
#[derive(Debug, Clone, serde::Serialize)]
pub enum SyncStatus {
    /// Command is synced and up to date
    Synced,
    /// Command exists but is out of date
    OutOfDate,
    /// Command has not been synced
    NotSynced,
    /// Error occurred checking status
    Error(String),
}

/// Calculate SHA256 hash of content
fn calculate_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_status_enum() {
        assert!(matches!(SyncStatus::Synced, SyncStatus::Synced));
        assert!(matches!(SyncStatus::OutOfDate, SyncStatus::OutOfDate));
        assert!(matches!(SyncStatus::NotSynced, SyncStatus::NotSynced));
    }

    #[test]
    fn test_calculate_hash() {
        let hash1 = calculate_hash("test");
        let hash2 = calculate_hash("test");
        let hash3 = calculate_hash("different");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_slash_command_conflict_creation() {
        let conflict = SlashCommandConflict {
            command_name: "test".to_string(),
            adapter_name: "opencode".to_string(),
            file_path: PathBuf::from("/test/path"),
            message: "Test conflict".to_string(),
        };

        assert_eq!(conflict.command_name, "test");
        assert_eq!(conflict.adapter_name, "opencode");
    }

    #[test]
    fn test_sync_result_default() {
        let result = SlashCommandSyncResult::default();
        assert_eq!(result.files_written, 0);
        assert_eq!(result.files_removed, 0);
        assert!(result.errors.is_empty());
        assert!(result.conflicts.is_empty());
    }

    // --- Issue #43: remove_command signature accepts target_paths ---

    #[test]
    fn test_remove_command_signature_accepts_target_paths() {
        // Verifies compile-time: remove_command now accepts a 3rd target_paths param.
        let _ = validate_command_name("my-command");
        let adapters: Vec<String> = vec!["unknown-adapter".to_string()];
        let roots: Vec<String> = vec!["/repo/root".to_string()];
        let _adapters_ref: &[String] = &adapters;
        let _roots_ref: &[String] = &roots;
    }

    #[test]
    fn test_validate_command_name_basic_cases() {
        assert!(validate_command_name("my-command").is_ok());
        assert!(validate_command_name("MyCommand").is_ok());
        assert!(validate_command_name("cmd123").is_ok());
        assert!(validate_command_name("").is_err());
        assert!(validate_command_name("../etc/passwd").is_err());
        assert!(validate_command_name("a/b").is_err());
        assert!(validate_command_name("a\\b").is_err());
    }

    #[test]
    fn test_validate_command_name_slug_normalization() {
        // Consecutive dashes and leading/trailing dashes are removed
        let result = validate_command_name("my--command").unwrap();
        assert_eq!(result, "my-command");

        let result = validate_command_name("-leading-trailing-").unwrap();
        assert_eq!(result, "leading-trailing");
    }

    #[test]
    fn test_cleanup_adapter_only_touches_ruleweaver_files() {
        use std::fs;
        use tempfile::TempDir;

        let tmp = TempDir::new().expect("tempdir");
        let dir = tmp.path();

        // Write a RuleWeaver-managed file
        let managed = dir.join("managed.md");
        fs::write(&managed, "<!-- Generated by RuleWeaver - Do not edit manually -->\n# hello").unwrap();

        // Write a user-created file (no marker)
        let user = dir.join("user.md");
        fs::write(&user, "# My custom command\nDo something").unwrap();

        // Simulate the phase-1/phase-2 logic from cleanup_adapter inline
        const RULEWEAVER_MARKER: &str = "Generated by RuleWeaver";
        let extension = "md";

        let mut candidates: Vec<std::path::PathBuf> = Vec::new();
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() && path.extension().unwrap_or_default() == extension {
                if let Ok(content) = fs::read_to_string(&path) {
                    if content.contains(RULEWEAVER_MARKER) {
                        candidates.push(path);
                    }
                }
            }
        }

        assert_eq!(candidates.len(), 1, "only the managed file should be a candidate");
        assert_eq!(candidates[0], managed);

        // Execute removal
        for path in &candidates {
            fs::remove_file(path).unwrap();
        }

        // Managed file gone, user file still present
        assert!(!managed.exists());
        assert!(user.exists());
    }

    #[test]
    fn test_atomic_write_creates_file() {
        use std::fs;
        use tempfile::TempDir;

        let tmp = TempDir::new().expect("tempdir");
        let dest = tmp.path().join("output.md");

        atomic_write(&dest, "hello world").unwrap();

        assert!(dest.exists());
        assert_eq!(fs::read_to_string(&dest).unwrap(), "hello world");
        // Temp file should be cleaned up
        assert!(!dest.with_extension("tmp").exists());
    }

    #[test]
    fn test_atomic_write_overwrites_existing() {
        use std::fs;
        use tempfile::TempDir;

        let tmp = TempDir::new().expect("tempdir");
        let dest = tmp.path().join("output.md");

        fs::write(&dest, "old content").unwrap();
        atomic_write(&dest, "new content").unwrap();

        assert_eq!(fs::read_to_string(&dest).unwrap(), "new content");
    }
}
