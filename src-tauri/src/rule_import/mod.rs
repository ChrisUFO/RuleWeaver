use std::collections::{HashMap, HashSet};
use std::fs;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use chrono::Utc;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

use crate::commands::{
    reconcile_after_mutation, register_local_rule_paths, storage_location_for_rule,
    use_file_storage,
};
use crate::database::Database;
use crate::error::{AppError, Result};
use crate::file_storage;
use crate::models::{
    AdapterType, Command, CreateCommandInput, CreateRuleInput, CreateSkillInput,
    ImportArtifactType, ImportCandidate, ImportConflict, ImportConflictMode,
    ImportExecutionOptions, ImportExecutionResult, ImportHistoryEntry, ImportScanResult,
    ImportSkip, Rule, Scope, Skill, UpdateCommandInput, UpdateRuleInput, UpdateSkillInput,
};
use crate::sync::SyncEngine;

const DEFAULT_IMPORT_FILE_LIMIT: u64 = 10 * 1024 * 1024;
const MAX_IMPORT_CANDIDATES: usize = 1000;
const IMPORT_SOURCE_MAP_KEY: &str = "import_source_map";
const IMPORT_HISTORY_KEY: &str = "import_history";
const LOCAL_RULE_PATHS_KEY: &str = "local_rule_paths";

#[derive(Debug, Deserialize)]
struct JsonRulePayload {
    name: Option<String>,
    content: Option<String>,
    scope: Option<String>,
    // Security: target_paths is intentionally omitted to prevent arbitrary file writes.
    // Untrusted payloads cannot specify their own target paths.
    #[serde(rename = "enabledAdapters")]
    enabled_adapters: Option<Vec<String>>,
}

pub async fn scan_url_to_candidates(url: &str, max_size: u64) -> Result<ImportScanResult> {
    let parsed_url = validate_url_for_import(url)?;
    let response = reqwest::get(parsed_url.clone())
        .await
        .map_err(|e| AppError::InvalidInput {
            message: format!("Failed to fetch URL: {}", e),
        })?;

    validate_url_for_import(response.url().as_str())?;

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
    let inferred_name = parsed_url
        .path_segments()
        .and_then(|mut segments| segments.rfind(|s| !s.is_empty()))
        .unwrap_or("imported-url");
    scan.candidates.push(candidate_from_text(
        body,
        inferred_name,
        crate::models::ImportSourceType::Url,
        "URL",
        parsed_url.as_str(),
        None,
        Scope::Global,
        None,
        ImportArtifactType::Rule,
    ));
    Ok(scan)
}

pub fn scan_clipboard_to_candidates(
    content: &str,
    name: Option<&str>,
    max_size: u64,
) -> Result<ImportScanResult> {
    if content.len() as u64 > max_size {
        return Err(AppError::InvalidInput {
            message: format!("Clipboard content exceeds max size ({} bytes)", max_size),
        });
    }

    let mut scan = ImportScanResult::default();
    let inferred = name.unwrap_or("clipboard-import");
    scan.candidates.push(candidate_from_text(
        content.to_string(),
        inferred,
        crate::models::ImportSourceType::Clipboard,
        "Clipboard",
        "Clipboard",
        None,
        Scope::Global,
        None,
        ImportArtifactType::Rule,
    ));
    Ok(scan)
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
        ImportArtifactType::Rule,
        max_size,
    ) {
        Ok(candidate) => scan.candidates.push(candidate),
        Err(e) => scan.errors.push(e.to_string()),
    }
    scan
}

pub fn scan_directory_to_candidates(
    path: &Path,
    max_size: u64,
    artifact_filter: Option<ImportArtifactType>,
) -> ImportScanResult {
    let mut scan = ImportScanResult::default();
    let canonical_root = match path.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            scan.errors.push(format!(
                "Could not resolve directory '{}': {}",
                path.display(),
                e
            ));
            return scan;
        }
    };

    if !canonical_root.is_dir() {
        scan.errors.push(format!(
            "Import path '{}' is not a directory",
            canonical_root.display()
        ));
        return scan;
    }

    for entry in WalkDir::new(&canonical_root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let item_path = entry.path();
        if !item_path.is_file() {
            continue;
        }

        let artifact_type = detect_artifact_type_from_path(item_path);
        if let Some(filter) = artifact_filter {
            if artifact_type != filter {
                continue;
            }
        }

        if !is_supported_import_extension(item_path) {
            continue;
        }

        match candidate_from_path(
            item_path,
            crate::models::ImportSourceType::File,
            "File",
            None,
            Scope::Global,
            None,
            artifact_type,
            max_size,
        ) {
            Ok(candidate) => {
                if scan.candidates.len() >= MAX_IMPORT_CANDIDATES {
                    scan.errors.push(format!(
                        "Import candidate limit reached ({}). Narrow scan directory or import in batches.",
                        MAX_IMPORT_CANDIDATES
                    ));
                    return scan;
                }
                scan.candidates.push(candidate);
            }
            Err(e) => scan.errors.push(e.to_string()),
        }
    }

    apply_tool_suffix_name_policy(&mut scan.candidates);
    scan
}

pub async fn scan_ai_tool_candidates(db: Arc<Database>, max_size: u64) -> Result<ImportScanResult> {
    let mut scan = ImportScanResult::default();
    let home = dirs::home_dir()
        .ok_or_else(|| AppError::Path("Could not determine home directory".to_string()))?;

    for tool_path in global_tool_paths(&home) {
        if !tool_path.path.exists() {
            continue;
        }

        let tp = tool_path.clone();
        let label = adapter_label(tp.adapter);

        if tp.path.is_file() {
            match candidate_from_path(
                &tp.path,
                crate::models::ImportSourceType::AiTool,
                label,
                Some(tp.adapter),
                Scope::Global,
                None,
                tp.artifact_type,
                max_size,
            ) {
                Ok(candidate) => scan.candidates.push(candidate),
                Err(e) => scan.errors.push(e.to_string()),
            }
        } else if tool_path.path.is_dir() {
            let inner_scan = scan_directory_for_artifact_type(
                &tool_path.path,
                tool_path.adapter,
                max_size,
                tool_path.artifact_type,
            );
            for mut candidate in inner_scan.candidates {
                candidate.source_type = crate::models::ImportSourceType::AiTool;
                candidate.source_tool = Some(tool_path.adapter);
                candidate.source_label = adapter_label(tool_path.adapter).to_string();
                scan.candidates.push(candidate);
            }
            for err in inner_scan.errors {
                scan.errors.push(err);
            }
        }
    }

    for local_root in get_local_rule_roots(db.clone()).await {
        for local_path in local_tool_paths() {
            let path = local_root.join(local_path.relative_path);
            if !path.exists() || !path.is_file() {
                continue;
            }

            match candidate_from_path(
                &path,
                crate::models::ImportSourceType::AiTool,
                adapter_label(local_path.adapter),
                Some(local_path.adapter),
                Scope::Local,
                Some(vec![local_root.to_string_lossy().to_string()]),
                local_path.artifact_type,
                max_size,
            ) {
                Ok(candidate) => {
                    if scan.candidates.len() >= MAX_IMPORT_CANDIDATES {
                        scan.errors.push(format!(
                            "Import candidate limit reached ({}). Narrow configured repository roots or import in batches.",
                            MAX_IMPORT_CANDIDATES
                        ));
                        return Ok(scan);
                    }
                    scan.candidates.push(candidate)
                }
                Err(e) => scan.errors.push(e.to_string()),
            }
        }
    }

    apply_tool_suffix_name_policy(&mut scan.candidates);
    Ok(scan)
}

