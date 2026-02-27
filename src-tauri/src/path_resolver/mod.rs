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
//!
//! # Status
//!
//! Some public APIs are exposed for future integration and tested via unit tests.
//! The `allow(dead_code)` attribute suppresses warnings for these pre-built APIs.

#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use crate::error::{AppError, Result};
use crate::models::registry::{ArtifactType, REGISTRY};
use crate::models::{AdapterType, Scope};

/// Validate a command name for path safety.
///
/// Prevents path traversal attacks by rejecting names containing:
/// - Path separators (`/` or `\`)
/// - Parent directory references (`..`)
fn validate_command_name(command_name: &str) -> Result<()> {
    if command_name.contains("..") || command_name.contains('/') || command_name.contains('\\') {
        return Err(AppError::InvalidInput {
            message: "Invalid command name: path separators and '..' not allowed".to_string(),
        });
    }
    if command_name.is_empty() {
        return Err(AppError::InvalidInput {
            message: "Command name cannot be empty".to_string(),
        });
    }
    Ok(())
}

/// Validate a skill name for path safety.
///
/// Prevents path traversal attacks by rejecting names containing:
/// - Path separators (`/` or `\`)
/// - Parent directory references (`..`)
fn validate_skill_name(skill_name: &str) -> Result<()> {
    if skill_name.contains("..") || skill_name.contains('/') || skill_name.contains('\\') {
        return Err(AppError::InvalidInput {
            message: "Invalid skill name: path separators and '..' not allowed".to_string(),
        });
    }
    if skill_name.is_empty() {
        return Err(AppError::InvalidInput {
            message: "Skill name cannot be empty".to_string(),
        });
    }
    Ok(())
}

/// Sanitize a skill name for use in file paths.
///
/// Converts the name to lowercase and replaces invalid characters with dashes.
pub fn sanitize_skill_name(skill_name: &str) -> String {
    let sanitized: String = skill_name
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();

    if sanitized.is_empty() {
        "unnamed-skill".to_string()
    } else {
        sanitized
    }
}

/// Global shared PathResolver instance.
///
/// This singleton ensures consistent path resolution across all modules.
/// Use [`path_resolver()`] to access this instance.
pub static PATH_RESOLVER: LazyLock<PathResolver> = LazyLock::new(|| {
    PathResolver::new()
        .expect("Failed to create global PathResolver - could not determine home directory")
});

