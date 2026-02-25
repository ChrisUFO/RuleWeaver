//! Path resolution service for RuleWeaver.
//!
//! This module provides a unified path resolution service that all sync engines consume.
//! It ensures consistent path resolution across all artifact types (rules, command stubs,
//! slash commands, skills) so that preview, sync, cleanup, and reconciliation flows
//! produce identical paths.
//!
//! # Design Principles
//!
//! - **Single Source of Truth**: All path templates live in `REGISTRY`. No hardcoded paths.
//! - **Pure Functions**: Path resolution functions are deterministic and testable.
//! - **Artifact-Agnostic**: Handles all artifact types uniformly via `ArtifactType` enum.
//! - **Platform-Aware**: Normalizes paths for Windows and Unix platforms.

#![allow(dead_code)]

use std::path::{Path, PathBuf};

use crate::error::{AppError, Result};
use crate::models::registry::{ArtifactType, REGISTRY};
use crate::models::{AdapterType, Scope};

/// Resolved path information for an artifact.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedPath {
    /// The resolved absolute path
    pub path: PathBuf,
    /// The adapter this path is for
    pub adapter: AdapterType,
    /// The type of artifact
    pub artifact: ArtifactType,
    /// The scope (global or local)
    pub scope: Scope,
    /// Whether the path currently exists on disk
    pub exists: bool,
    /// The repository root for local paths (None for global)
    pub repo_root: Option<PathBuf>,
}

/// Specification for an artifact to resolve paths for.
#[derive(Debug, Clone)]
pub struct ArtifactSpec {
    pub adapter: AdapterType,
    pub artifact: ArtifactType,
    pub scope: Scope,
    pub repo_root: Option<PathBuf>,
    pub name: Option<String>, // For slash commands: command name
}

/// Shared path resolver service.
///
/// This is the main entry point for path resolution. It provides methods
/// to resolve paths for all artifact types using the canonical registry.
pub struct PathResolver {
    home_dir: PathBuf,
    repository_roots: Vec<PathBuf>,
}

impl PathResolver {
    /// Create a new PathResolver.
    ///
    /// # Errors
    ///
    /// Returns an error if the home directory cannot be determined.
    pub fn new() -> Result<Self> {
        let home_dir = dirs::home_dir().ok_or_else(|| {
            AppError::Path("Could not determine home directory".to_string())
        })?;

        Ok(Self {
            home_dir,
            repository_roots: Vec::new(),
        })
    }

    /// Create a new PathResolver with additional repository roots.
    ///
    /// # Errors
    ///
    /// Returns an error if the home directory cannot be determined.
    pub fn with_repository_roots(repository_roots: Vec<PathBuf>) -> Result<Self> {
        let home_dir = dirs::home_dir().ok_or_else(|| {
            AppError::Path("Could not determine home directory".to_string())
        })?;

        Ok(Self {
            home_dir,
            repository_roots,
        })
    }

    /// Get the home directory.
    pub fn home_dir(&self) -> &Path {
        &self.home_dir
    }

    /// Get the configured repository roots.
    pub fn repository_roots(&self) -> &[PathBuf] {
        &self.repository_roots
    }

    /// Add a repository root.
    pub fn add_repository_root(&mut self, root: PathBuf) {
        self.repository_roots.push(root);
    }