/// Scan a directory for a specific artifact type, excluding other artifact directories
fn scan_directory_for_artifact_type(
    dir: &Path,
    adapter: AdapterType,
    max_size: u64,
    artifact_type_filter: ImportArtifactType,
) -> ImportScanResult {
    let mut scan = ImportScanResult::default();

    for entry in WalkDir::new(dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let item_path = entry.path();

        // Skip directories that contain other artifact types
        if is_slash_command_or_skill_directory(item_path)
            && artifact_type_filter == ImportArtifactType::Rule
        {
            continue;
        }

        if !item_path.is_file() {
            continue;
        }

        if !is_supported_import_extension(item_path) {
            continue;
        }

        // Double-check: detect artifact type from path and skip if it doesn't match the filter
        if detect_artifact_type_from_path(item_path) != artifact_type_filter {
            continue;
        }

        match candidate_from_path(
            item_path,
            crate::models::ImportSourceType::AiTool,
            adapter_label(adapter),
            Some(adapter),
            Scope::Global,
            None,
            artifact_type_filter,
            max_size,
        ) {
            Ok(candidate) => {
                if scan.candidates.len() >= MAX_IMPORT_CANDIDATES {
                    scan.errors.push(format!(
                        "Import candidate limit reached ({}). Narrow directory scope or import in batches.",
                        MAX_IMPORT_CANDIDATES
                    ));
                    break;
                }
                scan.candidates.push(candidate)
            }
            Err(e) => scan.errors.push(e.to_string()),
        }
    }

    scan
}

/// Check if a path is within a slash command or skill directory
fn is_slash_command_or_skill_directory(path: &Path) -> bool {
    let path_str = path.to_string_lossy().to_lowercase();

    // Slash command directory patterns (case-insensitive)
    let slash_command_patterns = [
        "/commands/",
        "/workflows/",
        "/.gemini/antigravity/global_workflows/",
        "/documents/cline/workflows/",
        "\\commands\\",
        "\\workflows\\",
    ];

    // Skill directory patterns (case-insensitive)
    let skill_patterns = ["/skills/", "/documents/cline/skills/", "\\skills\\"];

    for pattern in &slash_command_patterns {
        if path_str.contains(pattern) {
            return true;
        }
    }

    for pattern in &skill_patterns {
        if path_str.contains(pattern) {
            return true;
        }
    }

    false
}

