use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

use crate::commands::{register_local_rule_paths, storage_location_for_rule, use_file_storage};
use crate::database::Database;
use crate::error::{AppError, Result};
use crate::file_storage;
use crate::models::{
    AdapterType, CreateRuleInput, ImportCandidate, ImportConflict, ImportConflictMode,
    ImportExecutionOptions, ImportExecutionResult, ImportHistoryEntry, ImportScanResult,
    ImportSkip, Rule, Scope, UpdateRuleInput,
};
use crate::sync::SyncEngine;

const DEFAULT_IMPORT_FILE_LIMIT: u64 = 10 * 1024 * 1024;
const IMPORT_SOURCE_MAP_KEY: &str = "import_source_map";
const IMPORT_HISTORY_KEY: &str = "import_history";
const LOCAL_RULE_PATHS_KEY: &str = "local_rule_paths";

#[derive(Debug, Deserialize)]
struct JsonRulePayload {
    name: Option<String>,
    content: Option<String>,
    scope: Option<String>,
    #[serde(rename = "targetPaths")]
    target_paths: Option<Vec<String>>,
    #[serde(rename = "enabledAdapters")]
    enabled_adapters: Option<Vec<String>>,
}

pub async fn scan_url_to_candidates(url: &str, max_size: u64) -> Result<ImportScanResult> {
    let response = reqwest::get(url)
        .await
        .map_err(|e| AppError::InvalidInput {
            message: format!("Failed to fetch URL: {}", e),
        })?;

    if !response.status().is_success() {
        return Err(AppError::InvalidInput {
            message: format!("URL returned non-success status: {}", response.status()),
        });
    }

    let body = response.text().await.map_err(|e| AppError::InvalidInput {
        message: format!("Failed to read URL response body: {}", e),
    })?;

    if body.len() as u64 > max_size {
        return Err(AppError::InvalidInput {
            message: format!("URL content exceeds max size ({} bytes)", max_size),
        });
    }

    let mut scan = ImportScanResult::default();
    let inferred_name = url
        .split('/')
        .next_back()
        .filter(|s| !s.is_empty())
        .unwrap_or("imported-url");
    scan.candidates.push(candidate_from_text(
        body,
        inferred_name,
        crate::models::ImportSourceType::Url,
        "URL",
        url,
        None,
        Scope::Global,
        None,
    ));
    Ok(scan)
}

pub fn scan_clipboard_to_candidates(content: &str, name: Option<&str>) -> ImportScanResult {
    let mut scan = ImportScanResult::default();
    let inferred = name.unwrap_or("clipboard-import");
    scan.candidates.push(candidate_from_text(
        content.to_string(),
        inferred,
        crate::models::ImportSourceType::Clipboard,
        "Clipboard",
        "clipboard",
        None,
        Scope::Global,
        None,
    ));
    scan
}

pub fn scan_file_to_candidates(path: &Path, max_size: u64) -> ImportScanResult {
    let mut scan = ImportScanResult::default();
    match candidate_from_path(
        path,
        crate::models::ImportSourceType::File,
        "File",
        None,
        Scope::Global,
        None,
        max_size,
    ) {
        Ok(candidate) => scan.candidates.push(candidate),
        Err(e) => scan.errors.push(e.to_string()),
    }
    scan
}

pub fn scan_directory_to_candidates(path: &Path, max_size: u64) -> ImportScanResult {
    let mut scan = ImportScanResult::default();
    for entry in WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let item_path = entry.path();
        if !item_path.is_file() {
            continue;
        }

        if !is_supported_import_extension(item_path) {
            continue;
        }

        match candidate_from_path(
            item_path,
            crate::models::ImportSourceType::Directory,
            "Directory",
            None,
            Scope::Global,
            None,
            max_size,
        ) {
            Ok(candidate) => scan.candidates.push(candidate),
            Err(e) => scan.errors.push(e.to_string()),
        }
    }
    scan
}

