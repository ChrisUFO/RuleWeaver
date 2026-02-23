mod migration;
mod parser;
mod serializer;
pub mod skills;
pub mod watcher;

#[allow(unused_imports)]
pub use migration::{
    get_migration_progress, migrate_to_file_storage, rollback_migration, verify_migration,
    MigrationError, MigrationProgress, MigrationResult, MigrationStatus, VerificationResult,
};
#[allow(unused_imports)]
pub use parser::{parse_rule_file, ParsedRuleFile, RuleFrontmatter};
#[allow(unused_imports)]
pub use serializer::{generate_filename, generate_rule_file_path, serialize_rule_to_file_content};
#[allow(unused_imports)]
pub use watcher::{FileChangeEvent, RuleFileWatcher};

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::database::Database;
use crate::error::{AppError, Result};
use crate::models::Rule;

pub const RULES_DIR_NAME: &str = "rules";
pub const RULEWEAVER_DIR_NAME: &str = ".ruleweaver";

pub fn get_global_rules_dir() -> Result<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| AppError::Path("Could not determine home directory".to_string()))?;
    Ok(home.join(RULEWEAVER_DIR_NAME).join(RULES_DIR_NAME))
}

pub fn get_local_rules_dir(project_path: &Path) -> PathBuf {
    project_path.join(RULEWEAVER_DIR_NAME).join(RULES_DIR_NAME)
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RuleLoadResult {
    pub rules: Vec<Rule>,
    pub errors: Vec<RuleLoadError>,
    pub files_scanned: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RuleLoadError {
    pub file_path: String,
    pub error: String,
}

#[derive(Debug, Clone)]
pub enum StorageLocation {
    Global,
    Local(PathBuf),
}

pub fn load_rules_from_disk() -> Result<RuleLoadResult> {
    load_rules_from_locations(&[])
}

pub fn load_rules_from_locations(local_roots: &[PathBuf]) -> Result<RuleLoadResult> {
    let mut all_rules = Vec::new();
    let mut all_errors = Vec::new();
    let mut files_scanned = 0u32;

    let global_dir = get_global_rules_dir()?;
    if global_dir.exists() {
        let (rules, errors, count) = load_rules_from_directory(&global_dir)?;
        all_rules.extend(rules);
        all_errors.extend(errors);
        files_scanned += count;
    }

    for root in local_roots {
        let local_dir = get_local_rules_dir(root);
        if local_dir.exists() {
            let (rules, errors, count) = load_rules_from_directory(&local_dir)?;
            all_rules.extend(rules);
            all_errors.extend(errors);
            files_scanned += count;
        }
    }

    let result = RuleLoadResult {
        rules: all_rules,
        errors: all_errors,
        files_scanned,
    };

    Ok(result)
}

pub fn load_rules_from_directory(dir: &Path) -> Result<(Vec<Rule>, Vec<RuleLoadError>, u32)> {
    let mut rules = Vec::new();
    let mut errors = Vec::new();
    let mut files_scanned = 0u32;

    if !dir.exists() {
        return Ok((rules, errors, files_scanned));
    }

    for entry in WalkDir::new(dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let extension = path.extension().and_then(|e| e.to_str());
        if extension != Some("md") {
            continue;
        }

        files_scanned += 1;

        match load_rule_from_file(path) {
            Ok(rule) => rules.push(rule),
            Err(e) => errors.push(RuleLoadError {
                file_path: path.to_string_lossy().to_string(),
                error: e.to_string(),
            }),
        }
    }

    Ok((rules, errors, files_scanned))
}

pub fn load_rule_from_file(path: &Path) -> Result<Rule> {
    let content = fs::read_to_string(path).map_err(|e| AppError::InvalidInput {
        message: format!("Failed to read file '{}': {}", path.display(), e),
    })?;

    let parsed = parse_rule_file(path, &content)?;
    parsed.to_rule()
}

pub fn save_rule_to_disk(rule: &Rule, location: &StorageLocation) -> Result<PathBuf> {
    let base_dir = match location {
        StorageLocation::Global => get_global_rules_dir()?,
        StorageLocation::Local(project_path) => get_local_rules_dir(project_path),
    };

    fs::create_dir_all(&base_dir)?;

    let file_content = serialize_rule_to_file_content(rule)?;

    let file_path = find_or_create_rule_file(&base_dir, rule)?;

    let temp_path = file_path.with_extension(format!("md.tmp-{}", uuid::Uuid::new_v4()));
    {
        let mut file = fs::File::create(&temp_path)?;
        file.write_all(file_content.as_bytes())?;
        file.sync_all()?;
    }

    fs::rename(&temp_path, &file_path)?;

    // Ensure directory metadata is also synced to disk if possible
    if let Some(parent) = file_path.parent() {
        if let Ok(dir) = fs::File::open(parent) {
            let _ = dir.sync_all();
        }
    }

    Ok(file_path)
}

fn find_or_create_rule_file(base_dir: &Path, rule: &Rule) -> Result<PathBuf> {
    for entry in WalkDir::new(base_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }

        match fs::read_to_string(path) {
            Ok(content) => {
                if let Ok(parsed) = parse_rule_file(path, &content) {
                    if parsed.frontmatter.id == rule.id {
                        return Ok(path.to_path_buf());
                    }
                }
            }
            Err(_) => continue,
        }
    }

    Ok(generate_rule_file_path(base_dir, rule))
}

pub fn delete_rule_file(
    rule_id: &str,
    location: &StorageLocation,
    db: Option<&Database>,
) -> Result<bool> {
    // Optimization: try to use database index first
    if let Some(db) = db {
        if let Ok(Some(path_str)) = db.get_rule_file_path(rule_id) {
            let path = PathBuf::from(path_str);
            if path.exists() {
                fs::remove_file(path)?;
                return Ok(true);
            }
        }
    }

    let base_dir = match location {
        StorageLocation::Global => get_global_rules_dir()?,
        StorageLocation::Local(project_path) => get_local_rules_dir(project_path),
    };

    if !base_dir.exists() {
        return Ok(false);
    }

    for entry in WalkDir::new(&base_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }

        match fs::read_to_string(path) {
            Ok(content) => {
                if let Ok(parsed) = parse_rule_file(path, &content) {
                    if parsed.frontmatter.id == rule_id {
                        fs::remove_file(path)?;
                        return Ok(true);
                    }
                }
            }
            Err(_) => continue,
        }
    }

    Ok(false)
}