pub async fn execute_import(
    db: Arc<Database>,
    scan_result: ImportScanResult,
    options: ImportExecutionOptions,
) -> Result<ImportExecutionResult> {
    let mut result = ImportExecutionResult::default();
    let history_source_type = scan_result
        .candidates
        .first()
        .map(|c| c.source_type.clone())
        .unwrap_or(crate::models::ImportSourceType::AiTool);
    let scan_errors = scan_result.errors.clone();
    let selected_set = options
        .selected_candidate_ids
        .as_ref()
        .map(|ids| ids.iter().cloned().collect::<HashSet<String>>());
    let mut existing_rules = db.get_all_rules().await?;
    let mut existing_commands = db.get_all_commands().await?;
    let mut existing_skills = db.get_all_skills().await?;
    let mut source_map = read_source_map(db.clone()).await;

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

        if let Some(existing_exact_id) = match candidate.artifact_type {
            ImportArtifactType::Rule => existing_rules
                .iter()
                .find(|r| compute_content_hash(&r.content) == candidate.content_hash)
                .map(|r| r.name.clone()),
            ImportArtifactType::SlashCommand => existing_commands
                .iter()
                .find(|c| compute_content_hash(&c.script) == candidate.content_hash)
                .map(|c| c.name.clone()),
            ImportArtifactType::Skill => existing_skills
                .iter()
                .find(|s| compute_content_hash(&s.instructions) == candidate.content_hash)
                .map(|s| s.name.clone()),
        } {
            result.skipped.push(ImportSkip {
                candidate_id: candidate.id.clone(),
                name: candidate.proposed_name.clone(),
                reason: format!(
                    "Duplicate content already exists as '{}'",
                    existing_exact_id
                ),
            });
            continue;
        }

        let mapped_artifact_id = source_map.get(&source_key).cloned();
        if let Some(artifact_id) = mapped_artifact_id {
            match candidate.artifact_type {
                ImportArtifactType::Rule => {
                    let update = db
                        .update_rule(
                            &artifact_id,
                            UpdateRuleInput {
                                name: Some(candidate.proposed_name.clone()),
                                description: None,
                                content: Some(candidate.content.clone()),
                                scope: Some(effective_scope),
                                target_paths: candidate.target_paths.clone(),
                                enabled_adapters: Some(effective_adapters.clone()),
                                enabled: Some(true),
                            },
                        )
                        .await?;
                    persist_rule_to_file_if_needed(db.clone(), &update).await?;
                    existing_rules.retain(|r| r.id != update.id);
                    existing_rules.push(update.clone());
                    result.imported_rules.push(update.clone());
                    result.imported.push(update);
                }
                ImportArtifactType::SlashCommand => {
                    let update = db
                        .update_command(
                            &artifact_id,
                            UpdateCommandInput {
                                name: Some(candidate.proposed_name.clone()),
                                script: Some(candidate.content.clone()),
                                ..Default::default()
                            },
                        )
                        .await?;
                    // Persistence for commands
                    persist_command_to_file_if_needed(db.clone(), &update).await?;
                    existing_commands.retain(|c| c.id != update.id);
                    existing_commands.push(update.clone());
                    result.imported_commands.push(update);
                }
                ImportArtifactType::Skill => {
                    let update = db
                        .update_skill(
                            &artifact_id,
                            UpdateSkillInput {
                                name: Some(candidate.proposed_name.clone()),
                                instructions: Some(candidate.content.clone()),
                                ..Default::default()
                            },
                        )
                        .await?;
                    // Persistence for skills
                    persist_skill_to_file_if_needed(db.clone(), &update).await?;
                    existing_skills.retain(|s| s.id != update.id);
                    existing_skills.push(update.clone());
                    result.imported_skills.push(update);
                }
            }
            continue;
        }

        let same_name_id = match candidate.artifact_type {
            ImportArtifactType::Rule => existing_rules
                .iter()
                .find(|r| r.name.eq_ignore_ascii_case(&candidate.proposed_name))
                .map(|r| (r.id.clone(), r.name.clone(), r.content.clone())),
            ImportArtifactType::SlashCommand => existing_commands
                .iter()
                .find(|c| c.name.eq_ignore_ascii_case(&candidate.proposed_name))
                .map(|c| (c.id.clone(), c.name.clone(), c.script.clone())),
            ImportArtifactType::Skill => existing_skills
                .iter()
                .find(|s| s.name.eq_ignore_ascii_case(&candidate.proposed_name))
                .map(|s| (s.id.clone(), s.name.clone(), s.instructions.clone())),
        };

        if let Some((existing_id, existing_name, existing_content)) = same_name_id {
            if existing_content == candidate.content {
                result.skipped.push(ImportSkip {
                    candidate_id: candidate.id.clone(),
                    name: candidate.proposed_name.clone(),
                    reason: format!(
                        "Duplicate name and content already exists as '{}'",
                        existing_name
                    ),
                });
                continue;
            }

            match options.conflict_mode {
                ImportConflictMode::Skip => {
                    result.conflicts.push(ImportConflict {
                        candidate_id: candidate.id.clone(),
                        candidate_name: candidate.proposed_name.clone(),
                        existing_rule_id: Some(existing_id.clone()),
                        existing_rule_name: Some(existing_name.clone()),
                        existing_id: Some(existing_id),
                        existing_name: Some(existing_name),
                        reason: "Name collision with different content".to_string(),
                    });
                    continue;
                }
                ImportConflictMode::Replace => {
                    match candidate.artifact_type {
                        ImportArtifactType::Rule => {
                            let update = db
                                .update_rule(
                                    &existing_id,
                                    UpdateRuleInput {
                                        name: Some(candidate.proposed_name.clone()),
                                        description: None,
                                        content: Some(candidate.content.clone()),
                                        scope: Some(effective_scope),
                                        target_paths: candidate.target_paths.clone(),
                                        enabled_adapters: Some(effective_adapters.clone()),
                                        enabled: Some(true),
                                    },
                                )
                                .await?;
                            persist_rule_to_file_if_needed(db.clone(), &update).await?;
                            source_map.insert(source_key, update.id.clone());
                            existing_rules.retain(|r| r.id != update.id);
                            existing_rules.push(update.clone());
                            result.imported_rules.push(update.clone());
                            result.imported.push(update);
                        }
                        ImportArtifactType::SlashCommand => {
                            let update = db
                                .update_command(
                                    &existing_id,
                                    UpdateCommandInput {
                                        name: Some(candidate.proposed_name.clone()),
                                        script: Some(candidate.content.clone()),
                                        ..Default::default()
                                    },
                                )
                                .await?;
                            persist_command_to_file_if_needed(db.clone(), &update).await?;
                            source_map.insert(source_key, update.id.clone());
                            existing_commands.retain(|c| c.id != update.id);
                            existing_commands.push(update.clone());
                            result.imported_commands.push(update);
                        }
                        ImportArtifactType::Skill => {
                            let update = db
                                .update_skill(
                                    &existing_id,
                                    UpdateSkillInput {
                                        name: Some(candidate.proposed_name.clone()),
                                        instructions: Some(candidate.content.clone()),
                                        ..Default::default()
                                    },
                                )
                                .await?;
                            persist_skill_to_file_if_needed(db.clone(), &update).await?;
                            source_map.insert(source_key, update.id.clone());
                            existing_skills.retain(|s| s.id != update.id);
                            existing_skills.push(update.clone());
                            result.imported_skills.push(update);
                        }
                    }
                    continue;
                }
                ImportConflictMode::Rename => {
                    let unique_name = match candidate.artifact_type {
                        ImportArtifactType::Rule => make_unique_name(
                            &candidate.proposed_name,
                            &existing_rules
                                .iter()
                                .map(|r| r.name.clone())
                                .collect::<Vec<_>>(),
                        ),
                        ImportArtifactType::SlashCommand => make_unique_name(
                            &candidate.proposed_name,
                            &existing_commands
                                .iter()
                                .map(|c| c.name.clone())
                                .collect::<Vec<_>>(),
                        ),
                        ImportArtifactType::Skill => make_unique_name(
                            &candidate.proposed_name,
                            &existing_skills
                                .iter()
                                .map(|s| s.name.clone())
                                .collect::<Vec<_>>(),
                        ),
                    };

                    match candidate.artifact_type {
                        ImportArtifactType::Rule => {
                            let created = db
                                .create_rule(CreateRuleInput {
                                    id: None,
                                    name: unique_name,
                                    description: String::new(),
                                    content: candidate.content.clone(),
                                    scope: effective_scope,
                                    target_paths: candidate.target_paths.clone(),
                                    enabled_adapters: effective_adapters.clone(),
                                    enabled: true,
                                })
                                .await?;
                            persist_rule_to_file_if_needed(db.clone(), &created).await?;
                            source_map.insert(source_key, created.id.clone());
                            existing_rules.push(created.clone());
                            result.imported_rules.push(created.clone());
                            result.imported.push(created);
                        }
                        ImportArtifactType::SlashCommand => {
                            let created = db
                                .create_command(CreateCommandInput {
                                    name: unique_name,
                                    script: candidate.content.clone(),
                                    ..Default::default()
                                })
                                .await?;
                            persist_command_to_file_if_needed(db.clone(), &created).await?;
                            source_map.insert(source_key, created.id.clone());
                            existing_commands.push(created.clone());
                            result.imported_commands.push(created);
                        }
                        ImportArtifactType::Skill => {
                            let created = db
                                .create_skill(CreateSkillInput {
                                    name: unique_name,
                                    instructions: candidate.content.clone(),
                                    ..Default::default()
                                })
                                .await?;
                            persist_skill_to_file_if_needed(db.clone(), &created).await?;
                            source_map.insert(source_key, created.id.clone());
                            existing_skills.push(created.clone());
                            result.imported_skills.push(created);
                        }
                    }
                    continue;
                }
            }
        }

        match candidate.artifact_type {
            ImportArtifactType::Rule => {
                let created = db
                    .create_rule(CreateRuleInput {
                        id: None,
                        name: candidate.proposed_name.clone(),
                        description: String::new(),
                        content: candidate.content.clone(),
                        scope: effective_scope,
                        target_paths: candidate.target_paths.clone(),
                        enabled_adapters: effective_adapters,
                        enabled: true,
                    })
                    .await?;
                persist_rule_to_file_if_needed(db.clone(), &created).await?;
                source_map.insert(source_key, created.id.clone());
                existing_rules.retain(|r| r.id != created.id); // Guard against DB race
                existing_rules.push(created.clone());
                result.imported_rules.push(created.clone());
                result.imported.push(created);
            }
            ImportArtifactType::SlashCommand => {
                let created = db
                    .create_command(CreateCommandInput {
                        name: candidate.proposed_name.clone(),
                        script: candidate.content.clone(),
                        ..Default::default()
                    })
                    .await?;
                persist_command_to_file_if_needed(db.clone(), &created).await?;
                source_map.insert(source_key, created.id.clone());
                existing_commands.push(created.clone());
                result.imported_commands.push(created);
            }
            ImportArtifactType::Skill => {
                let created = db
                    .create_skill(CreateSkillInput {
                        name: candidate.proposed_name.clone(),
                        instructions: candidate.content.clone(),
                        ..Default::default()
                    })
                    .await?;
                persist_skill_to_file_if_needed(db.clone(), &created).await?;
                source_map.insert(source_key, created.id.clone());
                existing_skills.push(created.clone());
                result.imported_skills.push(created);
            }
        }
    }

    write_source_map(db.clone(), &source_map).await?;
    append_history(
        db.clone(),
        ImportHistoryEntry {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            source_type: history_source_type,
            imported_count: result.imported_rules.len()
                + result.imported_commands.len()
                + result.imported_skills.len(),
            skipped_count: result.skipped.len(),
            conflict_count: result.conflicts.len(),
            error_count: result.errors.len() + scan_errors.len(),
        },
    )
    .await?;

    let engine = SyncEngine::new(&db);
    let all_rules = db.get_all_rules().await?;
    let sync_res = engine.sync_all(all_rules);
    for err in sync_res.await.errors {
        result.errors.push(format!(
            "Sync error for {}: {}",
            err.adapter_name, err.message
        ));
    }

    // Comprehensive reconciliation to ensure all artifact types are synced to disk
    reconcile_after_mutation(db.clone()).await;

    for err in scan_errors {
        result.errors.push(err);
    }

    Ok(result)
}

pub async fn read_import_history(db: Arc<Database>) -> Vec<ImportHistoryEntry> {
    let encoded = match db.get_setting(IMPORT_HISTORY_KEY).await {
        Ok(Some(v)) => v,
        _ => return Vec::new(),
    };
    serde_json::from_str(&encoded).unwrap_or_default()
}

async fn append_history(db: Arc<Database>, entry: ImportHistoryEntry) -> Result<()> {
    let mut history = read_import_history(db.clone()).await;
    history.insert(0, entry);
    if history.len() > 50 {
        history.truncate(50);
    }
    let encoded = serde_json::to_string(&history)?;
    db.set_setting(IMPORT_HISTORY_KEY, &encoded).await
}

async fn persist_rule_to_file_if_needed(db: Arc<Database>, rule: &Rule) -> Result<()> {
    if use_file_storage(&db).await {
        let location = storage_location_for_rule(rule);
        file_storage::save_rule_to_disk(rule, &location)?;
        db.update_rule_file_index(&rule.id, &location).await?;
        register_local_rule_paths(&db, rule).await?;
    }
    Ok(())
}