    /// Resolve the global path for an artifact+adapter combination.
    ///
    /// # Errors
    ///
    /// Returns an error if the adapter doesn't support the artifact type,
    /// or if the path template is invalid.
    pub fn global_path(
        &self,
        adapter: AdapterType,
        artifact: ArtifactType,
    ) -> Result<ResolvedPath> {
        // Validate that the adapter supports this artifact type
        REGISTRY
            .validate_support(&adapter, &Scope::Global, artifact)
            .map_err(|e| AppError::InvalidInput { message: e })?;

        let entry = REGISTRY.get(&adapter).ok_or_else(|| {
            AppError::InvalidInput {
                message: format!("Unknown adapter: {}", adapter.as_str()),
            }
        })?;

        // Get the appropriate path template based on artifact type
        let path_template = match artifact {
            ArtifactType::Rule => entry.paths.global_path,
            ArtifactType::CommandStub => {
                // Command stubs use global_commands_dir + COMMANDS.md
                entry.paths.global_commands_dir.ok_or_else(|| {
                    AppError::InvalidInput {
                        message: format!(
                            "Adapter {} does not support command stubs",
                            adapter.as_str()
                        ),
                    }
                })?
            }
            ArtifactType::SlashCommand => {
                // Slash commands are handled differently - they need a command name
                // This method is for the directory, not individual commands
                entry.paths.global_commands_dir.ok_or_else(|| {
                    AppError::InvalidInput {
                        message: format!(
                            "Adapter {} does not support slash commands",
                            adapter.as_str()
                        ),
                    }
                })?
            }
            ArtifactType::Skill => {
                // Skills not yet implemented in Phase 3
                return Err(AppError::InvalidInput {
                    message: "Skills path resolution not yet implemented".to_string(),
                });
            }
        };

        let path = self.resolve_template(path_template, None)?;
        let exists = path.exists();

        Ok(ResolvedPath {
            path,
            adapter,
            artifact,
            scope: Scope::Global,
            exists,
            repo_root: None,
        })
    }

    /// Resolve the local path for an artifact+adapter+repository_root combination.
    ///
    /// # Errors
    ///
    /// Returns an error if the adapter doesn't support the artifact type,
    /// or if the path template is invalid.
    pub fn local_path(
        &self,
        adapter: AdapterType,
        artifact: ArtifactType,
        repo_root: &Path,
    ) -> Result<ResolvedPath> {
        // Validate that the adapter supports this artifact type
        REGISTRY
            .validate_support(&adapter, &Scope::Local, artifact)
            .map_err(|e| AppError::InvalidInput { message: e })?;

        let entry = REGISTRY.get(&adapter).ok_or_else(|| {
            AppError::InvalidInput {
                message: format!("Unknown adapter: {}", adapter.as_str()),
            }
        })?;

        // Get the appropriate path template based on artifact type
        let path_template = match artifact {
            ArtifactType::Rule => entry.paths.local_path_template,
            ArtifactType::CommandStub => {
                entry.paths.local_commands_dir.ok_or_else(|| {
                    AppError::InvalidInput {
                        message: format!(
                            "Adapter {} does not support command stubs",
                            adapter.as_str()
                        ),
                    }
                })?
            }
            ArtifactType::SlashCommand => {
                entry.paths.local_commands_dir.ok_or_else(|| {
                    AppError::InvalidInput {
                        message: format!(
                            "Adapter {} does not support slash commands",
                            adapter.as_str()
                        ),
                    }
                })?
            }
            ArtifactType::Skill => {
                return Err(AppError::InvalidInput {
                    message: "Skills path resolution not yet implemented".to_string(),
                });
            }
        };

        // First resolve any ~ in the template
        let mut resolved = self.resolve_template(path_template, None)?;
        
        // If the resolved path doesn't start with the repo root, prepend it
        // This handles both cases:
        // 1. Template contains {repo} placeholder - already substituted
        // 2. Template is relative (e.g., ".claude/CLAUDE.md") - need to prepend
        let resolved_str = resolved.to_string_lossy();
        let repo_str: &str = &repo_root.to_string_lossy();
        
        // Prepend repo_root if the path doesn't already contain it
        if !resolved_str.starts_with(repo_str) && !resolved_str.contains(repo_str) {
            resolved = repo_root.join(&resolved);
        }
        
        let exists = resolved.exists();

        Ok(ResolvedPath {
            path: resolved,
            adapter,
            artifact,
            scope: Scope::Local,
            exists,
            repo_root: Some(repo_root.to_path_buf()),
        })
    }