pub fn scan_ai_tool_candidates(db: &Database, max_size: u64) -> Result<ImportScanResult> {
    let mut scan = ImportScanResult::default();
    let home = dirs::home_dir()
        .ok_or_else(|| AppError::Path("Could not determine home directory".to_string()))?;

    for (adapter, path) in global_tool_paths(&home) {
        if !path.exists() || !path.is_file() {
            continue;
        }

        match candidate_from_path(
            &path,
            crate::models::ImportSourceType::AiTool,
            adapter_label(adapter),
            Some(adapter),
            Scope::Global,
            None,
            max_size,
        ) {
            Ok(candidate) => scan.candidates.push(candidate),
            Err(e) => scan.errors.push(e.to_string()),
        }
    }

    for local_root in get_local_rule_roots(db) {
        for (adapter, relative) in local_tool_paths() {
            let path = local_root.join(relative);
            if !path.exists() || !path.is_file() {
                continue;
            }

            match candidate_from_path(
                &path,
                crate::models::ImportSourceType::AiTool,
                adapter_label(adapter),
                Some(adapter),
                Scope::Local,
                Some(vec![local_root.to_string_lossy().to_string()]),
                max_size,
            ) {
                Ok(candidate) => scan.candidates.push(candidate),
                Err(e) => scan.errors.push(e.to_string()),
            }
        }
    }

    apply_tool_suffix_name_policy(&mut scan.candidates);
    Ok(scan)
}

pub fn execute_import(
    db: &Database,
    scan_result: ImportScanResult,
    options: ImportExecutionOptions,
) -> Result<ImportExecutionResult> {
    let mut result = ImportExecutionResult::default();
    let selected_set = options
        .selected_candidate_ids
        .as_ref()
        .map(|ids| ids.iter().cloned().collect::<HashSet<String>>());
    let mut existing_rules = db.get_all_rules()?;
    let mut source_map = read_source_map(db);

    for candidate in scan_result.candidates {
        if let Some(selected) = selected_set.as_ref() {
            if !selected.contains(&candidate.id) {
                continue;
            }
        }

        let source_key = source_identity(&candidate);
        let effective_scope = options.default_scope.unwrap_or(candidate.scope);
        let effective_adapters = options
            .default_adapters
            .clone()
            .unwrap_or_else(|| candidate.enabled_adapters.clone());

        if candidate.content.trim().is_empty() {
            result.skipped.push(ImportSkip {
                candidate_id: candidate.id.clone(),
                name: candidate.proposed_name.clone(),
                reason: "Content is empty".to_string(),
            });
            continue;
        }

        if let Some(existing_exact) = existing_rules
            .iter()
            .find(|r| compute_content_hash(&r.content) == candidate.content_hash)
        {
            result.skipped.push(ImportSkip {
                candidate_id: candidate.id.clone(),
                name: candidate.proposed_name.clone(),
                reason: format!(
                    "Duplicate content already exists as '{}'",
                    existing_exact.name
                ),
            });
            continue;
        }

        let mapped_rule_id = source_map.get(&source_key).cloned();
        if let Some(rule_id) = mapped_rule_id {
            let update = db.update_rule(
                &rule_id,
                UpdateRuleInput {
                    name: Some(candidate.proposed_name.clone()),
                    content: Some(candidate.content.clone()),
                    scope: Some(effective_scope),
                    target_paths: candidate.target_paths.clone(),
                    enabled_adapters: Some(effective_adapters.clone()),
                    enabled: Some(true),
                },
            )?;

            persist_rule_to_file_if_needed(db, &update)?;
            existing_rules.retain(|r| r.id != update.id);
            existing_rules.push(update.clone());
            result.imported.push(update);
            continue;
        }

        let same_name = existing_rules
            .iter()
            .find(|r| r.name.eq_ignore_ascii_case(&candidate.proposed_name))
            .cloned();

        if let Some(existing_same_name) = same_name {
            if existing_same_name.content == candidate.content {
                result.skipped.push(ImportSkip {
                    candidate_id: candidate.id.clone(),
                    name: candidate.proposed_name.clone(),
                    reason: format!(
                        "Duplicate name and content already exists as '{}'",
                        existing_same_name.name
                    ),
                });
                continue;
            }

            match options.conflict_mode {
                ImportConflictMode::Skip => {
                    result.conflicts.push(ImportConflict {
                        candidate_id: candidate.id.clone(),
                        candidate_name: candidate.proposed_name.clone(),
                        existing_rule_id: Some(existing_same_name.id.clone()),
                        existing_rule_name: Some(existing_same_name.name.clone()),
                        reason: "Name collision with different content".to_string(),
                    });
                    continue;
                }
                ImportConflictMode::Replace => {
                    let update = db.update_rule(
                        &existing_same_name.id,
                        UpdateRuleInput {
                            name: Some(candidate.proposed_name.clone()),
                            content: Some(candidate.content.clone()),
                            scope: Some(effective_scope),
                            target_paths: candidate.target_paths.clone(),
                            enabled_adapters: Some(effective_adapters.clone()),
                            enabled: Some(true),
                        },
                    )?;
                    persist_rule_to_file_if_needed(db, &update)?;
                    source_map.insert(source_key, update.id.clone());
                    existing_rules.retain(|r| r.id != update.id);
                    existing_rules.push(update.clone());
                    result.imported.push(update);
                    continue;
                }
                ImportConflictMode::Rename => {
                    let unique_name = make_unique_name(
                        &candidate.proposed_name,
                        &existing_rules
                            .iter()
                            .map(|r| r.name.clone())
                            .collect::<Vec<_>>(),
                    );

                    let created = db.create_rule(CreateRuleInput {
                        name: unique_name,
                        content: candidate.content.clone(),
                        scope: effective_scope,
                        target_paths: candidate.target_paths.clone(),
                        enabled_adapters: effective_adapters.clone(),
                        enabled: true,
                    })?;
                    persist_rule_to_file_if_needed(db, &created)?;
                    source_map.insert(source_key, created.id.clone());
                    existing_rules.push(created.clone());
                    result.imported.push(created);
                    continue;
                }
            }
        }

        let created = db.create_rule(CreateRuleInput {
            name: candidate.proposed_name.clone(),
            content: candidate.content.clone(),
            scope: effective_scope,
            target_paths: candidate.target_paths.clone(),
            enabled_adapters: effective_adapters,
            enabled: true,
        })?;
        persist_rule_to_file_if_needed(db, &created)?;
        source_map.insert(source_key, created.id.clone());
        existing_rules.push(created.clone());
        result.imported.push(created);
    }

    write_source_map(db, &source_map)?;
    append_history(
        db,
        ImportHistoryEntry {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            source_type: crate::models::ImportSourceType::AiTool,
            imported_count: result.imported.len(),
            skipped_count: result.skipped.len(),
            conflict_count: result.conflicts.len(),
            error_count: result.errors.len() + scan_result.errors.len(),
        },
    )?;

    let engine = SyncEngine::new(db);
    let all_rules = db.get_all_rules()?;
    let sync_res = engine.sync_all(all_rules);
    for err in sync_res.errors {
        result.errors.push(format!(
            "Sync error for {}: {}",
            err.adapter_name, err.message
        ));
    }
    for err in scan_result.errors {
        result.errors.push(err);
    }

    Ok(result)
}