async fn persist_command_to_file_if_needed(db: Arc<Database>, command: &Command) -> Result<()> {
    if use_file_storage(&db).await {
        // Commands and skills are currently managed via reconciliation which handles periodic sync.
        // For individual mutations, we trigger a global reconciliation at the end of execution.
        log::debug!(
            "Mutation for command {} recorded. Persistence will be handled by final reconciliation.",
            command.id
        );
    }
    Ok(())
}

async fn persist_skill_to_file_if_needed(db: Arc<Database>, skill: &Skill) -> Result<()> {
    if use_file_storage(&db).await {
        log::debug!(
            "Mutation for skill {} recorded. Persistence will be handled by final reconciliation.",
            skill.id
        );
    }
    Ok(())
}

async fn get_local_rule_roots(db: Arc<Database>) -> Vec<PathBuf> {
    let roots_json = db
        .get_setting(LOCAL_RULE_PATHS_KEY)
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "[]".to_string());
    let roots: Vec<String> = serde_json::from_str(&roots_json).unwrap_or_default();
    roots.into_iter().map(PathBuf::from).collect()
}

async fn read_source_map(db: Arc<Database>) -> HashMap<String, String> {
    let encoded = match db.get_setting(IMPORT_SOURCE_MAP_KEY).await {
        Ok(Some(v)) => v,
        _ => return HashMap::new(),
    };
    serde_json::from_str(&encoded).unwrap_or_default()
}

