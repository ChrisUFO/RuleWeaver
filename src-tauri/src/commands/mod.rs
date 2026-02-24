pub mod adapters;
pub mod command_commands;
pub mod import_commands;
pub mod mcp_commands;
pub mod migration_commands;
pub mod rule_commands;
pub mod skill_commands;
pub mod system_commands;

use adapters::{
    ClaudeAdapter, CommandAdapter, CursorAdapter, GeminiAdapter, KiloAdapter, OpenCodeAdapter,
    RooCodeAdapter, WindsurfAdapter,
};
pub use command_commands::*;
pub use import_commands::*;
pub use mcp_commands::*;
pub use migration_commands::*;
pub use rule_commands::*;
pub use skill_commands::*;
pub use system_commands::*;

use parking_lot::Mutex;
use std::collections::{HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};
use std::sync::LazyLock;
use std::time::Instant;

use crate::constants::limits::{
    MAX_COMMAND_NAME_LENGTH, MAX_COMMAND_SCRIPT_LENGTH, MAX_RULE_CONTENT_LENGTH,
    MAX_RULE_NAME_LENGTH,
};
use crate::constants::{
    NEW_CURSOR_DIR, NEW_GEMINI_DIR, NEW_KILO_DIR, NEW_ROO_CODE_DIR,
    NEW_WINDSURF_DIR,
};
use crate::database::Database;
use crate::error::{AppError, Result};
use crate::file_storage;
use crate::models::Rule;

pub static RUNNING_TESTS: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
pub static TEST_INVOCATION_TIMESTAMPS: LazyLock<Mutex<VecDeque<Instant>>> =
    LazyLock::new(|| Mutex::new(VecDeque::new()));