pub fn read_import_history(db: &Database) -> Vec<ImportHistoryEntry> {
    let json = match db.get_setting(IMPORT_HISTORY_KEY) {
        Ok(Some(value)) => value,
        _ => return Vec::new(),
    };
    serde_json::from_str(&json).unwrap_or_default()
}

fn append_history(db: &Database, entry: ImportHistoryEntry) -> Result<()> {
    let mut history = read_import_history(db);
    history.insert(0, entry);
    if history.len() > 50 {
        history.truncate(50);
    }
    let encoded = serde_json::to_string(&history)?;
    db.set_setting(IMPORT_HISTORY_KEY, &encoded)
}

fn persist_rule_to_file_if_needed(db: &Database, rule: &Rule) -> Result<()> {
    if use_file_storage(db) {
        let location = storage_location_for_rule(rule);
        file_storage::save_rule_to_disk(rule, &location)?;
        db.update_rule_file_index(&rule.id, &location)?;
        register_local_rule_paths(db, rule)?;
    }
    Ok(())
}

fn get_local_rule_roots(db: &Database) -> Vec<PathBuf> {
    let roots_json = db
        .get_setting(LOCAL_RULE_PATHS_KEY)
        .ok()
        .flatten()
        .unwrap_or_else(|| "[]".to_string());
    let roots: Vec<String> = serde_json::from_str(&roots_json).unwrap_or_default();
    roots.into_iter().map(PathBuf::from).collect()
}

fn read_source_map(db: &Database) -> HashMap<String, String> {
    let encoded = match db.get_setting(IMPORT_SOURCE_MAP_KEY) {
        Ok(Some(v)) => v,
        _ => return HashMap::new(),
    };
    serde_json::from_str(&encoded).unwrap_or_default()
}

