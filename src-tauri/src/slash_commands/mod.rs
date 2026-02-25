use crate::error::{AppError, Result};
use crate::models::Command;
use crate::path_resolver::path_resolver;
use std::path::PathBuf;

#[allow(dead_code)]
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
    ///
    /// This method now uses the PathResolver for consistent path resolution.
    fn get_command_path(&self, command_name: &str, is_global: bool) -> Result<PathBuf> {
        let resolver = path_resolver();
        
        // Get the adapter type from the adapter name
        let adapter = crate::models::AdapterType::from_str(self.name())
            .ok_or_else(|| AppError::InvalidInput {
                message: format!("Unknown adapter: {}", self.name()),
            })?;

        if is_global {
            // Use PathResolver for global paths
            let resolved = resolver.slash_command_path(adapter, command_name, true)?;
            Ok(resolved.path)
        } else {
            // For local paths without a repo root, we need the caller to provide one
            // This is a breaking change - callers must now provide repo_root for local commands
            Err(AppError::InvalidInput {
                message: "Local slash command path requires repo_root. Use get_command_path_for_root() instead".to_string(),
            })
        }
    }

    /// Returns a local command path rooted at a specific repository path.
    ///
    /// This method now uses the PathResolver for consistent path resolution.
    fn get_command_path_for_root(&self, command_name: &str, root: &std::path::Path) -> Result<PathBuf> {
        let resolver = path_resolver();
        
        // Get the adapter type from the adapter name
        let adapter = crate::models::AdapterType::from_str(self.name())
            .ok_or_else(|| AppError::InvalidInput {
                message: format!("Unknown adapter: {}", self.name()),
            })?;

        // Use PathResolver for local paths with repo root
        let resolved = resolver.local_slash_command_path(adapter, command_name, root)?;
        Ok(resolved.path)
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