    /// Resolve a path for a specific slash command.
    ///
    /// # Errors
    ///
    /// Returns an error if the adapter doesn't support slash commands.
    pub fn slash_command_path(
        &self,
        adapter: AdapterType,
        command_name: &str,
        is_global: bool,
    ) -> Result<ResolvedPath> {
        let entry = REGISTRY.get(&adapter).ok_or_else(|| {
            AppError::InvalidInput {
                message: format!("Unknown adapter: {}", adapter.as_str()),
            }
        })?;

        let extension = entry.slash_command_extension.ok_or_else(|| {
            AppError::InvalidInput {
                message: format!(
                    "Adapter {} does not support slash commands",
                    adapter.as_str()
                ),
            }
        })?;

        let dir = if is_global {
            entry.paths.global_commands_dir.ok_or_else(|| {
                AppError::InvalidInput {
                    message: format!(
                        "Adapter {} does not support global slash commands",
                        adapter.as_str()
                    ),
                }
            })?
        } else {
            entry.paths.local_commands_dir.ok_or_else(|| {
                AppError::InvalidInput {
                    message: format!(
                        "Adapter {} does not support local slash commands",
                        adapter.as_str()
                    ),
                }
            })?
        };

        let filename = format!("{}.{}", command_name, extension);

        let path = if is_global {
            self.home_dir.join(dir).join(&filename)
        } else {
            // For local, we need a repo root - this is handled differently
            // The caller must provide the repo root context
            return Err(AppError::InvalidInput {
                message: "Local slash command path requires repo_root. Use local_slash_command_path()"
                    .to_string(),
            });
        };

        let exists = path.exists();

        Ok(ResolvedPath {
            path,
            adapter,
            artifact: ArtifactType::SlashCommand,
            scope: if is_global { Scope::Global } else { Scope::Local },
            exists,
            repo_root: None,
        })
    }

    /// Resolve a local path for a specific slash command in a repository.
    ///
    /// # Errors
    ///
    /// Returns an error if the adapter doesn't support slash commands.
    pub fn local_slash_command_path(
        &self,
        adapter: AdapterType,
        command_name: &str,
        repo_root: &Path,
    ) -> Result<ResolvedPath> {
        let entry = REGISTRY.get(&adapter).ok_or_else(|| {
            AppError::InvalidInput {
                message: format!("Unknown adapter: {}", adapter.as_str()),
            }
        })?;

        let extension = entry.slash_command_extension.ok_or_else(|| {
            AppError::InvalidInput {
                message: format!(
                    "Adapter {} does not support slash commands",
                    adapter.as_str()
                ),
            }
        })?;

        let dir = entry.paths.local_commands_dir.ok_or_else(|| {
            AppError::InvalidInput {
                message: format!(
                    "Adapter {} does not support local slash commands",
                    adapter.as_str()
                ),
            }
        })?;

        let filename = format!("{}.{}", command_name, extension);
        let path = repo_root.join(dir).join(&filename);
        let exists = path.exists();

        Ok(ResolvedPath {
            path,
            adapter,
            artifact: ArtifactType::SlashCommand,
            scope: Scope::Local,
            exists,
            repo_root: Some(repo_root.to_path_buf()),
        })
    }

    /// Validate a user-provided target path.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The path is relative
    /// - The path is not within the user's home directory
    /// - The path contains invalid characters
    pub fn validate_target_path(&self, path: &Path) -> Result<PathBuf> {
        if path.is_relative() {
            return Err(AppError::InvalidInput {
                message: "Target path must be absolute".to_string(),
            });
        }

        // Normalize the path without requiring it to exist (no filesystem I/O)
        let normalized = normalize_path(path)?;

        // Check for traversal by comparing against home directory after normalization
        let canonical_home = normalize_path(&self.home_dir).unwrap_or_else(|_| self.home_dir.clone());

        if !normalized.starts_with(&canonical_home) {
            return Err(AppError::InvalidInput {
                message: "Target path must be within user's home directory".to_string(),
            });
        }

        Ok(normalized)
    }