fn write_source_map(db: &Database, map: &HashMap<String, String>) -> Result<()> {
    let encoded = serde_json::to_string(map)?;
    db.set_setting(IMPORT_SOURCE_MAP_KEY, &encoded)
}

fn source_identity(candidate: &ImportCandidate) -> String {
    format!(
        "{}|{}|{}",
        serde_json::to_string(&candidate.source_type).unwrap_or_else(|_| "unknown".to_string()),
        candidate
            .source_tool
            .as_ref()
            .map(|a| a.as_str().to_string())
            .unwrap_or_else(|| "none".to_string()),
        candidate.source_path
    )
}

fn apply_tool_suffix_name_policy(candidates: &mut [ImportCandidate]) {
    let mut groups: HashMap<String, HashSet<AdapterType>> = HashMap::new();
    for candidate in candidates.iter() {
        if let Some(tool) = candidate.source_tool {
            groups
                .entry(candidate.name.to_lowercase())
                .or_default()
                .insert(tool);
        }
    }

    for candidate in candidates.iter_mut() {
        let distinct_tool_count = groups
            .get(&candidate.name.to_lowercase())
            .map(|set| set.len())
            .unwrap_or(0);
        if distinct_tool_count <= 1 {
            continue;
        }
        if let Some(tool) = candidate.source_tool {
            candidate.proposed_name = format!("{}-{}", candidate.name, tool.as_str());
        }
    }
}

fn global_tool_paths(home: &Path) -> Vec<(AdapterType, PathBuf)> {
    vec![
        (AdapterType::Gemini, home.join(".gemini").join("GEMINI.md")),
        (
            AdapterType::OpenCode,
            home.join(".config").join("opencode").join("AGENTS.md"),
        ),
        (AdapterType::Cline, home.join(".clinerules")),
        (
            AdapterType::ClaudeCode,
            home.join(".claude").join("CLAUDE.md"),
        ),
        (AdapterType::Codex, home.join(".codex").join("AGENTS.md")),
        (
            AdapterType::Kilo,
            home.join(".kilocode").join("rules").join("AGENTS.md"),
        ),
        (
            AdapterType::Cursor,
            home.join(".cursor").join("COMMANDS.md"),
        ),
        (
            AdapterType::Windsurf,
            home.join(".windsurf").join("rules").join("AGENTS.md"),
        ),
        (
            AdapterType::RooCode,
            home.join(".roo").join("rules").join("AGENTS.md"),
        ),
        (
            AdapterType::Cursor,
            home.join(".cursor").join(".cursorrules"),
        ),
        (
            AdapterType::Windsurf,
            home.join(".windsurf").join("rules.md"),
        ),
    ]
}

fn local_tool_paths() -> Vec<(AdapterType, &'static str)> {
    vec![
        (AdapterType::Gemini, ".gemini/GEMINI.md"),
        (AdapterType::OpenCode, ".opencode/AGENTS.md"),
        (AdapterType::Cline, ".clinerules"),
        (AdapterType::ClaudeCode, ".claude/CLAUDE.md"),
        (AdapterType::Codex, ".codex/AGENTS.md"),
        (AdapterType::Kilo, ".kilocode/rules/AGENTS.md"),
        (AdapterType::Cursor, ".cursor/COMMANDS.md"),
        (AdapterType::Windsurf, ".windsurf/rules/AGENTS.md"),
        (AdapterType::RooCode, ".roo/rules/AGENTS.md"),
        (AdapterType::Cursor, ".cursorrules"),
        (AdapterType::Windsurf, ".windsurf/rules.md"),
    ]
}

fn adapter_label(adapter: AdapterType) -> &'static str {
    match adapter {
        AdapterType::Antigravity => "Antigravity",
        AdapterType::Gemini => "Gemini",
        AdapterType::OpenCode => "OpenCode",
        AdapterType::Cline => "Cline",
        AdapterType::ClaudeCode => "Claude Code",
        AdapterType::Codex => "Codex",
        AdapterType::Kilo => "Kilo",
        AdapterType::Cursor => "Cursor",
        AdapterType::Windsurf => "Windsurf",
        AdapterType::RooCode => "Roo Code",
    }
}