pub fn validate_path(path: &str) -> Result<PathBuf> {
    let p = PathBuf::from(path);

    // Check for traversal components before canonicalization for defense-in-depth
    if path.contains("..") {
        return Err(AppError::InvalidInput {
            message: "Path cannot contain traversal sequences (..)".to_string(),
        });
    }

    let canonical_path = std::fs::canonicalize(&p).map_err(|e| AppError::InvalidInput {
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

pub fn validate_rule_input(name: &str, content: &str) -> Result<()> {
    let trimmed_name = name.trim();
    if trimmed_name.is_empty() {
        return Err(AppError::Validation(
            "Rule name cannot be empty".to_string(),
        ));
    }
    if trimmed_name.len() > MAX_RULE_NAME_LENGTH {
        return Err(AppError::Validation(format!(
            "Rule name too long (max {} characters)",
            MAX_RULE_NAME_LENGTH
        )));
    }
    if content.len() > MAX_RULE_CONTENT_LENGTH {
        return Err(AppError::Validation(format!(
            "Rule content too large (max {} characters)",
            MAX_RULE_CONTENT_LENGTH
        )));
    }
    Ok(())
}

pub fn validate_command_input(name: &str, script: &str) -> Result<()> {
    let trimmed_name = name.trim();
    if trimmed_name.is_empty() {
        return Err(AppError::Validation(
            "Command name cannot be empty".to_string(),
        ));
    }
    if trimmed_name.len() > MAX_COMMAND_NAME_LENGTH {
        return Err(AppError::Validation(format!(
            "Command name too long (max {} characters)",
            MAX_COMMAND_NAME_LENGTH
        )));
    }
    if script.trim().is_empty() {
        return Err(AppError::Validation(
            "Command script cannot be empty".to_string(),
        ));
    }
    if script.len() > MAX_COMMAND_SCRIPT_LENGTH {
        return Err(AppError::Validation(format!(
            "Command script too long (max {} characters)",
            MAX_COMMAND_SCRIPT_LENGTH
        )));
    }
    Ok(())
}

pub fn validate_command_arguments(args: &[crate::models::CommandArgument]) -> Result<()> {
    for arg in args {
        if arg.name.trim().is_empty() {
            return Err(AppError::Validation(
                "Argument name cannot be empty".to_string(),
            ));
        }

        if matches!(arg.arg_type, crate::models::ArgumentType::Enum) {
            match &arg.options {
                Some(options) => {
                    if options.is_empty() {
                        return Err(AppError::Validation(format!(
                            "Enum argument '{}' must have at least one option",
                            arg.name
                        )));
                    }
                    let mut seen = std::collections::HashSet::new();
                    for opt in options {
                        if opt.trim().is_empty() {
                            return Err(AppError::Validation(format!(
                                "Enum argument '{}' contains an empty option",
                                arg.name
                            )));
                        }
                        if !seen.insert(opt) {
                            return Err(AppError::Validation(format!(
                                "Enum argument '{}' contains duplicate option: {}",
                                arg.name, opt
                            )));
                        }
                    }
                }
                None => {
                    return Err(AppError::Validation(format!(
                        "Enum argument '{}' must have options defined",
                        arg.name
                    )));
                }
            }
        }
    }
    Ok(())
}

pub fn markdown_escape_inline(input: &str) -> String {
    input.replace('`', "\\`")
}

fn command_adapters() -> &'static Vec<Arc<dyn CommandAdapter>> {
    static COMMAND_ADAPTERS: OnceLock<Vec<Arc<dyn CommandAdapter>>> = OnceLock::new();
    COMMAND_ADAPTERS.get_or_init(|| {
        vec![
            Arc::new(GeminiAdapter),
            Arc::new(OpenCodeAdapter),
            Arc::new(ClaudeAdapter),
            Arc::new(KiloAdapter),
            Arc::new(CursorAdapter),
            Arc::new(WindsurfAdapter),
            Arc::new(RooCodeAdapter),
        ]
    })
}

fn command_target_path_for_adapter(root: &Path, adapter_name: &str) -> Option<String> {
    let path = match adapter_name {
        "gemini" => root.join(NEW_GEMINI_DIR).join("COMMANDS.toml"),
        "opencode" => root.join(".opencode").join("COMMANDS.md"),
        "claude" => root.join(".claude").join("COMMANDS.md"),
        "kilo" => root.join(NEW_KILO_DIR).join("rules").join("COMMANDS.md"),
        "cursor" => root.join(NEW_CURSOR_DIR).join("COMMANDS.md"),
        "windsurf" => root.join(NEW_WINDSURF_DIR).join("rules").join("COMMANDS.md"),
        "roo" => root.join(NEW_ROO_CODE_DIR).join("COMMANDS.md"),
        _ => return None,
    };
    Some(path.to_string_lossy().to_string())
}

pub fn command_file_targets() -> Result<Vec<(String, Arc<dyn CommandAdapter>)>> {
    let home = dirs::home_dir()
        .ok_or_else(|| AppError::Path("Could not determine home directory".to_string()))?;
    let mut targets = Vec::new();
    for adapter in command_adapters() {
        if let Some(path) = command_target_path_for_adapter(&home, adapter.name()) {
            targets.push((path, Arc::clone(adapter)));
        }
    }
    Ok(targets)
}

pub fn use_file_storage(db: &Database) -> bool {
    db.get_storage_mode()
        .map(|mode| mode == "file")
        .unwrap_or(false)
}

pub const LOCAL_RULE_PATHS_KEY: &str = "local_rule_paths";

pub fn get_local_rule_roots(db: &Database) -> Result<Vec<PathBuf>> {
    let roots_json = db
        .get_setting(LOCAL_RULE_PATHS_KEY)?
        .unwrap_or_else(|| "[]".to_string());
    let roots: Vec<String> = serde_json::from_str(&roots_json)?;
    Ok(roots.into_iter().map(PathBuf::from).collect())
}

pub fn register_local_rule_paths(db: &Database, rule: &Rule) -> Result<()> {
    if !matches!(rule.scope, crate::models::Scope::Local) {
        return Ok(());
    }

    let paths = rule.target_paths.clone().unwrap_or_default();
    register_local_paths(db, &paths)
}

pub fn command_file_targets_for_root(root: &Path) -> Vec<(String, Arc<dyn CommandAdapter>)> {
    let mut targets = Vec::new();
    for adapter in command_adapters() {
        if let Some(path) = command_target_path_for_adapter(root, adapter.name()) {
            targets.push((path, Arc::clone(adapter)));
        }
    }
    targets
}

pub fn register_local_paths(db: &Database, paths: &[String]) -> Result<()> {
    db.merge_setting_string_array_unique(LOCAL_RULE_PATHS_KEY, paths)
}

pub fn validate_paths_within_registered_roots(db: &Database, paths: &[String]) -> Result<()> {
    if paths.is_empty() {
        return Ok(());
    }

    let roots = get_local_rule_roots(db)?;
    if roots.is_empty() {
        return Err(AppError::InvalidInput {
            message: "No repository roots configured. Add roots in Settings first.".to_string(),
        });
    }

    let canonical_roots = roots
        .iter()
        .filter_map(|root| std::fs::canonicalize(root).ok())
        .collect::<Vec<_>>();

    for path in paths {
        let canonical = validate_path(path)?;
        let in_roots = canonical_roots.iter().any(|root| canonical.starts_with(root));
        if !in_roots {
            return Err(AppError::InvalidInput {
                message: format!(
                    "Target path '{}' is not inside configured repository roots",
                    path
                ),
            });
        }
    }

    Ok(())
}

pub fn storage_location_for_rule(rule: &Rule) -> file_storage::StorageLocation {
    match rule.scope {
        crate::models::Scope::Global => file_storage::StorageLocation::Global,
        crate::models::Scope::Local => {
            if let Some(paths) = &rule.target_paths {
                if let Some(first) = paths.first() {
                    return file_storage::StorageLocation::Local(PathBuf::from(first));
                }
            }
            file_storage::StorageLocation::Global
        }
    }
}

pub fn validate_local_rule_paths(
    db: &Database,
    id: Option<&str>,
    scope: Option<crate::models::Scope>,
    target_paths: &Option<Vec<String>>,
) -> Result<()> {
    let final_scope = if let Some(s) = scope {
        s
    } else if let Some(rule_id) = id {
        db.get_rule_by_id(rule_id)?.scope
    } else {
        return Ok(());
    };

    if matches!(final_scope, crate::models::Scope::Local) {
        if let Some(ref paths) = target_paths {
            for path in paths {
                validate_path(path)?;
            }
        } else if let Some(rule_id) = id {
            let existing = db.get_rule_by_id(rule_id)?;
            if let Some(ref paths) = existing.target_paths {
                for path in paths {
                    validate_path(path)?;
                }
            }
        }

        let paths_exist = if let Some(p) = target_paths {
            !p.is_empty()
        } else if let Some(rule_id) = id {
            let existing = db.get_rule_by_id(rule_id)?;
            existing
                .target_paths
                .as_ref()
                .map(|p| !p.is_empty())
                .unwrap_or(false)
        } else {
            false
        };

        if !paths_exist {
            return Err(AppError::InvalidInput {
                message: "Local rules must have at least one target path".to_string(),
            });
        }
    }
    Ok(())
}