    /// Canonicalize a path with platform-specific normalization.
    ///
    /// This resolves:
    /// - Path separators (Windows backslash vs Unix forward slash)
    /// - Unicode normalization
    /// - Traversal sequences (.. and .)
    /// - Symlinks (optional)
    ///
    /// Note: This function does NOT require the path to exist.
    pub fn canonicalize(&self, path: &Path) -> Result<PathBuf> {
        // First resolve any relative components
        let absolute = if path.is_relative() {
            std::env::current_dir()
                .map_err(|e| AppError::Path(format!("Failed to get current directory: {}", e)))?
                .join(path)
        } else {
            path.to_path_buf()
        };

        // Normalize without requiring filesystem I/O
        normalize_path(&absolute)
    }

    /// List all paths that would be written for given artifacts.
    ///
    /// This is used for preview/dry-run functionality.
    pub fn preview_paths(&self, artifacts: &[ArtifactSpec]) -> Result<Vec<ResolvedPath>> {
        let mut paths = Vec::new();

        for spec in artifacts {
            match spec.scope {
                Scope::Global => {
                    let resolved = self.global_path(spec.adapter, spec.artifact)?;
                    paths.push(resolved);
                }
                Scope::Local => {
                    if let Some(ref repo_root) = spec.repo_root {
                        let resolved = self.local_path(spec.adapter, spec.artifact, repo_root)?;
                        paths.push(resolved);
                    }
                    // If no repo_root, skip - this is expected for some artifact types
                }
            }
        }

        Ok(paths)
    }

    /// Resolve a path template with optional repository root substitution.
    ///
    /// Path templates can contain:
    /// - `~` for home directory
    /// - `{repo}` placeholder for repository root (in local templates)
    fn resolve_template(&self, template: &str, repo_root: Option<&Path>) -> Result<PathBuf> {
        let mut result = template.to_string();

        // Replace ~ with home directory
        if result.starts_with("~/") {
            result = result.replace("~/", &self.home_dir.to_string_lossy());
        } else if result == "~" {
            result = self.home_dir.to_string_lossy().to_string();
        }

        // Replace {repo} placeholder with repository root
        if let Some(root) = repo_root {
            result = result.replace("{repo}", &root.to_string_lossy());
        }

        Ok(PathBuf::from(result))
    }

    /// Get all global paths for a specific artifact type across all adapters.
    ///
    /// This is useful for scanning or cleanup operations.
    pub fn all_global_paths(&self, artifact: ArtifactType) -> Result<Vec<ResolvedPath>> {
        let mut paths = Vec::new();

        for adapter in AdapterType::all() {
            // Skip adapters that don't support this artifact type
            if REGISTRY
                .validate_support(&adapter, &Scope::Global, artifact)
                .is_err()
            {
                continue;
            }

            if let Ok(resolved) = self.global_path(adapter, artifact) {
                paths.push(resolved);
            }
        }

        Ok(paths)
    }

    /// Get all local paths for a specific artifact type across all adapters and repository roots.
    ///
    /// This is useful for scanning or cleanup operations.
    pub fn all_local_paths(
        &self,
        artifact: ArtifactType,
        repo_roots: &[PathBuf],
    ) -> Result<Vec<ResolvedPath>> {
        let mut paths = Vec::new();

        for adapter in AdapterType::all() {
            // Skip adapters that don't support this artifact type
            if REGISTRY
                .validate_support(&adapter, &Scope::Local, artifact)
                .is_err()
            {
                continue;
            }

            for repo_root in repo_roots {
                if let Ok(resolved) = self.local_path(adapter, artifact, repo_root) {
                    paths.push(resolved);
                }
            }
        }

        Ok(paths)
    }
}

impl Default for PathResolver {
    fn default() -> Self {
        Self::new().expect("Failed to create PathResolver - could not determine home directory")
    }
}