async fn write_source_map(db: Arc<Database>, map: &HashMap<String, String>) -> Result<()> {
    let encoded = serde_json::to_string(map)?;
    db.set_setting(IMPORT_SOURCE_MAP_KEY, &encoded).await
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

/// Represents a tool path with its artifact type (rules vs slash commands vs skills)
#[derive(Debug, Clone)]
struct ToolPath {
    adapter: AdapterType,
    path: PathBuf,
    artifact_type: ImportArtifactType,
}

/// Represents a local tool path with its artifact type
#[derive(Debug, Clone)]
struct LocalToolPath {
    adapter: AdapterType,
    relative_path: &'static str,
    artifact_type: ImportArtifactType,
}

/// Detect artifact type based on directory patterns in the path
fn detect_artifact_type_from_path(path: &Path) -> ImportArtifactType {
    let path_str = path.to_string_lossy().to_lowercase();
    let path_str_lower = path_str.to_lowercase();

    // Slash command directories - these are workflow/command files
    let slash_command_patterns = [
        "/commands/",
        "/workflows/",
        "\\commands\\",
        "\\workflows\\",
        ".gemini/antigravity/global_workflows",
        "documents/cline/workflows",
        ".agents/workflows",
        ".clinerules/workflows",
    ];

    // Skill directories - these contain skill definitions
    let skill_patterns = ["/skills/", "\\skills\\", "documents/cline/skills"];

    // Check for slash command patterns
    for pattern in &slash_command_patterns {
        if path_str_lower.contains(pattern) {
            return ImportArtifactType::SlashCommand;
        }
    }

    // Check for skill patterns
    for pattern in &skill_patterns {
        if path_str_lower.contains(pattern) {
            return ImportArtifactType::Skill;
        }
    }

    // Default to rule
    ImportArtifactType::Rule
}

fn global_tool_paths(home: &Path) -> Vec<ToolPath> {
    // Rule paths only - these are the main rule/configuration files
    vec![
        ToolPath {
            adapter: AdapterType::Gemini,
            path: home.join(".gemini").join("GEMINI.md"),
            artifact_type: ImportArtifactType::Rule,
        },
        ToolPath {
            adapter: AdapterType::Antigravity,
            path: home.join(".antigravity").join("GEMINI.md"),
            artifact_type: ImportArtifactType::Rule,
        },
        ToolPath {
            adapter: AdapterType::Antigravity,
            path: home.join(".gemini").join("antigravity").join("GEMINI.md"),
            artifact_type: ImportArtifactType::Rule,
        },
        ToolPath {
            adapter: AdapterType::OpenCode,
            path: home.join(".config").join("opencode").join("AGENTS.md"),
            artifact_type: ImportArtifactType::Rule,
        },
        ToolPath {
            adapter: AdapterType::OpenCode,
            path: home.join(".opencode").join("AGENTS.md"),
            artifact_type: ImportArtifactType::Rule,
        },
        ToolPath {
            adapter: AdapterType::Cline,
            path: home.join(".clinerules"),
            artifact_type: ImportArtifactType::Rule,
        },
        ToolPath {
            adapter: AdapterType::Cline,
            path: home.join("Documents").join("Cline").join("Rules"),
            artifact_type: ImportArtifactType::Rule,
        },
        ToolPath {
            adapter: AdapterType::ClaudeCode,
            path: home.join(".claude").join("CLAUDE.md"),
            artifact_type: ImportArtifactType::Rule,
        },
        ToolPath {
            adapter: AdapterType::Codex,
            path: home.join(".codex").join("AGENTS.md"),
            artifact_type: ImportArtifactType::Rule,
        },
        ToolPath {
            adapter: AdapterType::Codex,
            path: home.join(".agents").join("AGENTS.md"),
            artifact_type: ImportArtifactType::Rule,
        },
        ToolPath {
            adapter: AdapterType::Kilo,
            path: home.join(".kilocode").join("rules").join("AGENTS.md"),
            artifact_type: ImportArtifactType::Rule,
        },
        ToolPath {
            adapter: AdapterType::Kilo,
            path: home.join(".kilo").join("rules").join("AGENTS.md"),
            artifact_type: ImportArtifactType::Rule,
        },
        ToolPath {
            adapter: AdapterType::Cursor,
            path: home.join(".cursorrules"),
            artifact_type: ImportArtifactType::Rule,
        },
        ToolPath {
            adapter: AdapterType::Windsurf,
            path: home.join(".windsurf").join("rules").join("AGENTS.md"),
            artifact_type: ImportArtifactType::Rule,
        },
        ToolPath {
            adapter: AdapterType::Windsurf,
            path: home.join(".windsurfrules"),
            artifact_type: ImportArtifactType::Rule,
        },
        ToolPath {
            adapter: AdapterType::Windsurf,
            path: home.join(".windsurf").join("rules").join("rules.md"),
            artifact_type: ImportArtifactType::Rule,
        },
        ToolPath {
            adapter: AdapterType::RooCode,
            path: home.join(".roo").join("rules").join("AGENTS.md"),
            artifact_type: ImportArtifactType::Rule,
        },
        ToolPath {
            adapter: AdapterType::RooCode,
            path: home.join(".roo").join("rules").join("rules.md"),
            artifact_type: ImportArtifactType::Rule,
        },
        ToolPath {
            adapter: AdapterType::RooCode,
            path: home.join(".roocode").join("rules").join("AGENTS.md"),
            artifact_type: ImportArtifactType::Rule,
        },
        ToolPath {
            adapter: AdapterType::RooCode,
            path: home.join(".roocode").join("rules").join("rules.md"),
            artifact_type: ImportArtifactType::Rule,
        },
        // Slash Command Paths
        ToolPath {
            adapter: AdapterType::Antigravity,
            path: home
                .join(".gemini")
                .join("antigravity")
                .join("global_workflows"),
            artifact_type: ImportArtifactType::SlashCommand,
        },
        ToolPath {
            adapter: AdapterType::Cline,
            path: home.join("Documents").join("Cline").join("Workflows"),
            artifact_type: ImportArtifactType::SlashCommand,
        },
        ToolPath {
            adapter: AdapterType::RooCode,
            path: home.join(".roo").join("workflows"),
            artifact_type: ImportArtifactType::SlashCommand,
        },
        // Skill Paths
        ToolPath {
            adapter: AdapterType::Cline,
            path: home.join("Documents").join("Cline").join("Skills"),
            artifact_type: ImportArtifactType::Skill,
        },
        ToolPath {
            adapter: AdapterType::RooCode,
            path: home.join(".roo").join("skills"),
            artifact_type: ImportArtifactType::Skill,
        },
    ]
}

fn local_tool_paths() -> Vec<LocalToolPath> {
    // Rule paths only - local repository rule files
    vec![
        LocalToolPath {
            adapter: AdapterType::Gemini,
            relative_path: ".gemini/GEMINI.md",
            artifact_type: ImportArtifactType::Rule,
        },
        LocalToolPath {
            adapter: AdapterType::Antigravity,
            relative_path: ".antigravity/GEMINI.md",
            artifact_type: ImportArtifactType::Rule,
        },
        LocalToolPath {
            adapter: AdapterType::Antigravity,
            relative_path: ".gemini/antigravity/GEMINI.md",
            artifact_type: ImportArtifactType::Rule,
        },
        LocalToolPath {
            adapter: AdapterType::OpenCode,
            relative_path: ".opencode/AGENTS.md",
            artifact_type: ImportArtifactType::Rule,
        },
        LocalToolPath {
            adapter: AdapterType::OpenCode,
            relative_path: ".config/opencode/AGENTS.md",
            artifact_type: ImportArtifactType::Rule,
        },
        LocalToolPath {
            adapter: AdapterType::Cline,
            relative_path: ".clinerules",
            artifact_type: ImportArtifactType::Rule,
        },
        LocalToolPath {
            adapter: AdapterType::ClaudeCode,
            relative_path: ".claude/CLAUDE.md",
            artifact_type: ImportArtifactType::Rule,
        },
        LocalToolPath {
            adapter: AdapterType::Codex,
            relative_path: ".codex/AGENTS.md",
            artifact_type: ImportArtifactType::Rule,
        },
        LocalToolPath {
            adapter: AdapterType::Codex,
            relative_path: ".agents/AGENTS.md",
            artifact_type: ImportArtifactType::Rule,
        },
        LocalToolPath {
            adapter: AdapterType::Kilo,
            relative_path: ".kilocode/rules/AGENTS.md",
            artifact_type: ImportArtifactType::Rule,
        },
        LocalToolPath {
            adapter: AdapterType::Kilo,
            relative_path: ".kilo/rules/AGENTS.md",
            artifact_type: ImportArtifactType::Rule,
        },
        LocalToolPath {
            adapter: AdapterType::Cursor,
            relative_path: ".cursorrules",
            artifact_type: ImportArtifactType::Rule,
        },
        LocalToolPath {
            adapter: AdapterType::Windsurf,
            relative_path: ".windsurf/rules/AGENTS.md",
            artifact_type: ImportArtifactType::Rule,
        },
        LocalToolPath {
            adapter: AdapterType::Windsurf,
            relative_path: ".windsurfrules",
            artifact_type: ImportArtifactType::Rule,
        },
        LocalToolPath {
            adapter: AdapterType::RooCode,
            relative_path: ".roo/rules/AGENTS.md",
            artifact_type: ImportArtifactType::Rule,
        },
        LocalToolPath {
            adapter: AdapterType::RooCode,
            relative_path: ".roo/rules/rules.md",
            artifact_type: ImportArtifactType::Rule,
        },
        // Local Workflows
        LocalToolPath {
            adapter: AdapterType::Gemini,
            relative_path: ".agents/workflows",
            artifact_type: ImportArtifactType::SlashCommand,
        },
        LocalToolPath {
            adapter: AdapterType::Antigravity,
            relative_path: ".gemini/antigravity/workflows",
            artifact_type: ImportArtifactType::SlashCommand,
        },
        // Local Skills
        LocalToolPath {
            adapter: AdapterType::Gemini,
            relative_path: ".agents/skills",
            artifact_type: ImportArtifactType::Skill,
        },
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

#[allow(clippy::too_many_arguments)]
fn candidate_from_path(
    path: &Path,
    source_type: crate::models::ImportSourceType,
    source_label: &str,
    source_tool: Option<AdapterType>,
    scope: Scope,
    target_paths: Option<Vec<String>>,
    artifact_type: ImportArtifactType,
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
        artifact_type,
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
    artifact_type: ImportArtifactType,
) -> ImportCandidate {
    let (name, parsed_content, parsed_scope, parsed_target_paths, parsed_adapters) =
        if artifact_type == ImportArtifactType::Rule {
            extract_rule_payload(default_name, &content, scope, target_paths, source_tool)
        } else {
            (
                default_name.to_string(),
                content.clone(),
                scope,
                target_paths,
                default_adapters(source_tool),
            )
        };

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
        artifact_type,
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

    let try_parse = |text: &str| -> Option<JsonRulePayload> {
        if let Ok(payload) = serde_json::from_str::<JsonRulePayload>(text) {
            return Some(payload);
        }
        if let Ok(payload) = serde_yaml::from_str::<JsonRulePayload>(text) {
            return Some(payload);
        }
        None
    };

    if let Some(payload) = try_parse(&trimmed) {
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
            fallback_targets, // Always use fallback targets (safe), never payload targets
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

fn validate_url_for_import(input: &str) -> Result<url::Url> {
    let parsed = url::Url::parse(input).map_err(|e| AppError::InvalidInput {
        message: format!("Invalid URL: {}", e),
    })?;

    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(AppError::InvalidInput {
            message: "Only http/https URLs are allowed".to_string(),
        });
    }

    let host = parsed.host_str().ok_or_else(|| AppError::InvalidInput {
        message: "URL must include a host".to_string(),
    })?;

    if host.eq_ignore_ascii_case("localhost") {
        return Err(AppError::InvalidInput {
            message: "Localhost URLs are not allowed".to_string(),
        });
    }

    if let Ok(ip) = host.parse::<IpAddr>() {
        if is_disallowed_ip(&ip) {
            return Err(AppError::InvalidInput {
                message: "URLs targeting private or local IP ranges are not allowed".to_string(),
            });
        }
    }

    Ok(parsed)
}

fn is_disallowed_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_private()
                || v4.is_loopback()
                || v4.is_link_local()
                || v4.is_broadcast()
                || v4.is_unspecified()
                || v4.is_multicast()
                || v4.is_documentation()
        }
        IpAddr::V6(v6) => {
            v6.is_loopback()
                || v6.is_unique_local()
                || v6.is_unicast_link_local()
                || v6.is_unspecified()
                || v6.is_multicast()
        }
    }
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
    use std::fs;
    use std::path::PathBuf;

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
                ImportArtifactType::Rule,
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
                ImportArtifactType::Rule,
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

    #[tokio::test]
    async fn execute_import_skips_duplicate_content() {
        let db = Database::new_in_memory().await.expect("in-memory db");
        db.create_rule(CreateRuleInput {
            id: None,
            name: "Existing".to_string(),
            description: "".to_string(),
            content: "same-content".to_string(),
            scope: Scope::Global,
            target_paths: None,
            enabled_adapters: vec![AdapterType::Gemini],
            enabled: true,
        })
        .await
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
            ImportArtifactType::Rule,
        );

        let result = execute_import(
            db.clone(),
            ImportScanResult {
                candidates: vec![candidate],
                errors: vec![],
            },
            ImportExecutionOptions::default(),
        )
        .await
        .expect("execute import");

        assert_eq!(result.imported.len(), 0);
        assert_eq!(result.skipped.len(), 1);
    }

    #[tokio::test]
    async fn execute_import_rename_mode_creates_unique_name() {
        let db = Database::new_in_memory().await.expect("in-memory db");
        db.create_rule(CreateRuleInput {
            id: None,
            name: "quality".to_string(),
            description: "".to_string(),
            content: "old".to_string(),
            scope: Scope::Global,
            target_paths: None,
            enabled_adapters: vec![AdapterType::Gemini],
            enabled: true,
        })
        .await
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
            ImportArtifactType::Rule,
        );

        let scan_result = ImportScanResult {
            candidates: vec![candidate],
            errors: vec![],
        };

        let result = execute_import(
            db.clone(),
            scan_result,
            ImportExecutionOptions {
                conflict_mode: ImportConflictMode::Rename,
                ..Default::default()
            },
        )
        .await
        .expect("execute import");

        assert_eq!(result.imported.len(), 1);
        assert_eq!(result.imported[0].name, "quality-2");
    }

    #[tokio::test]
    async fn execute_import_replace_mode_updates_existing_rule() {
        let db = Arc::new(Database::new_in_memory().await.expect("in-memory db"));
        let existing = db
            .create_rule(CreateRuleInput {
                id: None,
                name: "policy".to_string(),
                description: "".to_string(),
                content: "old".to_string(),
                scope: Scope::Global,
                target_paths: None,
                enabled_adapters: vec![AdapterType::Gemini],
                enabled: true,
            })
            .await
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
            ImportArtifactType::Rule,
        );

        let scan_result = ImportScanResult {
            candidates: vec![candidate],
            errors: vec![],
        };

        let result = execute_import(
            db.clone(),
            scan_result,
            ImportExecutionOptions {
                conflict_mode: ImportConflictMode::Replace,
                ..Default::default()
            },
        )
        .await
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
        // Security fix: verify target_paths from JSON are IGNORED
        assert_eq!(target_paths, None);
        assert_eq!(adapters.len(), 2);
    }

    #[test]
    fn extract_payload_ignores_malicious_paths() {
        let json = r#"{
          "name": "malicious",
          "content": "bad",
          "targetPaths": ["../../../../Windows/System32"]
        }"#;

        let (_, _, _, target_paths, _) =
            extract_rule_payload("fallback", json, Scope::Global, None, None);

        assert_eq!(target_paths, None);
    }

    #[test]
    fn extract_payload_respects_fallback_paths() {
        let json = r#"{ "name": "ok", "content": "ok" }"#;
        let fallback = Some(vec!["C:/safe/path".to_string()]);

        let (_, _, _, target_paths, _) =
            extract_rule_payload("fallback", json, Scope::Global, fallback.clone(), None);

        assert_eq!(target_paths, fallback);
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

        let (name, content, scope, _target_paths, adapters) = extract_rule_payload(
            "fallback",
            yaml,
            Scope::Local,
            Some(vec!["x".to_string()]),
            None,
        );

        assert_eq!(name, "yaml-rule");
        assert_eq!(content, "yaml body");
        assert_eq!(scope, Scope::Global);
        assert_eq!(adapters, vec![AdapterType::Cline]);
    }

    #[tokio::test]
    async fn execute_import_reimport_updates_mapped_rule_idempotently() {
        let db = Database::new_in_memory().await.expect("in-memory db");

        let first_candidate = candidate_from_text(
            "original content".to_string(),
            "shared-rule",
            crate::models::ImportSourceType::File,
            "File",
            "C:/tmp/shared-rule.md",
            None,
            Scope::Global,
            None,
            ImportArtifactType::Rule,
        );

        let first_result = execute_import(
            db.clone(),
            ImportScanResult {
                candidates: vec![first_candidate],
                errors: vec![],
            },
            ImportExecutionOptions::default(),
        )
        .await
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
            ImportArtifactType::Rule,
        );

        let second_result = execute_import(
            db.clone(),
            ImportScanResult {
                candidates: vec![second_candidate],
                errors: vec![],
            },
            ImportExecutionOptions::default(),
        )
        .await
        .expect("second import");

        assert_eq!(second_result.imported.len(), 1);
        assert_eq!(second_result.imported[0].id, imported_id);
        assert_eq!(second_result.imported[0].content, "updated content");
    }

    #[test]
    fn scan_clipboard_respects_max_size_limit() {
        let oversized = "a".repeat(11);
        let result = scan_clipboard_to_candidates(&oversized, Some("clip"), 10);
        assert!(result.is_err());
    }

    #[test]
    fn validate_url_blocks_localhost_and_private_ips() {
        assert!(validate_url_for_import("http://localhost:8080/a").is_err());
        assert!(validate_url_for_import("http://127.0.0.1:8080/a").is_err());
        assert!(validate_url_for_import("https://10.0.0.5/a").is_err());
    }

    #[test]
    fn validate_url_allows_public_http_https() {
        assert!(validate_url_for_import("https://example.com/rules.md").is_ok());
        assert!(validate_url_for_import("http://example.com/rules.md").is_ok());
        assert!(validate_url_for_import("ftp://example.com/rules.md").is_err());
    }

    #[tokio::test]
    async fn history_source_type_matches_candidate_source() {
        let db = Database::new_in_memory().await.expect("in-memory db");

        let candidate = candidate_from_text(
            "file content".to_string(),
            "file-rule",
            crate::models::ImportSourceType::File,
            "File",
            "C:/tmp/r.md",
            None,
            Scope::Global,
            None,
            ImportArtifactType::Rule,
        );

        execute_import(
            db.clone(),
            ImportScanResult {
                candidates: vec![candidate],
                errors: vec![],
            },
            ImportExecutionOptions::default(),
        )
        .await
        .expect("import succeeds");

        let history = read_import_history(db.clone()).await;
        assert!(!history.is_empty());
        assert_eq!(
            history[0].source_type,
            crate::models::ImportSourceType::File
        );
    }

    #[test]
    fn scan_directory_reports_error_for_non_directory_path() {
        let mut temp_file = std::env::temp_dir();
        temp_file.push(format!(
            "ruleweaver-import-test-{}.md",
            uuid::Uuid::new_v4()
        ));
        fs::write(&temp_file, "test").expect("write temp file");

        let result = scan_directory_to_candidates(&temp_file, 1024, None);
        assert!(!result.errors.is_empty());

        let _ = fs::remove_file(temp_file);
    }

    #[test]
    fn tool_path_matrix_includes_legacy_and_alternate_locations() {
        let home = PathBuf::from("/home/test");
        let global = global_tool_paths(&home)
            .into_iter()
            .map(|tp| tp.path.to_string_lossy().to_string())
            .collect::<Vec<_>>();

        assert!(global
            .iter()
            .any(|p| p.contains(".antigravity") && p.contains("GEMINI.md")));
        assert!(global.iter().any(|p| p.contains(".windsurfrules")));
        assert!(global
            .iter()
            .any(|p| p.contains(".roocode") && p.contains("rules.md")));
        assert!(global
            .iter()
            .any(|p| p.contains(".kilo") && p.contains("AGENTS.md")));

        // Verify no slash command or skill directories are included in rule paths
        assert!(!global.iter().any(|p| p.contains("/commands/")));
        assert!(!global.iter().any(|p| p.contains("/workflows/")));
        assert!(!global.iter().any(|p| p.contains("/skills/")));
    }

    #[test]
    fn detect_artifact_type_identifies_rules_vs_commands_vs_skills() {
        // Rule paths
        assert_eq!(
            detect_artifact_type_from_path(Path::new("/home/.gemini/GEMINI.md")),
            ImportArtifactType::Rule
        );
        assert_eq!(
            detect_artifact_type_from_path(Path::new("/home/.clinerules")),
            ImportArtifactType::Rule
        );
        assert_eq!(
            detect_artifact_type_from_path(Path::new("/repo/.claude/CLAUDE.md")),
            ImportArtifactType::Rule
        );

        // Slash command paths
        assert_eq!(
            detect_artifact_type_from_path(Path::new("/home/.claude/commands/test.md")),
            ImportArtifactType::SlashCommand
        );
        assert_eq!(
            detect_artifact_type_from_path(Path::new(
                "/home/Documents/Cline/Workflows/my-workflow.md"
            )),
            ImportArtifactType::SlashCommand
        );
        assert_eq!(
            detect_artifact_type_from_path(Path::new(
                "C:/Users/test/.gemini/antigravity/global_workflows/workflow.md"
            )),
            ImportArtifactType::SlashCommand
        );

        // Skill paths
        assert_eq!(
            detect_artifact_type_from_path(Path::new("/home/.claude/skills/my-skill/SKILL.md")),
            ImportArtifactType::Skill
        );
        assert_eq!(
            detect_artifact_type_from_path(Path::new(
                "/home/Documents/Cline/Skills/test-skill/skill.md"
            )),
            ImportArtifactType::Skill
        );
    }

    #[test]
    fn scan_directory_excludes_slash_commands_and_skills() {
        let temp_dir = tempfile::TempDir::new().unwrap();

        // Create rule file
        let rule_file = temp_dir.path().join("GEMINI.md");
        fs::write(&rule_file, "# Rule content\n\nSome rule text").unwrap();

        // Create slash command file
        let commands_dir = temp_dir.path().join("commands");
        fs::create_dir_all(&commands_dir).unwrap();
        let command_file = commands_dir.join("my-command.md");
        fs::write(&command_file, "# My Command\n\nCommand content").unwrap();

        // Create skill file
        let skills_dir = temp_dir.path().join("skills");
        fs::create_dir_all(&skills_dir).unwrap();
        let skill_file = skills_dir.join("my-skill.md");
        fs::write(&skill_file, "# My Skill\n\nSkill content").unwrap();

        // Scan the directory
        let result = scan_directory_to_candidates(temp_dir.path(), 1024 * 1024, None);

        // Should only have 1 candidate (the rule file)
        assert_eq!(
            result.candidates.len(),
            1,
            "Should only find rule files, not commands or skills"
        );
        assert!(result.candidates[0].name.to_lowercase().contains("gemini"));
    }

    #[test]
    fn is_slash_command_or_skill_directory_detects_correctly() {
        // Should be detected as command/skill directories
        assert!(is_slash_command_or_skill_directory(Path::new(
            "/home/.claude/commands/test.md"
        )));
        assert!(is_slash_command_or_skill_directory(Path::new(
            "C:/Users/test/Documents/Cline/Workflows/workflow.md"
        )));
        assert!(is_slash_command_or_skill_directory(Path::new(
            "/home/.gemini/skills/my-skill/skill.md"
        )));

        // Should NOT be detected as command/skill directories (these are rule files)
        assert!(!is_slash_command_or_skill_directory(Path::new(
            "/home/.gemini/GEMINI.md"
        )));
        assert!(!is_slash_command_or_skill_directory(Path::new(
            "/home/.clinerules"
        )));
        assert!(!is_slash_command_or_skill_directory(Path::new(
            "/repo/.claude/CLAUDE.md"
        )));
    }

    // =====================================
    // RULE IMPORT TESTS
    // =====================================

    #[test]
    fn import_rule_happy_path_global() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let rule_file = temp_dir.path().join("GEMINI.md");
        fs::write(&rule_file, "# Test Rule\n\nThis is a test rule for import.").unwrap();

        let result = scan_directory_to_candidates(temp_dir.path(), 1024 * 1024, None);

        assert_eq!(result.candidates.len(), 1);
        assert_eq!(result.candidates[0].artifact_type, ImportArtifactType::Rule);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn import_rule_happy_path_local() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let local_rules = temp_dir.path().join(".gemini");
        fs::create_dir_all(&local_rules).unwrap();
        let rule_file = local_rules.join("GEMINI.md");
        fs::write(&rule_file, "# Local Rule\n\nThis is a local rule.").unwrap();

        let result = scan_directory_to_candidates(temp_dir.path(), 1024 * 1024, None);

        assert_eq!(result.candidates.len(), 1);
        assert_eq!(result.candidates[0].artifact_type, ImportArtifactType::Rule);
    }

    #[test]
    fn import_rule_failure_path_empty_file() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let rule_file = temp_dir.path().join("empty.md");
        fs::write(&rule_file, "").unwrap();

        let result = scan_directory_to_candidates(temp_dir.path(), 1024 * 1024, None);

        // Empty files should be imported but with empty content
        assert_eq!(result.candidates.len(), 1);
        assert_eq!(result.candidates[0].content.trim(), "");
    }

    #[test]
    fn import_rule_failure_path_too_large() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let rule_file = temp_dir.path().join("large.md");
        let large_content = "x".repeat(2000);
        fs::write(&rule_file, &large_content).unwrap();

        let result = scan_directory_to_candidates(temp_dir.path(), 1000, None);

        assert!(!result.errors.is_empty());
        assert!(result.errors[0].contains("exceeds max import size"));
    }

    #[test]
    fn import_rule_failure_path_invalid_utf8() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let rule_file = temp_dir.path().join("binary.md");
        fs::write(&rule_file, &[0xFF, 0xFE, 0x00, 0x01]).unwrap();

        let result = scan_directory_to_candidates(temp_dir.path(), 1024 * 1024, None);

        assert!(!result.errors.is_empty());
        assert!(result.errors.iter().any(|e| e.contains("not valid UTF-8")));
    }

    #[test]
    fn import_rule_with_frontmatter() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let rule_file = temp_dir.path().join("rule-with-frontmatter.md");
        let content = r#"---
