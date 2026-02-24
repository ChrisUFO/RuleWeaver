use crate::models::Command;
use crate::error::{AppError, Result};
use std::path::PathBuf;

pub trait SlashCommandAdapter: Send + Sync {
    /// Returns the adapter name (e.g., "opencode", "claude")
    fn name(&self) -> &'static str;
    
    /// Returns the file extension for this adapter (e.g., "md", "toml")
    fn file_extension(&self) -> &'static str;
    
    /// Returns the global directory path for this tool
    fn global_dir(&self) -> &'static str;
    
    /// Returns the local directory path for this tool
    fn local_dir(&self) -> &'static str;
    
    /// Formats a command into the tool-specific format
    fn format_command(&self, command: &Command) -> String;
    
    /// Returns the filename for a command (usually the command name)
    fn get_filename(&self, command_name: &str) -> String {
        format!("{}.{}", command_name, self.file_extension())
    }
    
    /// Returns the full path for a command
    fn get_command_path(&self, command_name: &str, is_global: bool) -> Result<PathBuf> {
        let path_str = if is_global {
            self.global_dir()
        } else {
            self.local_dir()
        };

        let base_path = if is_global {
            dirs::home_dir().ok_or_else(|| AppError::Path(
                "Could not determine home directory for global commands".to_string()
            ))?
        } else {
            PathBuf::new()
        };

        Ok(base_path.join(path_str).join(self.get_filename(command_name)))
    }

    /// Whether this adapter supports argument substitution
    fn supports_argument_substitution(&self) -> bool {
        false
    }

    /// Returns the argument substitution pattern (e.g., "$ARGUMENTS", "{{args}}")
    fn argument_pattern(&self) -> Option<&'static str> {
        None
    }
}

pub mod adapters;
pub mod commands;
pub mod sync;

pub use adapters::*;
pub use sync::*;
