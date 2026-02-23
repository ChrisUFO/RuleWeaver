use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::database::Database;
use crate::error::{AppError, Result};
use crate::models::Command;
use crate::slash_commands::{get_adapter, SlashCommandAdapter};

/// Represents the result of a slash command sync operation
#[derive(Debug, Clone)]
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

    pub fn success(&self) -> bool {
        self.errors.is_empty() && self.conflicts.is_empty()
    }
}

impl Default for SlashCommandSyncResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a conflict in slash command sync
#[derive(Debug, Clone)]
pub struct SlashCommandConflict {
    pub command_name: String,
    pub adapter_name: String,
    pub file_path: PathBuf,
    pub message: String,
}

/// Engine for syncing slash commands to AI tools
pub struct SlashCommandSyncEngine {
    database: std::sync::Arc<Database>,
}

impl SlashCommandSyncEngine {
    pub fn new(database: std::sync::Arc<Database>) -> Self {
        Self { database }
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

            match self.sync_command_with_adapter(command, adapter.as_ref(), is_global) {
                Ok(true) => result.files_written += 1,
                Ok(false) => {} // No changes needed
                Err(e) => result.errors.push(format!(
                    "Failed to sync {} to {}: {}",
                    command.name, adapter_name, e
                )),
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
        let file_path = adapter.get_command_path(&command.name, is_global);
        let content = adapter.format_command(command);
        let content_hash = calculate_hash(&content);

        // Create parent directory if needed
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Check if file exists and has the same content
        if file_path.exists() {
            let existing_content = fs::read_to_string(&file_path)?;
            let existing_hash = calculate_hash(&existing_content);

            if existing_hash == content_hash {
                // No changes needed
                return Ok(false);
            }
        }

        // Write the file
        fs::write(&file_path, content)?;

        Ok(true)
    }

    /// Sync all commands that have slash commands enabled
    pub fn sync_all_commands(&self, is_global: bool) -> Result<SlashCommandSyncResult> {
        let mut result = SlashCommandSyncResult::new();

        // Get all commands from database
        let commands = self.database.get_all_commands()?;

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

    /// Remove slash command files for a deleted command
    pub fn remove_command(
        &self,
        command_name: &str,
        adapters: &[String],
    ) -> Result<SlashCommandSyncResult> {
        let mut result = SlashCommandSyncResult::new();

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

            // Try to remove both global and local versions
            let global_path = adapter.get_command_path(command_name, true);
            let local_path = adapter.get_command_path(command_name, false);

            if global_path.exists() {
                fs::remove_file(&global_path)?;
                result.files_removed += 1;
            }

            if local_path.exists() {
                fs::remove_file(&local_path)?;
                result.files_removed += 1;
            }
        }

        Ok(result)
    }

    /// Clean up all slash command files for a given adapter
    pub fn cleanup_adapter(&self, adapter_name: &str, is_global: bool) -> Result<usize> {
        let adapter = match get_adapter(adapter_name) {
            Some(a) => a,
            None => {
                return Err(AppError::InvalidInput {
                    message: format!("Unknown adapter: {}", adapter_name),
                })
            }
        };

        let dir = if is_global {
            adapter.global_dir()
        } else {
            adapter.local_dir()
        };

        let dir_path = PathBuf::from(dir);
        let mut removed_count = 0;

        if dir_path.exists() {
            for entry in fs::read_dir(&dir_path)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_file()
                    && path.extension().unwrap_or_default() == adapter.file_extension()
                {
                    fs::remove_file(&path)?;
                    removed_count += 1;
                }
            }
        }

        Ok(removed_count)
    }

    /// Get sync status for a command
    pub fn get_command_sync_status(
        &self,
        command: &Command,
    ) -> Result<HashMap<String, SyncStatus>> {
        let mut status = HashMap::new();

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
            let global_path = adapter.get_command_path(&command.name, true);
            let local_path = adapter.get_command_path(&command.name, false);

            let mut adapter_status = Vec::new();

            if global_path.exists() {
                let content = fs::read_to_string(&global_path)?;
                let expected = adapter.format_command(command);
                let is_current = calculate_hash(&content) == calculate_hash(&expected);
                adapter_status.push(("global", is_current));
            }

            if local_path.exists() {
                let content = fs::read_to_string(&local_path)?;
                let expected = adapter.format_command(command);
                let is_current = calculate_hash(&content) == calculate_hash(&expected);
                adapter_status.push(("local", is_current));
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
#[derive(Debug, Clone)]
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
    use crate::models::{Command, CreateCommandInput};

    #[test]
    fn test_sync_status_enum() {
        assert!(matches!(SyncStatus::Synced, SyncStatus::Synced));
        assert!(matches!(SyncStatus::OutOfDate, SyncStatus::OutOfDate));
        assert!(matches!(SyncStatus::NotSynced, SyncStatus::NotSynced));
    }
}