name: My Custom Rule
scope: global
enabledAdapters:
  - claude-code
  - opencode
---

# My Custom Rule

This is the rule content.
"#;
        fs::write(&rule_file, content).unwrap();

        let result = scan_directory_to_candidates(temp_dir.path(), 1024 * 1024, None);

        assert_eq!(result.candidates.len(), 1);
        // The name is extracted from the frontmatter or filename
        assert!(!result.candidates[0].name.is_empty());
    }

    // =====================================
    // SLASH COMMAND IMPORT TESTS (exclusion)
    // =====================================

    #[test]
    fn import_excludes_global_slash_commands() {
        let temp_dir = tempfile::TempDir::new().unwrap();

        // Create global slash command location
        let commands_dir = temp_dir.path().join(".claude").join("commands");
        fs::create_dir_all(&commands_dir).unwrap();
        fs::write(
            commands_dir.join("my-command.md"),
            "# My Command\n\nCommand content",
        )
        .unwrap();

        // Also create a rule
        fs::write(temp_dir.path().join("CLAUDE.md"), "# Rule\n\nRule content").unwrap();

        let result = scan_directory_to_candidates(temp_dir.path(), 1024 * 1024, None);

        // Only rule should be found
        assert_eq!(result.candidates.len(), 1);
        assert!(result.candidates[0].name.to_lowercase().contains("claude"));
    }

    #[test]
    fn import_excludes_local_slash_commands() {
        let temp_dir = tempfile::TempDir::new().unwrap();

        // Create local slash command location
        let commands_dir = temp_dir.path().join("commands");
        fs::create_dir_all(&commands_dir).unwrap();
        fs::write(
            commands_dir.join("workflow.md"),
            "# Workflow\n\nWorkflow content",
        )
        .unwrap();

        // Also create a rule
        fs::write(temp_dir.path().join("AGENTS.md"), "# Rule\n\nRule content").unwrap();

        let result = scan_directory_to_candidates(temp_dir.path(), 1024 * 1024, None);

        // Only rule should be found
        assert_eq!(result.candidates.len(), 1);
        assert!(result.candidates[0].name.to_lowercase().contains("agents"));
    }

    #[test]
    fn import_excludes_cline_workflows() {
        let temp_dir = tempfile::TempDir::new().unwrap();

        // Cline Workflows directory
        let workflows_dir = temp_dir
            .path()
            .join("Documents")
            .join("Cline")
            .join("Workflows");
        fs::create_dir_all(&workflows_dir).unwrap();
        fs::write(
            workflows_dir.join("my-workflow.md"),
            "# Workflow\n\nContent",
        )
        .unwrap();

        // Rules directory
        let rules_dir = temp_dir
            .path()
            .join("Documents")
            .join("Cline")
            .join("Rules");
        fs::create_dir_all(&rules_dir).unwrap();
        fs::write(rules_dir.join("my-rule.md"), "# Rule\n\nContent").unwrap();

        let result = scan_directory_to_candidates(temp_dir.path(), 1024 * 1024, None);

        // Only rule should be found
        assert_eq!(result.candidates.len(), 1);
        assert!(result.candidates[0].name.to_lowercase().contains("rule"));
    }

    // =====================================
    // SKILL IMPORT TESTS (exclusion)
    // =====================================

    #[test]
    fn import_excludes_global_skills() {
        let temp_dir = tempfile::TempDir::new().unwrap();

        // Create global skill location
        let skills_dir = temp_dir
            .path()
            .join(".claude")
            .join("skills")
            .join("my-skill");
        fs::create_dir_all(&skills_dir).unwrap();
        fs::write(skills_dir.join("SKILL.md"), "# My Skill\n\nSkill content").unwrap();

        // Also create a rule
        fs::write(temp_dir.path().join("CLAUDE.md"), "# Rule\n\nRule content").unwrap();

        let result = scan_directory_to_candidates(temp_dir.path(), 1024 * 1024, None);

        // Only rule should be found
        assert_eq!(result.candidates.len(), 1);
        assert!(result.candidates[0].name.to_lowercase().contains("claude"));
    }

    #[test]
    fn import_excludes_local_skills() {
        let temp_dir = tempfile::TempDir::new().unwrap();

        // Create local skill location
        let skills_dir = temp_dir.path().join("skills");
        fs::create_dir_all(&skills_dir).unwrap();
        fs::write(skills_dir.join("my-skill.md"), "# Skill\n\nSkill content").unwrap();

        // Also create a rule
        fs::write(temp_dir.path().join("GEMINI.md"), "# Rule\n\nRule content").unwrap();

        let result = scan_directory_to_candidates(temp_dir.path(), 1024 * 1024, None);

        // Only rule should be found
        assert_eq!(result.candidates.len(), 1);
        assert!(result.candidates[0].name.to_lowercase().contains("gemini"));
    }

    #[test]
    fn import_excludes_cline_skills() {
        let temp_dir = tempfile::TempDir::new().unwrap();

        // Cline Skills directory
        let skills_dir = temp_dir
            .path()
            .join("Documents")
            .join("Cline")
            .join("Skills")
            .join("test");
        fs::create_dir_all(&skills_dir).unwrap();
        fs::write(skills_dir.join("skill.md"), "# Skill\n\nContent").unwrap();

        // Rules directory
        let rules_dir = temp_dir
            .path()
            .join("Documents")
            .join("Cline")
            .join("Rules");
        fs::create_dir_all(&rules_dir).unwrap();
        fs::write(rules_dir.join("rule.md"), "# Rule\n\nContent").unwrap();

        let result = scan_directory_to_candidates(temp_dir.path(), 1024 * 1024, None);

        // Only rule should be found
        assert_eq!(result.candidates.len(), 1);
        assert!(result.candidates[0].name.to_lowercase().contains("rule"));
    }

    // =====================================
    // MIXED ARTIFACT TESTS
    // =====================================

    #[test]
    fn import_mixed_artifacts_only_includes_rules() {
        let temp_dir = tempfile::TempDir::new().unwrap();

        // Create rule
        fs::write(temp_dir.path().join("GEMINI.md"), "# Rule\n\nRule content").unwrap();

        // Create slash command
        let commands_dir = temp_dir.path().join("commands");
        fs::create_dir_all(&commands_dir).unwrap();
        fs::write(commands_dir.join("cmd.md"), "# Command\n\nCommand content").unwrap();

        // Create skill
        let skills_dir = temp_dir.path().join("skills");
        fs::create_dir_all(&skills_dir).unwrap();
        fs::write(skills_dir.join("skill.md"), "# Skill\n\nSkill content").unwrap();

        // Create workflow
        let workflows_dir = temp_dir.path().join("workflows");
        fs::create_dir_all(&workflows_dir).unwrap();
        fs::write(
            workflows_dir.join("wf.md"),
            "# Workflow\n\nWorkflow content",
        )
        .unwrap();

        let result = scan_directory_to_candidates(temp_dir.path(), 1024 * 1024, None);

        // Only the rule should be found
        assert_eq!(result.candidates.len(), 1);
        assert_eq!(result.candidates[0].artifact_type, ImportArtifactType::Rule);
        assert!(result.candidates[0].name.to_lowercase().contains("gemini"));
    }

    #[test]
    fn import_multiple_rules_happy_path() {
        let temp_dir = tempfile::TempDir::new().unwrap();

        // Create multiple rule files
        fs::write(temp_dir.path().join("GEMINI.md"), "# Gemini Rule").unwrap();
        fs::write(temp_dir.path().join("AGENTS.md"), "# Agents Rule").unwrap();
        fs::write(temp_dir.path().join("CLAUDE.md"), "# Claude Rule").unwrap();

        let result = scan_directory_to_candidates(temp_dir.path(), 1024 * 1024, None);

        assert_eq!(result.candidates.len(), 3);
        assert!(result.errors.is_empty());

        // All should be rules
        for candidate in &result.candidates {
            assert_eq!(candidate.artifact_type, ImportArtifactType::Rule);
        }
    }

    // =====================================
    // DETECT ARTIFACT TYPE EDGE CASES
    // =====================================

    #[test]
    fn detect_artifact_type_case_insensitive() {
        // Uppercase
        assert_eq!(
            detect_artifact_type_from_path(Path::new("/home/.claude/COMMANDS/test.md")),
            ImportArtifactType::SlashCommand
        );

        // Mixed case
        assert_eq!(
            detect_artifact_type_from_path(Path::new("/home/.claude/Commands/test.md")),
            ImportArtifactType::SlashCommand
        );

        // Lowercase
        assert_eq!(
            detect_artifact_type_from_path(Path::new("/home/.claude/commands/test.md")),
            ImportArtifactType::SlashCommand
        );
    }

    #[test]
    fn detect_artifact_type_windows_paths() {
        // Windows-style paths
        assert_eq!(
            detect_artifact_type_from_path(Path::new("C:/Users/test/.claude/commands/test.md")),
            ImportArtifactType::SlashCommand
        );
        assert_eq!(
            detect_artifact_type_from_path(Path::new(
                "C:\\Users\\test\\.claude\\commands\\test.md"
            )),
            ImportArtifactType::SlashCommand
        );
        assert_eq!(
            detect_artifact_type_from_path(Path::new("C:/Users/test/.claude/CLAUDE.md")),
            ImportArtifactType::Rule
        );
    }

    #[test]
    fn detect_artifact_type_nested_directories() {
        // Deeply nested slash command
        assert_eq!(
            detect_artifact_type_from_path(Path::new(
                "/home/user/repo/.claude/commands/subdir/nested/cmd.md"
            )),
            ImportArtifactType::SlashCommand
        );

        // Deeply nested skill
        assert_eq!(
            detect_artifact_type_from_path(Path::new(
                "/home/user/repo/skills/category/my-skill/skill.md"
            )),
            ImportArtifactType::Skill
        );

        // Rule in subdirectory (should still be rule if no command/skill keywords)
        assert_eq!(
            detect_artifact_type_from_path(Path::new("/home/user/repo/docs/GEMINI.md")),
            ImportArtifactType::Rule
        );
    }
}