#[allow(dead_code)]
pub fn get_rule_file_path(
    rule_id: &str,
    location: &StorageLocation,
    db: Option<&Database>,
) -> Result<Option<PathBuf>> {
    // Optimization: try to use database index first
    if let Some(db) = db {
        if let Ok(Some(path_str)) = db.get_rule_file_path(rule_id) {
            let path = PathBuf::from(path_str);
            if path.exists() {
                return Ok(Some(path));
            }
        }
    }

    let base_dir = match location {
        StorageLocation::Global => get_global_rules_dir()?,
        StorageLocation::Local(project_path) => get_local_rules_dir(project_path),
    };

    if !base_dir.exists() {
        return Ok(None);
    }

    for entry in WalkDir::new(&base_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }

        match fs::read_to_string(path) {
            Ok(content) => {
                if let Ok(parsed) = parse_rule_file(path, &content) {
                    if parsed.frontmatter.id == rule_id {
                        return Ok(Some(path.to_path_buf()));
                    }
                }
            }
            Err(_) => continue,
        }
    }

    Ok(None)
}

#[allow(dead_code)]
pub fn storage_exists() -> bool {
    get_global_rules_dir()
        .map(|dir| dir.exists())
        .unwrap_or(false)
}

pub fn get_storage_info() -> Result<StorageInfo> {
    let global_dir = get_global_rules_dir()?;

    let exists = global_dir.exists();
    let mut rule_count = 0u32;
    let mut total_size = 0u64;

    if exists {
        for entry in WalkDir::new(&global_dir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("md") {
                rule_count += 1;
                if let Ok(metadata) = fs::metadata(path) {
                    total_size += metadata.len();
                }
            }
        }
    }

    Ok(StorageInfo {
        global_dir,
        exists,
        rule_count,
        total_size_bytes: total_size,
    })
}