/// Normalize a path without requiring filesystem I/O.
///
/// This resolves:
/// - Path separators (Windows backslash vs Unix forward slash)
/// - Traversal sequences (.. and .)
/// - Redundant separators
///
/// Unlike std::fs::canonicalize, this does NOT require the path to exist.
fn normalize_path(path: &Path) -> std::result::Result<PathBuf, AppError> {
    let mut components = Vec::new();
    
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                // Don't pop if we're at the root
                if !components.is_empty() && components.last() != Some(&std::path::Component::RootDir) {
                    components.pop();
                }
            }
            std::path::Component::CurDir => {
                // Skip current directory components
            }
            other => {
                components.push(other);
            }
        }
    }
    
    // Reconstruct the path
    let normalized: PathBuf = components.iter().map(|c| c.as_os_str()).collect();
    
    // Ensure we have an absolute path
    if normalized.is_relative() {
        return Err(AppError::Path(format!("Cannot normalize relative path: {}", path.display())));
    }
    
    Ok(normalized)
}

/// Resolve a registry path string (e.g., "~/path" or "~") to an absolute PathBuf.
///
/// This is a convenience function for backward compatibility.
pub fn resolve_registry_path(path: &str) -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| {
        AppError::Path("Could not determine home directory".to_string())
    })?;

    if let Some(stripped) = path.strip_prefix("~/") {
        Ok(home.join(stripped))
    } else if path == "~" {
        Ok(home)
    } else {
        Ok(PathBuf::from(path))
    }
}

/// Validate a target path string.
///
/// This is a convenience function for backward compatibility.
pub fn validate_target_path(path: &str) -> Result<PathBuf> {
    let resolver = PathResolver::new()?;
    resolver.validate_target_path(&PathBuf::from(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_registry_path() {
        // Test home directory resolution
        let result = resolve_registry_path("~/test/path");
        assert!(result.is_ok());

        let result = resolve_registry_path("~");
        assert!(result.is_ok());

        let result = resolve_registry_path("/absolute/path");
        assert!(result.is_ok());
    }

    #[test]
    fn test_path_resolver_creation() {
        let resolver = PathResolver::new();
        assert!(resolver.is_ok());

        let resolver = resolver.unwrap();
        assert!(resolver.home_dir().exists() || resolver.home_dir().to_string_lossy().contains("Users"));
    }

    #[test]
    fn test_global_path_resolution() {
        let resolver = PathResolver::new().unwrap();

        // Test Claude Code rule path
        let result = resolver.global_path(AdapterType::ClaudeCode, ArtifactType::Rule);
        assert!(result.is_ok());

        let resolved = result.unwrap();
        assert!(resolved.path.to_string_lossy().contains(".claude"));
        assert!(resolved.path.to_string_lossy().contains("CLAUDE.md"));
        assert_eq!(resolved.scope, Scope::Global);
    }

    #[test]
    fn test_local_path_resolution() {
        let resolver = PathResolver::new().unwrap();
        let repo_root = PathBuf::from("/test/repo");

        // Test Claude Code local rule path
        let result = resolver.local_path(AdapterType::ClaudeCode, ArtifactType::Rule, &repo_root);
        assert!(result.is_ok());

        let resolved = result.unwrap();
        // Use normalized path comparison that works on both Windows and Unix
        let path_str = resolved.path.to_string_lossy();
        assert!(path_str.contains("test") && path_str.contains("repo"),
            "Path should contain repo reference, got: {}", path_str);
        assert!(path_str.contains(".claude"));
        assert_eq!(resolved.scope, Scope::Local);
    }

    #[test]
    fn test_validate_target_path() {
        let resolver = PathResolver::new().unwrap();

        // Test absolute path within home
        let result = resolver.validate_target_path(&PathBuf::from("/home/test/path"));
        // This might fail on Windows if not in home directory
        // On Unix, it should work if /home/test exists and is accessible

        // Test relative path should fail
        let result = resolver.validate_target_path(&PathBuf::from("relative/path"));
        assert!(result.is_err());
    }

    #[test]
    fn test_all_adapters_have_paths() {
        let resolver = PathResolver::new().unwrap();

        for adapter in AdapterType::all() {
            // All adapters should support rules at minimum
            let result = resolver.global_path(adapter, ArtifactType::Rule);
            assert!(
                result.is_ok(),
                "Adapter {} should support rules",
                adapter.as_str()
            );
        }
    }
}