fn candidate_from_path(
    path: &Path,
    source_type: crate::models::ImportSourceType,
    source_label: &str,
    source_tool: Option<AdapterType>,
    scope: Scope,
    target_paths: Option<Vec<String>>,
    max_size: u64,
) -> Result<ImportCandidate> {
    let metadata = fs::metadata(path)?;
    if metadata.len() > max_size {
        return Err(AppError::InvalidInput {
            message: format!(
                "File '{}' exceeds max import size ({} bytes)",
                path.display(),
                max_size
            ),
        });
    }

    let raw = fs::read(path)?;
    let content = String::from_utf8(raw).map_err(|_| AppError::InvalidInput {
        message: format!("File '{}' is not valid UTF-8", path.display()),
    })?;

    let stem_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("imported-rule");

    let inferred_name = infer_name(path, stem_name, source_tool);
    Ok(candidate_from_text(
        content,
        &inferred_name,
        source_type,
        source_label,
        &path.to_string_lossy(),
        source_tool,
        scope,
        target_paths,
    ))
}

#[allow(clippy::too_many_arguments)]
fn candidate_from_text(
    content: String,
    default_name: &str,
    source_type: crate::models::ImportSourceType,
    source_label: &str,
    source_path: &str,
    source_tool: Option<AdapterType>,
    scope: Scope,
    target_paths: Option<Vec<String>>,
) -> ImportCandidate {
    let (name, parsed_content, parsed_scope, parsed_target_paths, parsed_adapters) =
        extract_rule_payload(default_name, &content, scope, target_paths, source_tool);

    let content_hash = compute_content_hash(&parsed_content);
    ImportCandidate {
        id: uuid::Uuid::new_v4().to_string(),
        source_type,
        source_label: source_label.to_string(),
        source_path: source_path.to_string(),
        source_tool,
        name: name.clone(),
        proposed_name: name,
        content: parsed_content.clone(),
        scope: parsed_scope,
        target_paths: parsed_target_paths,
        enabled_adapters: parsed_adapters,
        content_hash,
        file_size: parsed_content.len() as u64,
    }
}

fn extract_rule_payload(
    fallback_name: &str,
    content: &str,
    fallback_scope: Scope,
    fallback_targets: Option<Vec<String>>,
    source_tool: Option<AdapterType>,
) -> (String, String, Scope, Option<Vec<String>>, Vec<AdapterType>) {
    let trimmed = content.trim().to_string();

    if let Ok(payload) = serde_json::from_str::<JsonRulePayload>(&trimmed) {
        let name = payload
            .name
            .filter(|n| !n.trim().is_empty())
            .unwrap_or_else(|| fallback_name.to_string());
        let body = payload
            .content
            .filter(|c| !c.trim().is_empty())
            .unwrap_or(trimmed.clone());
        let scope = payload
            .scope
            .and_then(|s| Scope::from_str(&s))
            .unwrap_or(fallback_scope);
        let adapters = payload
            .enabled_adapters
            .unwrap_or_default()
            .iter()
            .filter_map(|a| AdapterType::from_str(a))
            .collect::<Vec<_>>();
        return (
            sanitize_rule_name(&name),
            body,
            scope,
            payload.target_paths.or(fallback_targets),
            if adapters.is_empty() {
                default_adapters(source_tool)
            } else {
                adapters
            },
        );
    }

    if let Ok(payload) = serde_yaml::from_str::<JsonRulePayload>(&trimmed) {
        let name = payload
            .name
            .filter(|n| !n.trim().is_empty())
            .unwrap_or_else(|| fallback_name.to_string());
        let body = payload
            .content
            .filter(|c| !c.trim().is_empty())
            .unwrap_or(trimmed.clone());
        let scope = payload
            .scope
            .and_then(|s| Scope::from_str(&s))
            .unwrap_or(fallback_scope);
        let adapters = payload
            .enabled_adapters
            .unwrap_or_default()
            .iter()
            .filter_map(|a| AdapterType::from_str(a))
            .collect::<Vec<_>>();
        return (
            sanitize_rule_name(&name),
            body,
            scope,
            payload.target_paths.or(fallback_targets),
            if adapters.is_empty() {
                default_adapters(source_tool)
            } else {
                adapters
            },
        );
    }

    (
        sanitize_rule_name(fallback_name),
        trimmed,
        fallback_scope,
        fallback_targets,
        default_adapters(source_tool),
    )
}