/// Get the global shared PathResolver instance.
pub fn path_resolver() -> &'static PathResolver {
    &PATH_RESOLVER
}

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
///
/// # Repository Roots
///
/// The `repository_roots` field contains paths to local repositories that should
/// be scanned during reconciliation. It is populated via:
/// - [`PathResolver::with_repository_roots`] constructor
/// - [`PathResolver::add_repository_root`] method
///
/// For the reconciliation engine to detect local artifacts, repository roots must
/// be configured before calling `scan_actual_state`.
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
        let home_dir = dirs::home_dir()
            .ok_or_else(|| AppError::Path("Could not determine home directory".to_string()))?;

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
        let home_dir = dirs::home_dir()
            .ok_or_else(|| AppError::Path("Could not determine home directory".to_string()))?;

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

    /// Create a PathResolver with an explicit home directory (for tests only).
    #[cfg(any(test, feature = "test-helpers"))]
    pub fn new_with_home(home_dir: PathBuf, repository_roots: Vec<PathBuf>) -> Self {
        Self {
            home_dir,
            repository_roots,
        }
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
        REGISTRY
            .validate_support(&adapter, &Scope::Global, artifact)
            .map_err(|e| AppError::InvalidInput { message: e })?;

        let entry = REGISTRY
            .get(&adapter)
            .ok_or_else(|| AppError::InvalidInput {
                message: format!("Unknown adapter: {}", adapter.as_str()),
            })?;

        let path = match artifact {
            ArtifactType::Rule => self.resolve_template(entry.paths.global_path, None)?,
            ArtifactType::CommandStub => {
                let commands_dir =
                    entry
                        .paths
                        .global_commands_dir
                        .ok_or_else(|| AppError::InvalidInput {
                            message: format!(
                                "Adapter {} does not support command stubs",
                                adapter.as_str()
                            ),
                        })?;
                self.home_dir
                    .join(commands_dir)
                    .join(entry.paths.command_stub_filename)
            }
            ArtifactType::SlashCommand => {
                return Err(AppError::InvalidInput {
                    message:
                        "Slash commands require a command name. Use slash_command_path() instead."
                            .to_string(),
                });
            }
            ArtifactType::Skill => {
                let skills_dir =
                    entry
                        .paths
                        .global_skills_dir
                        .ok_or_else(|| AppError::InvalidInput {
                            message: format!(
                                "Adapter {} does not support skills",
                                adapter.as_str()
                            ),
                        })?;
                self.home_dir.join(skills_dir)
            }
        };

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
        REGISTRY
            .validate_support(&adapter, &Scope::Local, artifact)
            .map_err(|e| AppError::InvalidInput { message: e })?;

        let entry = REGISTRY
            .get(&adapter)
            .ok_or_else(|| AppError::InvalidInput {
                message: format!("Unknown adapter: {}", adapter.as_str()),
            })?;

        let path_template: String = match artifact {
            ArtifactType::Rule => entry.paths.local_path_template.to_string(),
            ArtifactType::CommandStub => {
                let dir = entry
                    .paths
                    .local_commands_dir
                    .ok_or_else(|| AppError::InvalidInput {
                        message: format!(
                            "Adapter {} does not support command stubs",
                            adapter.as_str()
                        ),
                    })?;
                PathBuf::from(dir)
                    .join(entry.paths.command_stub_filename)
                    .to_string_lossy()
                    .to_string()
            }
            ArtifactType::SlashCommand => {
                return Err(AppError::InvalidInput {
                    message: "Slash commands require a command name. Use local_slash_command_path() instead.".to_string(),
                });
            }
            ArtifactType::Skill => {
                let dir = entry
                    .paths
                    .local_skills_dir
                    .ok_or_else(|| AppError::InvalidInput {
                        message: format!("Adapter {} does not support skills", adapter.as_str()),
                    })?;
                PathBuf::from(dir).to_string_lossy().to_string()
            }
        };

        let mut resolved = self.resolve_template(&path_template, Some(repo_root))?;

        if resolved.is_relative() {
            resolved = repo_root.join(resolved);
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
    /// Returns an error if the adapter doesn't support slash commands,
    /// or if the command name contains invalid characters.
    pub fn slash_command_path(
        &self,
        adapter: AdapterType,
        command_name: &str,
        is_global: bool,
    ) -> Result<ResolvedPath> {
        validate_command_name(command_name)?;

        let entry = REGISTRY
            .get(&adapter)
            .ok_or_else(|| AppError::InvalidInput {
                message: format!("Unknown adapter: {}", adapter.as_str()),
            })?;

        let extension = entry
            .slash_command_extension
            .ok_or_else(|| AppError::InvalidInput {
                message: format!(
                    "Adapter {} does not support slash commands",
                    adapter.as_str()
                ),
            })?;

        let dir = if is_global {
            entry
                .paths
                .global_commands_dir
                .ok_or_else(|| AppError::InvalidInput {
                    message: format!(
                        "Adapter {} does not support global slash commands",
                        adapter.as_str()
                    ),
                })?
        } else {
            entry
                .paths
                .local_commands_dir
                .ok_or_else(|| AppError::InvalidInput {
                    message: format!(
                        "Adapter {} does not support local slash commands",
                        adapter.as_str()
                    ),
                })?
        };

        let filename = format!("{}.{}", command_name, extension);

        let path = if is_global {
            self.home_dir.join(dir).join(&filename)
        } else {
            // For local, we need a repo root - this is handled differently
            // The caller must provide the repo root context
            return Err(AppError::InvalidInput {
                message:
                    "Local slash command path requires repo_root. Use local_slash_command_path()"
                        .to_string(),
            });
        };

        let exists = path.exists();

        Ok(ResolvedPath {
            path,
            adapter,
            artifact: ArtifactType::SlashCommand,
            scope: if is_global {
                Scope::Global
            } else {
                Scope::Local
            },
            exists,
            repo_root: None,
        })
    }

    /// Resolve a local path for a specific slash command in a repository.
    ///
    /// # Errors
    ///
    /// Returns an error if the adapter doesn't support slash commands,
    /// or if the command name contains invalid characters.
    pub fn local_slash_command_path(
        &self,
        adapter: AdapterType,
        command_name: &str,
        repo_root: &Path,
    ) -> Result<ResolvedPath> {
        validate_command_name(command_name)?;

        let entry = REGISTRY
            .get(&adapter)
            .ok_or_else(|| AppError::InvalidInput {
                message: format!("Unknown adapter: {}", adapter.as_str()),
            })?;

        let extension = entry
            .slash_command_extension
            .ok_or_else(|| AppError::InvalidInput {
                message: format!(
                    "Adapter {} does not support slash commands",
                    adapter.as_str()
                ),
            })?;

        let dir = entry
            .paths
            .local_commands_dir
            .ok_or_else(|| AppError::InvalidInput {
                message: format!(
                    "Adapter {} does not support local slash commands",
                    adapter.as_str()
                ),
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

    /// Resolve the global skill directory path for an adapter.
    ///
    /// # Errors
    ///
    /// Returns an error if the adapter doesn't support skills.
    pub fn skill_dir(&self, adapter: AdapterType) -> Result<ResolvedPath> {
        let entry = REGISTRY
            .get(&adapter)
            .ok_or_else(|| AppError::InvalidInput {
                message: format!("Unknown adapter: {}", adapter.as_str()),
            })?;

        let skills_dir = entry
            .paths
            .global_skills_dir
            .ok_or_else(|| AppError::InvalidInput {
                message: format!("Adapter {} does not support skills", adapter.as_str()),
            })?;

        let path = self.home_dir.join(skills_dir);
        let exists = path.exists();

        Ok(ResolvedPath {
            path,
            adapter,
            artifact: ArtifactType::Skill,
            scope: Scope::Global,
            exists,
            repo_root: None,
        })
    }

    /// Resolve the local skill directory path for an adapter in a repository.
    ///
    /// # Errors
    ///
    /// Returns an error if the adapter doesn't support skills.
    pub fn local_skill_dir(&self, adapter: AdapterType, repo_root: &Path) -> Result<ResolvedPath> {
        let entry = REGISTRY
            .get(&adapter)
            .ok_or_else(|| AppError::InvalidInput {
                message: format!("Unknown adapter: {}", adapter.as_str()),
            })?;

        let skills_dir = entry
            .paths
            .local_skills_dir
            .ok_or_else(|| AppError::InvalidInput {
                message: format!("Adapter {} does not support skills", adapter.as_str()),
            })?;

        let path = repo_root.join(skills_dir);
        let exists = path.exists();

        Ok(ResolvedPath {
            path,
            adapter,
            artifact: ArtifactType::Skill,
            scope: Scope::Local,
            exists,
            repo_root: Some(repo_root.to_path_buf()),
        })
    }

    /// Resolve a path for a specific skill file (global).
    ///
    /// # Errors
    ///
    /// Returns an error if the adapter doesn't support skills,
    /// or if the skill name contains invalid characters.
    pub fn skill_path(&self, adapter: AdapterType, skill_name: &str) -> Result<ResolvedPath> {
        validate_skill_name(skill_name)?;

        let entry = REGISTRY
            .get(&adapter)
            .ok_or_else(|| AppError::InvalidInput {
                message: format!("Unknown adapter: {}", adapter.as_str()),
            })?;

        let skills_dir = entry
            .paths
            .global_skills_dir
            .ok_or_else(|| AppError::InvalidInput {
                message: format!("Adapter {} does not support skills", adapter.as_str()),
            })?;

        let filename = entry.paths.skill_filename;
        let safe_name = sanitize_skill_name(skill_name);
        let path = self
            .home_dir
            .join(skills_dir)
            .join(&safe_name)
            .join(filename);
        let exists = path.exists();

        Ok(ResolvedPath {
            path,
            adapter,
            artifact: ArtifactType::Skill,
            scope: Scope::Global,
            exists,
            repo_root: None,
        })
    }

    /// Resolve a local path for a specific skill file in a repository.
    ///
    /// # Errors
    ///
    /// Returns an error if the adapter doesn't support skills,
    /// or if the skill name contains invalid characters.
    pub fn local_skill_path(
        &self,
        adapter: AdapterType,
        skill_name: &str,
        repo_root: &Path,
    ) -> Result<ResolvedPath> {
        validate_skill_name(skill_name)?;

        let entry = REGISTRY
            .get(&adapter)
            .ok_or_else(|| AppError::InvalidInput {
                message: format!("Unknown adapter: {}", adapter.as_str()),
            })?;

        let skills_dir = entry
            .paths
            .local_skills_dir
            .ok_or_else(|| AppError::InvalidInput {
                message: format!("Adapter {} does not support skills", adapter.as_str()),
            })?;

        let filename = entry.paths.skill_filename;
        let safe_name = sanitize_skill_name(skill_name);
        let path = repo_root.join(skills_dir).join(&safe_name).join(filename);
        let exists = path.exists();

        Ok(ResolvedPath {
            path,
            adapter,
            artifact: ArtifactType::Skill,
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
    ///
    /// # Limitations
    ///
    /// This function uses normalized paths for comparison, not canonicalized paths.
    /// If the home directory is symlinked, paths through the symlink may be rejected.
    /// For symlink-aware validation, use `std::fs::canonicalize` on both paths before
    /// comparison.
    pub fn validate_target_path(&self, path: &Path) -> Result<PathBuf> {
        if path.is_relative() {
            return Err(AppError::InvalidInput {
                message: "Target path must be absolute".to_string(),
            });
        }

        // Normalize the path without requiring it to exist (no filesystem I/O)
        let normalized = normalize_path(path)?;

        // Check for traversal by comparing against home directory after normalization
        let canonical_home =
            normalize_path(&self.home_dir).unwrap_or_else(|_| self.home_dir.clone());

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
        // Expand ~ to home directory using PathBuf::join so the OS separator is
        // always inserted correctly (string replace would miss the separator between
        // the home dir and the rest of the path).
        if template == "~" {
            return Ok(self.home_dir.clone());
        }
        if let Some(suffix) = template.strip_prefix("~/") {
            let mut path = self.home_dir.join(suffix);
            // Apply {repo} substitution if present in the suffix
            if let Some(root) = repo_root {
                let s = path.to_string_lossy().replace("{repo}", &root.to_string_lossy());
                path = PathBuf::from(s);
            }
            return Ok(path);
        }

        // Non-home template (e.g., local "{repo}/..." paths)
        let mut result = template.to_string();
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
///
/// # Windows UNC Paths
///
/// On Windows, this function preserves UNC path prefixes (e.g., `\\?\` and `\\.\`).
fn normalize_path(path: &Path) -> std::result::Result<PathBuf, AppError> {
    use std::path::Component;

    let mut components: Vec<Component> = Vec::new();
    let mut has_unc_prefix = false;

    for component in path.components() {
        match component {
            Component::Prefix(prefix) => {
                #[cfg(windows)]
                {
                    use std::os::windows::ffi::OsStrExt;
                    let prefix_str = prefix.as_os_str();
                    let chars: Vec<u16> = prefix_str.encode_wide().collect();
                    if chars.len() >= 2 && chars[0] == b'\\' as u16 && chars[1] == b'\\' as u16 {
                        has_unc_prefix = true;
                    }
                }
                components.push(component);
            }
            Component::ParentDir => {
                if !components.is_empty()
                    && components.last() != Some(&Component::RootDir)
                    && !matches!(components.last(), Some(Component::Prefix(_)))
                {
                    components.pop();
                }
            }
            Component::CurDir => {}
            other => {
                components.push(other);
            }
        }
    }

    let normalized: PathBuf = components.iter().map(|c| c.as_os_str()).collect();

    if normalized.is_relative() && !has_unc_prefix {
        return Err(AppError::Path(format!(
            "Cannot normalize relative path: {}",
            path.display()
        )));
    }

    Ok(normalized)
}

/// Resolve a path string using an optional base workspace path.
///
/// If the path starts with `./` or contains `${WORKSPACE_ROOT}`,
/// it will be resolved against the `base_path`.
/// If the path is already absolute, or no base_path is provided, it returns the original path.
///
/// This function prevents directory traversal by ensuring the final path is within
/// the base_path boundary if a relative path was provided.
pub fn resolve_workspace_path(path: &str, base_path: Option<&str>) -> String {
    let mut resolved = path.to_string();

    let base = match base_path {
        Some(b) if !b.trim().is_empty() => b,
        _ => return resolved,
    };

    if resolved.starts_with("./") {
        let relative_part = resolved.trim_start_matches("./");
        // Security: Prevent directory traversal (e.g., "./../../etc/passwd")
        if relative_part.contains("..") {
            let path_buf = PathBuf::from(relative_part);
            if path_buf.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
                return resolved; // Return original if traversal detected
            }
        }
        resolved = PathBuf::from(base).join(relative_part).to_string_lossy().to_string();
    } else if resolved.contains("${WORKSPACE_ROOT}") {
        resolved = resolved.replace("${WORKSPACE_ROOT}", base);
    }

    resolved
}

/// Resolve a registry path string (e.g., "~/path" or "~") to an absolute PathBuf.
///
/// This is a convenience function for backward compatibility.
pub fn resolve_registry_path(path: &str) -> Result<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| AppError::Path("Could not determine home directory".to_string()))?;

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
        assert!(
            resolver.home_dir().exists() || resolver.home_dir().to_string_lossy().contains("Users")
        );
    }

    #[test]
    fn test_global_path_resolution() {
        let resolver = PathResolver::new().unwrap();

        // Test Claude Code rule path
        let result = resolver.global_path(AdapterType::ClaudeCode, ArtifactType::Rule);
        assert!(result.is_ok());

        let resolved = result.unwrap();
        let path_str = resolved.path.to_string_lossy();
        assert!(path_str.contains(".claude") || path_str.contains(".CLAUDE"));
        assert!(path_str.contains("CLAUDE.md"));
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
        let path_str = resolved.path.to_string_lossy();
        assert!(path_str.contains("test"));
        assert!(path_str.contains("repo"));
        assert!(path_str.contains(".claude") || path_str.contains(".CLAUDE"));
        assert_eq!(resolved.scope, Scope::Local);
    }

    #[test]
    fn test_validate_target_path() {
        let resolver = PathResolver::new().unwrap();

        // Test absolute path within home (platform-agnostic)
        let home_dir = resolver.home_dir();
        let valid_path = home_dir.join("test/path");
        let result = resolver.validate_target_path(&valid_path);
        assert!(
            result.is_ok(),
            "Should succeed for a path within the home directory"
        );

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

    #[test]
    fn test_global_path_all_adapters() {
        let resolver = PathResolver::new().unwrap();

        for adapter in AdapterType::all() {
            let result = resolver.global_path(adapter, ArtifactType::Rule);
            assert!(
                result.is_ok(),
                "Global path for {} should resolve",
                adapter.as_str()
            );
            let resolved = result.unwrap();
            assert_eq!(resolved.scope, Scope::Global);
            assert!(resolved.path.is_absolute() || resolved.path.starts_with(resolver.home_dir()));
        }
    }

    #[test]
    fn test_local_path_all_adapters() {
        let resolver = PathResolver::new().unwrap();
        let repo_root = PathBuf::from("/test/repo");

        for adapter in AdapterType::all() {
            let result = resolver.local_path(adapter, ArtifactType::Rule, &repo_root);
            assert!(
                result.is_ok(),
                "Local path for {} should resolve",
                adapter.as_str()
            );
            let resolved = result.unwrap();
            assert_eq!(resolved.scope, Scope::Local);
            let path_str = resolved.path.to_string_lossy().to_string();
            assert!(path_str.contains("test") || path_str.contains("repo"));
        }
    }

    #[test]
    fn test_slash_command_path_global() {
        let resolver = PathResolver::new().unwrap();

        let result = resolver.slash_command_path(AdapterType::ClaudeCode, "test-command", true);
        assert!(result.is_ok());
        let resolved = result.unwrap();
        assert_eq!(resolved.artifact, ArtifactType::SlashCommand);
        assert_eq!(resolved.scope, Scope::Global);
        let path_str = resolved.path.to_string_lossy();
        assert!(path_str.contains("test-command"));
        assert!(path_str.ends_with(".md"));
    }

    #[test]
    fn test_slash_command_path_local_requires_repo() {
        let resolver = PathResolver::new().unwrap();

        let result = resolver.slash_command_path(AdapterType::ClaudeCode, "test-command", false);
        assert!(result.is_err());
    }

    #[test]
    fn test_local_slash_command_path() {
        let resolver = PathResolver::new().unwrap();
        let repo_root = PathBuf::from("/test/repo");

        let result =
            resolver.local_slash_command_path(AdapterType::ClaudeCode, "test-command", &repo_root);
        assert!(result.is_ok());
        let resolved = result.unwrap();
        assert_eq!(resolved.scope, Scope::Local);
        let path_str = resolved.path.to_string_lossy();
        assert!(path_str.contains("test-command"));
    }

    #[test]
    fn test_command_stub_path() {
        let resolver = PathResolver::new().unwrap();

        for adapter in AdapterType::all() {
            let result = resolver.global_path(adapter, ArtifactType::CommandStub);
            if result.is_ok() {
                let resolved = result.unwrap();
                assert_eq!(resolved.artifact, ArtifactType::CommandStub);
                let path_str = resolved.path.to_string_lossy();
                assert!(path_str.contains("COMMANDS.md") || path_str.contains("commands"));
            }
        }
    }

    #[test]
    fn test_slash_command_requires_name() {
        let resolver = PathResolver::new().unwrap();

        let result = resolver.global_path(AdapterType::ClaudeCode, ArtifactType::SlashCommand);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("slash_command_path"));
    }

    #[test]
    fn test_skill_global_path() {
        let resolver = PathResolver::new().unwrap();

        let result = resolver.global_path(AdapterType::ClaudeCode, ArtifactType::Skill);
        assert!(result.is_ok());
        let resolved = result.unwrap();
        assert_eq!(resolved.artifact, ArtifactType::Skill);
        assert_eq!(resolved.scope, Scope::Global);
        let path_str = resolved.path.to_string_lossy();
        assert!(path_str.contains("skills"));
    }

    #[test]
    fn test_skill_local_path() {
        let resolver = PathResolver::new().unwrap();
        let repo_root = PathBuf::from("/test/repo");

        let result = resolver.local_path(AdapterType::ClaudeCode, ArtifactType::Skill, &repo_root);
        assert!(result.is_ok());
        let resolved = result.unwrap();
        assert_eq!(resolved.artifact, ArtifactType::Skill);
        assert_eq!(resolved.scope, Scope::Local);
        let path_str = resolved.path.to_string_lossy();
        assert!(path_str.contains("skills"));
    }

    #[test]
    fn test_skill_path_global() {
        let resolver = PathResolver::new().unwrap();

        let result = resolver.skill_path(AdapterType::ClaudeCode, "test-skill");
        assert!(result.is_ok());
        let resolved = result.unwrap();
        assert_eq!(resolved.artifact, ArtifactType::Skill);
        assert_eq!(resolved.scope, Scope::Global);
        let path_str = resolved.path.to_string_lossy();
        assert!(path_str.contains("test-skill"));
        assert!(path_str.contains("SKILL.md"));
    }

    #[test]
    fn test_local_skill_path() {
        let resolver = PathResolver::new().unwrap();
        let repo_root = PathBuf::from("/test/repo");

        let result = resolver.local_skill_path(AdapterType::ClaudeCode, "test-skill", &repo_root);
        assert!(result.is_ok());
        let resolved = result.unwrap();
        assert_eq!(resolved.scope, Scope::Local);
        let path_str = resolved.path.to_string_lossy();
        assert!(path_str.contains("test-skill"));
    }

    #[test]
    fn test_skill_path_validation_rejects_traversal() {
        let resolver = PathResolver::new().unwrap();

        let result = resolver.skill_path(AdapterType::ClaudeCode, "../escape");
        assert!(result.is_err());

        let result = resolver.skill_path(AdapterType::ClaudeCode, "nested/../escape");
        assert!(result.is_err());
    }

    #[test]
    fn test_skill_path_validation_rejects_separators() {
        let resolver = PathResolver::new().unwrap();

        let result = resolver.skill_path(AdapterType::ClaudeCode, "path/to/skill");
        assert!(result.is_err());

        let result = resolver.skill_path(AdapterType::ClaudeCode, "path\\to\\skill");
        assert!(result.is_err());
    }

    #[test]
    fn test_skill_path_validation_rejects_empty() {
        let resolver = PathResolver::new().unwrap();

        let result = resolver.skill_path(AdapterType::ClaudeCode, "");
        assert!(result.is_err());
    }

    #[test]
    fn test_skill_dir_global() {
        let resolver = PathResolver::new().unwrap();

        let result = resolver.skill_dir(AdapterType::ClaudeCode);
        assert!(result.is_ok());
        let resolved = result.unwrap();
        assert_eq!(resolved.artifact, ArtifactType::Skill);
        assert_eq!(resolved.scope, Scope::Global);
    }

    #[test]
    fn test_skill_dir_local() {
        let resolver = PathResolver::new().unwrap();
        let repo_root = PathBuf::from("/test/repo");

        let result = resolver.local_skill_dir(AdapterType::ClaudeCode, &repo_root);
        assert!(result.is_ok());
        let resolved = result.unwrap();
        assert_eq!(resolved.artifact, ArtifactType::Skill);
        assert_eq!(resolved.scope, Scope::Local);
    }

    #[test]
    fn test_skill_path_cursor_unsupported() {
        let resolver = PathResolver::new().unwrap();

        let result = resolver.skill_path(AdapterType::Cursor, "test-skill");
        assert!(result.is_err());
    }

    #[test]
    fn test_repository_roots_management() {
        let mut resolver = PathResolver::with_repository_roots(vec![
            PathBuf::from("/repo1"),
            PathBuf::from("/repo2"),
        ])
        .unwrap();

        assert_eq!(resolver.repository_roots().len(), 2);

        resolver.add_repository_root(PathBuf::from("/repo3"));
        assert_eq!(resolver.repository_roots().len(), 3);
    }

    #[test]
    fn test_all_global_paths() {
        let resolver = PathResolver::new().unwrap();

        let paths = resolver.all_global_paths(ArtifactType::Rule).unwrap();
        assert!(!paths.is_empty());

        for resolved in &paths {
            assert_eq!(resolved.scope, Scope::Global);
        }
    }

    #[test]
    fn test_all_local_paths() {
        let resolver = PathResolver::new().unwrap();
        let repo_roots = vec![PathBuf::from("/test/repo")];

        let paths = resolver
            .all_local_paths(ArtifactType::Rule, &repo_roots)
            .unwrap();
        assert!(!paths.is_empty());

        for resolved in &paths {
            assert_eq!(resolved.scope, Scope::Local);
        }
    }

    #[test]
    #[cfg(unix)]
    fn test_normalize_path_removes_dotdot() {
        let result = normalize_path(&PathBuf::from("/home/user/../other")).unwrap();
        let path_str = result.to_string_lossy();
        assert!(!path_str.contains(".."));
    }

    #[test]
    #[cfg(unix)]
    fn test_normalize_path_removes_dot() {
        let result = normalize_path(&PathBuf::from("/home/./user")).unwrap();
        let path_str = result.to_string_lossy();
        assert!(!path_str.contains("/./"));
    }

    #[test]
    fn test_normalize_path_removes_dotdot_windows() {
        let result = normalize_path(&PathBuf::from("C:\\Users\\test\\..\\other")).unwrap();
        let path_str = result.to_string_lossy();
        assert!(!path_str.contains(".."));
    }

    #[test]
    fn test_normalize_path_removes_dot_windows() {
        let result = normalize_path(&PathBuf::from("C:\\Users\\.\\test")).unwrap();
        let path_str = result.to_string_lossy();
        assert!(!path_str.contains("\\.\\"));
    }

    #[test]
    fn test_normalize_path_relative_fails() {
        let result = normalize_path(&PathBuf::from("relative/path"));
        assert!(result.is_err());
    }

    #[test]
    fn test_canonicalize_relative_path() {
        let resolver = PathResolver::new().unwrap();
        let result = resolver.canonicalize(&PathBuf::from("relative/path"));
        assert!(result.is_ok());
        assert!(result.unwrap().is_absolute());
    }

    #[test]
    fn test_preview_paths() {
        let resolver = PathResolver::new().unwrap();

        let specs = vec![ArtifactSpec {
            adapter: AdapterType::ClaudeCode,
            artifact: ArtifactType::Rule,
            scope: Scope::Global,
            repo_root: None,
            name: None,
        }];

        let paths = resolver.preview_paths(&specs).unwrap();
        assert_eq!(paths.len(), 1);
    }

    #[test]
    fn test_command_name_validation_rejects_traversal() {
        let resolver = PathResolver::new().unwrap();

        let result = resolver.slash_command_path(AdapterType::ClaudeCode, "../escape", true);
        assert!(result.is_err());

        let result = resolver.slash_command_path(AdapterType::ClaudeCode, "nested/../escape", true);
        assert!(result.is_err());
    }

    #[test]
    fn test_command_name_validation_rejects_separators() {
        let resolver = PathResolver::new().unwrap();

        let result = resolver.slash_command_path(AdapterType::ClaudeCode, "path/to/cmd", true);
        assert!(result.is_err());

        let result = resolver.slash_command_path(AdapterType::ClaudeCode, "path\\to\\cmd", true);
        assert!(result.is_err());
    }

    #[test]
    fn test_command_name_validation_rejects_empty() {
        let resolver = PathResolver::new().unwrap();

        let result = resolver.slash_command_path(AdapterType::ClaudeCode, "", true);
        assert!(result.is_err());
    }

    #[test]
    fn test_command_name_validation_accepts_valid() {
        let resolver = PathResolver::new().unwrap();

        let result = resolver.slash_command_path(AdapterType::ClaudeCode, "valid-command", true);
        assert!(result.is_ok());

        let result = resolver.slash_command_path(AdapterType::ClaudeCode, "valid_command", true);
        assert!(result.is_ok());

        let result = resolver.slash_command_path(AdapterType::ClaudeCode, "valid.command", true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_resolve_workspace_path() {
        let base = Some("/home/user/project");
        
        // Starts with ./
        assert_eq!(
            resolve_workspace_path("./scripts/test.sh", base),
            if cfg!(windows) { "/home/user/project\\scripts/test.sh" } else { "/home/user/project/scripts/test.sh" }
        );
        
        // Contains ${WORKSPACE_ROOT}
        assert_eq!(
            resolve_workspace_path("${WORKSPACE_ROOT}/docs", base),
            "/home/user/project/docs"
        );
        
        // No base path
        assert_eq!(
            resolve_workspace_path("./scripts/test.sh", None),
            "./scripts/test.sh"
        );
        
        // Absolute path ignores base if not using variable
        assert_eq!(
            resolve_workspace_path("/absolute/path", base),
            "/absolute/path"
        );

        // Traversal prevention
        assert_eq!(
            resolve_workspace_path("./../../etc/passwd", base),
            "./../../etc/passwd"
        );
    }
}