#[derive(Debug, Clone)]
pub struct StorageInfo {
    pub global_dir: PathBuf,
    pub exists: bool,
    pub rule_count: u32,
    pub total_size_bytes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Scope;
    use std::fs;
    use std::path::PathBuf;

    fn create_temp_test_dir() -> PathBuf {
        let temp_dir =
            std::env::temp_dir().join(format!("ruleweaver_test_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");
        temp_dir
    }

    fn cleanup_temp_dir(dir: &Path) {
        let _ = fs::remove_dir_all(dir);
    }

    fn create_test_rule_content(id: &str, name: &str, body: &str) -> String {
        format!(
            "---\nid: {}\nname: {}\nscope: global\nenabledAdapters: [gemini]\ncreatedAt: 2024-01-15T10:30:00Z\nupdatedAt: 2024-01-15T10:30:00Z\n---\n{}\n",
            id, name, body
        )
    }

    #[test]
    fn test_load_rules_from_empty_directory() {
        let temp_dir = create_temp_test_dir();
        let rules_dir = temp_dir.join(".ruleweaver").join("rules");
        fs::create_dir_all(&rules_dir).expect("Failed to create rules dir");

        let (rules, errors, count) = load_rules_from_directory(&rules_dir).unwrap();

        assert!(rules.is_empty());
        assert!(errors.is_empty());
        assert_eq!(count, 0);

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_load_rules_from_directory_with_valid_files() {
        let temp_dir = create_temp_test_dir();
        let rules_dir = temp_dir.join(".ruleweaver").join("rules");
        fs::create_dir_all(&rules_dir).expect("Failed to create rules dir");

        let rule1_content = create_test_rule_content("rule-1", "First Rule", "Content 1");
        let rule2_content = create_test_rule_content("rule-2", "Second Rule", "Content 2");

        fs::write(rules_dir.join("rule1.md"), &rule1_content).expect("Failed to write rule1");
        fs::write(rules_dir.join("rule2.md"), &rule2_content).expect("Failed to write rule2");

        let (rules, errors, count) = load_rules_from_directory(&rules_dir).unwrap();

        assert_eq!(rules.len(), 2);
        assert!(errors.is_empty());
        assert_eq!(count, 2);

        let ids: Vec<&str> = rules.iter().map(|r| r.id.as_str()).collect();
        assert!(ids.contains(&"rule-1"));
        assert!(ids.contains(&"rule-2"));

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_load_rules_from_directory_with_invalid_file() {
        let temp_dir = create_temp_test_dir();
        let rules_dir = temp_dir.join(".ruleweaver").join("rules");
        fs::create_dir_all(&rules_dir).expect("Failed to create rules dir");

        let valid_content = create_test_rule_content("valid-rule", "Valid Rule", "Valid content");
        let invalid_content = "This is not valid frontmatter";

        fs::write(rules_dir.join("valid.md"), &valid_content).expect("Failed to write valid");
        fs::write(rules_dir.join("invalid.md"), invalid_content).expect("Failed to write invalid");

        let (rules, errors, count) = load_rules_from_directory(&rules_dir).unwrap();

        assert_eq!(rules.len(), 1);
        assert_eq!(errors.len(), 1);
        assert_eq!(count, 2);
        assert_eq!(rules[0].id, "valid-rule");
        assert!(errors[0].file_path.contains("invalid.md"));

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_load_rules_ignores_non_markdown_files() {
        let temp_dir = create_temp_test_dir();
        let rules_dir = temp_dir.join(".ruleweaver").join("rules");
        fs::create_dir_all(&rules_dir).expect("Failed to create rules dir");

        let rule_content = create_test_rule_content("only-rule", "Only Rule", "Content");

        fs::write(rules_dir.join("rule.md"), &rule_content).expect("Failed to write rule");
        fs::write(rules_dir.join("readme.txt"), "Not a rule file").expect("Failed to write txt");
        fs::write(rules_dir.join("data.json"), "{}").expect("Failed to write json");

        let (rules, errors, count) = load_rules_from_directory(&rules_dir).unwrap();

        assert_eq!(rules.len(), 1);
        assert!(errors.is_empty());
        assert_eq!(count, 1);

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_save_rule_creates_new_file() {
        let temp_dir = create_temp_test_dir();

        let rule = Rule {
            id: "new-rule-id".to_string(),
            name: "New Test Rule".to_string(),
            content: "New content".to_string(),
            scope: Scope::Global,
            target_paths: None,
            enabled_adapters: vec![crate::models::AdapterType::Gemini],
            enabled: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let result = save_rule_to_disk(&rule, &StorageLocation::Local(temp_dir.clone()));

        assert!(result.is_ok());
        let saved_path = result.unwrap();
        assert!(saved_path.exists());
        assert!(saved_path.to_string_lossy().ends_with(".md"));

        let loaded = load_rule_from_file(&saved_path).unwrap();
        assert_eq!(loaded.id, "new-rule-id");
        assert_eq!(loaded.name, "New Test Rule");

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_delete_rule_file() {
        let temp_dir = create_temp_test_dir();
        let rules_dir = temp_dir.join(".ruleweaver").join("rules");
        fs::create_dir_all(&rules_dir).expect("Failed to create rules dir");

        let rule_content = create_test_rule_content("to-delete", "Rule to Delete", "Content");
        let rule_path = rules_dir.join("delete-me.md");
        fs::write(&rule_path, &rule_content).expect("Failed to write rule");

        assert!(rule_path.exists());

        let deleted =
            delete_rule_file("to-delete", &StorageLocation::Local(temp_dir.clone()), None).unwrap();
        assert!(deleted);
        assert!(!rule_path.exists());

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_delete_rule_file_not_found() {
        let temp_dir = create_temp_test_dir();
        let result = delete_rule_file(
            "nonexistent",
            &StorageLocation::Local(temp_dir.clone()),
            None,
        )
        .unwrap();
        assert!(!result);
        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_generate_filename_sanitization() {
        let rule = Rule {
            id: "test".to_string(),
            name: "Test @#$% Rule!!!".to_string(),
            content: String::new(),
            scope: Scope::Global,
            target_paths: None,
            enabled_adapters: vec![crate::models::AdapterType::Gemini],
            enabled: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let filename = generate_filename(&rule);
        assert!(!filename.contains('@'));
        assert!(!filename.contains('#'));
        assert!(!filename.contains('!'));
        assert!(filename.ends_with(".md"));
    }

    #[test]
    fn test_load_rules_from_locations_includes_local_roots() {
        let temp_dir = create_temp_test_dir();
        let local_root = temp_dir.join("repo-a");
        let local_rules_dir = local_root.join(".ruleweaver").join("rules");
        fs::create_dir_all(&local_rules_dir).expect("Failed to create local rules dir");

        let local_rule_content =
            create_test_rule_content("local-rule-1", "Local Rule", "Local Content");
        fs::write(local_rules_dir.join("local-rule.md"), &local_rule_content)
            .expect("Failed to write local rule");

        let result = load_rules_from_locations(&[local_root.clone()]).unwrap();

        assert!(result.rules.iter().any(|r| r.id == "local-rule-1"));
        assert!(result.files_scanned >= 1);

        cleanup_temp_dir(&temp_dir);
    }
}