fn infer_name(path: &Path, fallback: &str, source_tool: Option<AdapterType>) -> String {
    let normalized = fallback.to_ascii_lowercase();
    if [
        "agents",
        "commands",
        "gemini",
        "claude",
        "rules",
        ".clinerules",
        ".cursorrules",
    ]
    .contains(&normalized.as_str())
    {
        if let Some(tool) = source_tool {
            return sanitize_rule_name(&format!("{}-import", tool.as_str()));
        }
    }
    sanitize_rule_name(
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(fallback),
    )
}

fn default_adapters(source_tool: Option<AdapterType>) -> Vec<AdapterType> {
    match source_tool {
        Some(tool) => vec![tool],
        None => vec![AdapterType::Gemini, AdapterType::OpenCode],
    }
}

fn sanitize_rule_name(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == ' ' {
            out.push(ch);
        }
    }
    let compact = out.split_whitespace().collect::<Vec<_>>().join("-");
    if compact.is_empty() {
        "imported-rule".to_string()
    } else {
        compact
    }
}

fn make_unique_name(base: &str, existing_names: &[String]) -> String {
    if !existing_names.iter().any(|n| n.eq_ignore_ascii_case(base)) {
        return base.to_string();
    }

    let mut index = 2usize;
    loop {
        let candidate = format!("{}-{}", base, index);
        if !existing_names
            .iter()
            .any(|n| n.eq_ignore_ascii_case(&candidate))
        {
            return candidate;
        }
        index += 1;
    }
}

fn is_supported_import_extension(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("md") | Some("txt") | Some("json") | Some("yaml") | Some("yml")
    )
}

fn compute_content_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn resolve_max_size(options: &ImportExecutionOptions) -> u64 {
    options
        .max_file_size_bytes
        .unwrap_or(DEFAULT_IMPORT_FILE_LIMIT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::Database;
    use crate::models::CreateRuleInput;

    #[test]
    fn unique_name_generation_is_stable() {
        let existing = vec![
            "quality".to_string(),
            "quality-2".to_string(),
            "quality-3".to_string(),
        ];
        assert_eq!(make_unique_name("quality", &existing), "quality-4");
    }

    #[test]
    fn suffix_policy_applies_for_multi_tool_name_collision() {
        let content = "test content".to_string();
        let mut candidates = vec![
            candidate_from_text(
                content.clone(),
                "quality",
                crate::models::ImportSourceType::AiTool,
                "Cline",
                "a",
                Some(AdapterType::Cline),
                Scope::Global,
                None,
            ),
            candidate_from_text(
                content,
                "quality",
                crate::models::ImportSourceType::AiTool,
                "Antigravity",
                "b",
                Some(AdapterType::Antigravity),
                Scope::Global,
                None,
            ),
        ];

        apply_tool_suffix_name_policy(&mut candidates);

        assert!(candidates
            .iter()
            .any(|c| c.proposed_name == "quality-cline"));
        assert!(candidates
            .iter()
            .any(|c| c.proposed_name == "quality-antigravity"));
    }

    #[test]
    fn execute_import_skips_duplicate_content() {
        let db = Database::new_in_memory().expect("in-memory db");
        db.create_rule(CreateRuleInput {
            name: "Existing".to_string(),
            content: "same-content".to_string(),
            scope: Scope::Global,
            target_paths: None,
            enabled_adapters: vec![AdapterType::Gemini],
            enabled: true,
        })
        .expect("seed rule");

        let candidate = candidate_from_text(
            "same-content".to_string(),
            "Imported",
            crate::models::ImportSourceType::File,
            "File",
            "C:/tmp/x.md",
            None,
            Scope::Global,
            None,
        );

        let result = execute_import(
            &db,
            ImportScanResult {
                candidates: vec![candidate],
                errors: vec![],
            },
            ImportExecutionOptions::default(),
        )
        .expect("execute import");

        assert_eq!(result.imported.len(), 0);
        assert_eq!(result.skipped.len(), 1);
    }

    #[test]
    fn execute_import_rename_mode_creates_unique_name() {
        let db = Database::new_in_memory().expect("in-memory db");
        db.create_rule(CreateRuleInput {
            name: "quality".to_string(),
            content: "old".to_string(),
            scope: Scope::Global,
            target_paths: None,
            enabled_adapters: vec![AdapterType::Gemini],
            enabled: true,
        })
        .expect("seed rule");

        let candidate = candidate_from_text(
            "new-content".to_string(),
            "quality",
            crate::models::ImportSourceType::File,
            "File",
            "C:/tmp/y.md",
            None,
            Scope::Global,
            None,
        );

        let result = execute_import(
            &db,
            ImportScanResult {
                candidates: vec![candidate],
                errors: vec![],
            },
            ImportExecutionOptions {
                conflict_mode: ImportConflictMode::Rename,
                ..Default::default()
            },
        )
        .expect("execute import");

        assert_eq!(result.imported.len(), 1);
        assert_eq!(result.imported[0].name, "quality-2");
    }

    #[test]
    fn execute_import_replace_mode_updates_existing_rule() {
        let db = Database::new_in_memory().expect("in-memory db");
        let existing = db
            .create_rule(CreateRuleInput {
                name: "policy".to_string(),
                content: "old".to_string(),
                scope: Scope::Global,
                target_paths: None,
                enabled_adapters: vec![AdapterType::Gemini],
                enabled: true,
            })
            .expect("seed rule");

        let candidate = candidate_from_text(
            "updated".to_string(),
            "policy",
            crate::models::ImportSourceType::File,
            "File",
            "C:/tmp/z.md",
            None,
            Scope::Global,
            None,
        );

        let result = execute_import(
            &db,
            ImportScanResult {
                candidates: vec![candidate],
                errors: vec![],
            },
            ImportExecutionOptions {
                conflict_mode: ImportConflictMode::Replace,
                ..Default::default()
            },
        )
        .expect("execute import");

        assert_eq!(result.imported.len(), 1);
        assert_eq!(result.imported[0].id, existing.id);
        assert_eq!(result.imported[0].content, "updated");
    }

    #[test]
    fn extract_payload_reads_json_rule_fields() {
        let json = r#"{
          "name": "json-rule",
          "content": "json body",
          "scope": "local",
          "targetPaths": ["C:/repo"],
          "enabledAdapters": ["gemini", "opencode"]
        }"#;

        let (name, content, scope, target_paths, adapters) =
            extract_rule_payload("fallback", json, Scope::Global, None, None);

        assert_eq!(name, "json-rule");
        assert_eq!(content, "json body");
        assert_eq!(scope, Scope::Local);
        assert_eq!(target_paths, Some(vec!["C:/repo".to_string()]));
        assert_eq!(adapters.len(), 2);
    }

    #[test]
    fn extract_payload_reads_yaml_rule_fields() {
        let yaml = r#"
name: yaml-rule
content: yaml body
scope: global
enabledAdapters:
  - cline
"#;

        let (name, content, scope, _target_paths, adapters) =
            extract_rule_payload("fallback", yaml, Scope::Local, Some(vec!["x".to_string()]), None);

        assert_eq!(name, "yaml-rule");
        assert_eq!(content, "yaml body");
        assert_eq!(scope, Scope::Global);
        assert_eq!(adapters, vec![AdapterType::Cline]);
    }

    #[test]
    fn execute_import_reimport_updates_mapped_rule_idempotently() {
        let db = Database::new_in_memory().expect("in-memory db");

        let first_candidate = candidate_from_text(
            "original content".to_string(),
            "shared-rule",
            crate::models::ImportSourceType::File,
            "File",
            "C:/tmp/shared-rule.md",
            None,
            Scope::Global,
            None,
        );

        let first_result = execute_import(
            &db,
            ImportScanResult {
                candidates: vec![first_candidate],
                errors: vec![],
            },
            ImportExecutionOptions::default(),
        )
        .expect("first import");

        assert_eq!(first_result.imported.len(), 1);
        let imported_id = first_result.imported[0].id.clone();

        let second_candidate = candidate_from_text(
            "updated content".to_string(),
            "shared-rule",
            crate::models::ImportSourceType::File,
            "File",
            "C:/tmp/shared-rule.md",
            None,
            Scope::Global,
            None,
        );

        let second_result = execute_import(
            &db,
            ImportScanResult {
                candidates: vec![second_candidate],
                errors: vec![],
            },
            ImportExecutionOptions::default(),
        )
        .expect("second import");

        assert_eq!(second_result.imported.len(), 1);
        assert_eq!(second_result.imported[0].id, imported_id);
        assert_eq!(second_result.imported[0].content, "updated content");
    }
}
