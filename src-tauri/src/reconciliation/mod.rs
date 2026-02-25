//! Reconciliation engine for RuleWeaver.
//!
//! This module provides a desired-state reconciliation system that automatically
//! cleans up stale artifacts when rules/commands/skills are renamed, deleted, or retargeted.
//!
//! # Design Principles
//!
//! - **Desired State**: Computed from database artifacts
//! - **Actual State**: Scanned from filesystem
//! - **Idempotent**: Safe to run multiple times; produces same result
//! - **Dry-Run First**: All operations support preview mode
//! - **Audit Logging**: All operations are logged for diagnostics
//!
//! # Status
//!
//! This module exposes public APIs that are not yet integrated into the main application.
//! The `allow(dead_code)` attribute suppresses warnings during this transitional phase.

#![allow(dead_code)]

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::database::Database;
use crate::error::Result;
use crate::models::registry::{ArtifactType, REGISTRY};
use crate::models::{AdapterType, Scope};
use crate::path_resolver::PathResolver;
use crate::slash_commands::adapters::get_adapter;

const MAX_FILE_SIZE_BYTES: u64 = 10 * 1024 * 1024;

/// Represents the desired state of generated artifacts.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesiredState {
    /// All paths that should exist with their expected content hashes
    #[serde(default)]
    pub expected_paths: HashMap<String, ExpectedArtifact>,
}

/// An artifact that should exist in the desired state.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpectedArtifact {
    /// The adapter this artifact is for
    pub adapter: AdapterType,
    /// The type of artifact
    pub artifact_type: ArtifactType,
    /// The scope (global or local)
    pub scope: Scope,
    /// The repository root for local artifacts (None for global)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_root: Option<PathBuf>,
    /// Expected content hash
    pub content_hash: String,
    /// The actual content to write (not serialized, used internally)
    #[serde(skip)]
    pub content: Option<String>,
}

/// Represents actual filesystem state.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActualState {
    /// All generated artifacts currently on disk
    #[serde(default)]
    pub found_paths: HashMap<String, FoundArtifact>,
}

/// An artifact found on the filesystem.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FoundArtifact {
    /// The path to the artifact
    pub path: PathBuf,
    /// The adapter this artifact is for (inferred from path)
    pub adapter: Option<AdapterType>,
    /// The type of artifact (inferred from path)
    pub artifact_type: Option<ArtifactType>,
    /// The scope (global or local)
    pub scope: Option<Scope>,
    /// Current content hash
    pub content_hash: String,
}

/// Reconciliation plan.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReconcilePlan {
    /// Paths that need to be created
    #[serde(default)]
    pub to_create: Vec<ResolvedArtifact>,
    /// Paths that need to be updated
    #[serde(default)]
    pub to_update: Vec<ResolvedArtifact>,
    /// Paths that need to be removed (stale)
    #[serde(default)]
    pub to_remove: Vec<FoundArtifact>,
    /// Paths that are unchanged
    #[serde(default)]
    pub unchanged: Vec<PathBuf>,
}

/// A resolved artifact in the reconciliation plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedArtifact {
    pub path: PathBuf,
    pub adapter: AdapterType,
    pub artifact_type: ArtifactType,
    pub scope: Scope,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_root: Option<PathBuf>,
    pub content_hash: String,
    /// The actual content to write (not serialized, used internally)
    #[serde(skip)]
    pub content: Option<String>,
}

/// Result of a reconciliation operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReconcileResult {
    /// Whether the operation was successful
    pub success: bool,
    /// Number of artifacts created
    pub created: usize,
    /// Number of artifacts updated
    pub updated: usize,
    /// Number of artifacts removed
    pub removed: usize,
    /// Number of artifacts unchanged
    pub unchanged: usize,
    /// Errors encountered during reconciliation
    #[serde(default)]
    pub errors: Vec<String>,
    /// Non-fatal warnings
    #[serde(default)]
    pub warnings: Vec<String>,
}

/// A structured error from reconciliation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReconcileError {
    /// The operation that failed
    pub operation: ReconcileOperation,
    /// The path affected
    pub path: PathBuf,
    /// Error message
    pub message: String,
    /// Whether this is recoverable
    pub recoverable: bool,
}

/// Log entry for reconciliation operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReconcileLogEntry {
    pub timestamp: DateTime<Utc>,
    pub operation: ReconcileOperation,
    pub artifact_type: Option<ArtifactType>,
    pub adapter: Option<AdapterType>,
    pub scope: Option<Scope>,
    pub path: PathBuf,
    pub result: ReconcileResultType,
}

/// Type of reconciliation operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReconcileOperation {
    Create,
    Update,
    Remove,
    Check,
}

/// Result type for a single reconciliation action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReconcileResultType {
    Success,
    Failed,
    Skipped,
}

/// Engine for reconciling desired state with actual filesystem state.
pub struct ReconciliationEngine {
    db: Arc<Database>,
    path_resolver: PathResolver,
}

impl ReconciliationEngine {
    /// Create a new ReconciliationEngine.
    pub fn new(db: Arc<Database>) -> Result<Self> {
        let path_resolver = PathResolver::new()?;
        Ok(Self { db, path_resolver })
    }

    /// Compute desired state from all database artifacts.
    ///
    /// This scans all rules, commands, and skills in the database and computes
    /// what paths should exist for each artifact type.
    pub async fn compute_desired_state(&self) -> Result<DesiredState> {
        let mut desired = DesiredState::default();

        self.compute_desired_state_rules(&mut desired).await?;
        self.compute_desired_state_command_stubs(&mut desired).await?;
        self.compute_desired_state_slash_commands(&mut desired).await?;
        self.compute_desired_state_skills(&mut desired).await?;

        Ok(desired)
    }

    /// Compute desired state for rules.
    async fn compute_desired_state_rules(&self, desired: &mut DesiredState) -> Result<()> {
        let rules = self.db.get_all_rules().await?;

        for rule in rules {
            if !rule.enabled {
                continue;
            }

            for adapter in &rule.enabled_adapters {
                if REGISTRY
                    .validate_support(adapter, &rule.scope, ArtifactType::Rule)
                    .is_err()
                {
                    continue;
                }

                let content_hash = compute_content_hash(&rule.content);

                match rule.scope {
                    Scope::Global => {
                        if let Ok(resolved) = self.path_resolver.global_path(*adapter, ArtifactType::Rule) {
                            let path_str = resolved.path.to_string_lossy().to_string();
                            desired.expected_paths.insert(
                                path_str.clone(),
                                ExpectedArtifact {
                                    adapter: *adapter,
                                    artifact_type: ArtifactType::Rule,
                                    scope: Scope::Global,
                                    repo_root: None,
                                    content_hash: content_hash.clone(),
                                    content: Some(rule.content.clone()),
                                },
                            );
                        }
                    }
                    Scope::Local => {
                        if let Some(target_paths) = &rule.target_paths {
                            for target_path in target_paths {
                                if let Ok(resolved) = self.path_resolver.local_path(
                                    *adapter,
                                    ArtifactType::Rule,
                                    Path::new(target_path),
                                ) {
                                    let path_str = resolved.path.to_string_lossy().to_string();
                                    desired.expected_paths.insert(
                                        path_str.clone(),
                                        ExpectedArtifact {
                                            adapter: *adapter,
                                            artifact_type: ArtifactType::Rule,
                                            scope: Scope::Local,
                                            repo_root: Some(PathBuf::from(target_path)),
                                            content_hash: content_hash.clone(),
                                            content: Some(rule.content.clone()),
                                        },
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Compute desired state for command stubs (COMMANDS.md/COMMANDS.toml files).
    async fn compute_desired_state_command_stubs(&self, desired: &mut DesiredState) -> Result<()> {
        let commands = self.db.get_all_commands().await?;

        let exposed_commands: Vec<_> = commands.into_iter().filter(|c| c.expose_via_mcp).collect();
        if exposed_commands.is_empty() {
            return Ok(());
        }

        for adapter in AdapterType::all() {
            if REGISTRY
                .validate_support(&adapter, &Scope::Global, ArtifactType::CommandStub)
                .is_err()
            {
                continue;
            }

            if let Ok(resolved) = self.path_resolver.global_path(adapter, ArtifactType::CommandStub) {
                let content = self.format_command_stub_content(&adapter, &exposed_commands);
                let content_hash = compute_content_hash(&content);
                let path_str = resolved.path.to_string_lossy().to_string();

                desired.expected_paths.insert(
                    path_str.clone(),
                    ExpectedArtifact {
                        adapter,
                        artifact_type: ArtifactType::CommandStub,
                        scope: Scope::Global,
                        repo_root: None,
                        content_hash,
                        content: Some(content),
                    },
                );
            }
        }

        Ok(())
    }

    /// Compute desired state for slash commands (individual command files).
    async fn compute_desired_state_slash_commands(&self, desired: &mut DesiredState) -> Result<()> {
        let commands = self.db.get_all_commands().await?;

        for command in commands {
            if !command.generate_slash_commands {
                continue;
            }

            for adapter_name in &command.slash_command_adapters {
                let adapter_type = match AdapterType::from_str(adapter_name) {
                    Some(a) => a,
                    None => continue,
                };

                if REGISTRY
                    .validate_support(&adapter_type, &Scope::Global, ArtifactType::SlashCommand)
                    .is_err()
                {
                    continue;
                }

                let slash_adapter = match get_adapter(adapter_name) {
                    Some(a) => a,
                    None => continue,
                };

                let safe_name = match crate::slash_commands::sync::validate_command_name(&command.name) {
                    Ok(name) => name,
                    Err(_) => continue,
                };

                let content = slash_adapter.format_command(&command);
                let content_hash = compute_content_hash(&content);

                if let Ok(resolved) = self.path_resolver.slash_command_path(adapter_type, &safe_name, true) {
                    let path_str = resolved.path.to_string_lossy().to_string();
                    desired.expected_paths.insert(
                        path_str.clone(),
                        ExpectedArtifact {
                            adapter: adapter_type,
                            artifact_type: ArtifactType::SlashCommand,
                            scope: Scope::Global,
                            repo_root: None,
                            content_hash: content_hash.clone(),
                            content: Some(content.clone()),
                        },
                    );
                }

                for target_path in &command.target_paths {
                    if let Ok(resolved) = self.path_resolver.local_slash_command_path(
                        adapter_type,
                        &safe_name,
                        Path::new(target_path),
                    ) {
                        let path_str = resolved.path.to_string_lossy().to_string();
                        desired.expected_paths.insert(
                            path_str.clone(),
                            ExpectedArtifact {
                                adapter: adapter_type,
                                artifact_type: ArtifactType::SlashCommand,
                                scope: Scope::Local,
                                repo_root: Some(PathBuf::from(target_path)),
                                content_hash: content_hash.clone(),
                                content: Some(content.clone()),
                            },
                        );
                    }
                }
            }
        }

        Ok(())
    }

    /// Compute desired state for skills.
    async fn compute_desired_state_skills(&self, desired: &mut DesiredState) -> Result<()> {
        let skills = self.db.get_all_skills().await?;

        for skill in skills {
            if !skill.enabled {
                continue;
            }

            for adapter in AdapterType::all() {
                if REGISTRY
                    .validate_support(&adapter, &skill.scope, ArtifactType::Skill)
                    .is_err()
                {
                    continue;
                }

                let content = self.format_skill_content(&skill);
                let content_hash = compute_content_hash(&content);
                let safe_name = sanitize_skill_name_for_path(&skill.name);

                match skill.scope {
                    Scope::Global => {
                        if let Ok(resolved) = self.path_resolver.skill_path(adapter, &safe_name) {
                            let path_str = resolved.path.to_string_lossy().to_string();
                            desired.expected_paths.insert(
                                path_str.clone(),
                                ExpectedArtifact {
                                    adapter,
                                    artifact_type: ArtifactType::Skill,
                                    scope: Scope::Global,
                                    repo_root: None,
                                    content_hash: content_hash.clone(),
                                    content: Some(content.clone()),
                                },
                            );
                        }
                    }
                    Scope::Local => {
                        let repo_roots = self.path_resolver.repository_roots();
                        for repo_root in repo_roots {
                            if let Ok(resolved) = self.path_resolver.local_skill_path(adapter, &safe_name, repo_root) {
                                let path_str = resolved.path.to_string_lossy().to_string();
                                desired.expected_paths.insert(
                                    path_str.clone(),
                                    ExpectedArtifact {
                                        adapter,
                                        artifact_type: ArtifactType::Skill,
                                        scope: Scope::Local,
                                        repo_root: Some(repo_root.clone()),
                                        content_hash: content_hash.clone(),
                                        content: Some(content.clone()),
                                    },
                                );
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Format command stub content for a specific adapter.
    fn format_command_stub_content(&self, _adapter: &AdapterType, commands: &[crate::models::Command]) -> String {
        use std::collections::HashMap;
        
        #[derive(serde::Serialize)]
        struct CommandStubArg {
            arg_type: String,
            required: bool,
        }

        #[derive(serde::Serialize)]
        struct CommandStub {
            description: String,
            script: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            arguments: Option<HashMap<String, CommandStubArg>>,
        }

        #[derive(serde::Serialize)]
        struct CommandsFile {
            command: Vec<CommandStub>,
        }

        let stubs: Vec<CommandStub> = commands
            .iter()
            .map(|cmd| {
                let mut args = HashMap::new();
                for arg in &cmd.arguments {
                    args.insert(
                        arg.name.clone(),
                        CommandStubArg {
                            arg_type: "string".to_string(),
                            required: arg.required,
                        },
                    );
                }

                CommandStub {
                    description: cmd.description.clone(),
                    script: cmd.script.clone(),
                    arguments: if args.is_empty() { None } else { Some(args) },
                }
            })
            .collect();

        let file = CommandsFile { command: stubs };
        let toml_content = toml::to_string(&file).unwrap_or_default();
        
        format!(
            "# Generated by RuleWeaver - Do not edit manually\n# Last synced: {}\n\n{}",
            chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ"),
            toml_content
        )
    }

    /// Format skill content for writing to SKILL.md files.
    fn format_skill_content(&self, skill: &crate::models::Skill) -> String {
        let mut content = format!(
            "# {}\n\n{}\n\n## Instructions\n\n{}\n",
            skill.name,
            skill.description,
            skill.instructions
        );

        if !skill.input_schema.is_empty() {
            content.push_str("\n## Parameters\n\n");
            for param in &skill.input_schema {
                content.push_str(&format!(
                    "- **{}** ({}{}): {}\n",
                    param.name,
                    match param.param_type {
                        crate::models::SkillParameterType::String => "string",
                        crate::models::SkillParameterType::Number => "number",
                        crate::models::SkillParameterType::Boolean => "boolean",
                        crate::models::SkillParameterType::Enum => "enum",
                        crate::models::SkillParameterType::Array => "array",
                        crate::models::SkillParameterType::Object => "object",
                    },
                    if param.required { ", required" } else { "" },
                    param.description
                ));
            }
        }

        content.push_str(&format!(
            "\n## Entry Point\n\n`{}`\n",
            skill.entry_point
        ));

        content
    }

    /// Scan filesystem for actual state.
    ///
    /// This scans known paths for all adapters to find what artifacts currently exist.
    pub async fn scan_actual_state(&self) -> Result<ActualState> {
        let mut actual = ActualState::default();

        self.scan_actual_state_rules(&mut actual)?;
        self.scan_actual_state_command_stubs(&mut actual)?;
        self.scan_actual_state_slash_commands(&mut actual)?;
        self.scan_actual_state_skills(&mut actual)?;

        Ok(actual)
    }

    /// Scan for rule artifacts.
    fn scan_actual_state_rules(&self, actual: &mut ActualState) -> Result<()> {
        for adapter in AdapterType::all() {
            if let Ok(resolved) = self.path_resolver.global_path(adapter, ArtifactType::Rule) {
                if let Some(found) = self.scan_artifact_file(
                    &resolved.path,
                    Some(adapter),
                    Some(ArtifactType::Rule),
                    Scope::Global,
                )? {
                    actual.found_paths.insert(resolved.path.to_string_lossy().to_string(), found);
                }
            }
        }

        let repo_roots = self.path_resolver.repository_roots();
        for repo_root in repo_roots {
            for adapter in AdapterType::all() {
                if let Ok(resolved) = self.path_resolver.local_path(adapter, ArtifactType::Rule, repo_root) {
                    if let Some(found) = self.scan_artifact_file(
                        &resolved.path,
                        Some(adapter),
                        Some(ArtifactType::Rule),
                        Scope::Local,
                    )? {
                        actual.found_paths.insert(resolved.path.to_string_lossy().to_string(), found);
                    }
                }
            }
        }

        Ok(())
    }

    /// Scan for command stub artifacts (COMMANDS.md files).
    fn scan_actual_state_command_stubs(&self, actual: &mut ActualState) -> Result<()> {
        for adapter in AdapterType::all() {
            if let Ok(resolved) = self.path_resolver.global_path(adapter, ArtifactType::CommandStub) {
                if let Some(found) = self.scan_artifact_file(
                    &resolved.path,
                    Some(adapter),
                    Some(ArtifactType::CommandStub),
                    Scope::Global,
                )? {
                    actual.found_paths.insert(resolved.path.to_string_lossy().to_string(), found);
                }
            }
        }

        let repo_roots = self.path_resolver.repository_roots();
        for repo_root in repo_roots {
            for adapter in AdapterType::all() {
                if let Ok(resolved) = self.path_resolver.local_path(adapter, ArtifactType::CommandStub, repo_root) {
                    if let Some(found) = self.scan_artifact_file(
                        &resolved.path,
                        Some(adapter),
                        Some(ArtifactType::CommandStub),
                        Scope::Local,
                    )? {
                        actual.found_paths.insert(resolved.path.to_string_lossy().to_string(), found);
                    }
                }
            }
        }

        Ok(())
    }

    /// Scan for slash command artifacts.
    fn scan_actual_state_slash_commands(&self, actual: &mut ActualState) -> Result<()> {
        for adapter in AdapterType::all() {
            let entry = match REGISTRY.get(&adapter) {
                Some(e) => e,
                None => continue,
            };

            let extension = match entry.slash_command_extension {
                Some(ext) => ext,
                None => continue,
            };

            if let Some(global_dir) = entry.paths.global_commands_dir {
                let dir_path = self.path_resolver.home_dir().join(global_dir);
                self.scan_command_directory(&dir_path, adapter, extension, Scope::Global, actual)?;
            }
        }

        let repo_roots = self.path_resolver.repository_roots();
        for repo_root in repo_roots {
            for adapter in AdapterType::all() {
                let entry = match REGISTRY.get(&adapter) {
                    Some(e) => e,
                    None => continue,
                };

                let extension = match entry.slash_command_extension {
                    Some(ext) => ext,
                    None => continue,
                };

                if let Some(local_dir) = entry.paths.local_commands_dir {
                    let dir_path = repo_root.join(local_dir);
                    self.scan_command_directory(&dir_path, adapter, extension, Scope::Local, actual)?;
                }
            }
        }

        Ok(())
    }

    /// Scan a directory for command files.
    fn scan_command_directory(
        &self,
        dir: &Path,
        adapter: AdapterType,
        extension: &str,
        scope: Scope,
        actual: &mut ActualState,
    ) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }

        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return Ok(()),
        };

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().map(|e| e == extension).unwrap_or(false) {
                if let Some(found) = self.scan_artifact_file(
                    &path,
                    Some(adapter),
                    Some(ArtifactType::SlashCommand),
                    scope,
                )? {
                    actual.found_paths.insert(path.to_string_lossy().to_string(), found);
                }
            }
        }

        Ok(())
    }

    /// Scan for skill artifacts.
    fn scan_actual_state_skills(&self, actual: &mut ActualState) -> Result<()> {
        for adapter in AdapterType::all() {
            if let Ok(resolved) = self.path_resolver.skill_dir(adapter) {
                self.scan_skill_directory(&resolved.path, adapter, Scope::Global, actual)?;
            }
        }

        let repo_roots = self.path_resolver.repository_roots();
        for repo_root in repo_roots {
            for adapter in AdapterType::all() {
                if let Ok(resolved) = self.path_resolver.local_skill_dir(adapter, repo_root) {
                    self.scan_skill_directory(&resolved.path, adapter, Scope::Local, actual)?;
                }
            }
        }

        Ok(())
    }

    /// Scan a skill directory for SKILL.md files.
    fn scan_skill_directory(
        &self,
        dir: &Path,
        adapter: AdapterType,
        scope: Scope,
        actual: &mut ActualState,
    ) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }

        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return Ok(()),
        };

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let skill_file = path.join("SKILL.md");
                if skill_file.exists() {
                    if let Some(found) = self.scan_artifact_file(
                        &skill_file,
                        Some(adapter),
                        Some(ArtifactType::Skill),
                        scope,
                    )? {
                        actual.found_paths.insert(skill_file.to_string_lossy().to_string(), found);
                    }
                }
            }
        }

        Ok(())
    }

    fn scan_artifact_file(&self, path: &Path, adapter: Option<AdapterType>, artifact_type: Option<ArtifactType>, scope: Scope) -> Result<Option<FoundArtifact>> {
        if !path.exists() {
            return Ok(None);
        }

        let metadata = fs::metadata(path)?;
        if metadata.len() > MAX_FILE_SIZE_BYTES {
            log::warn!(
                "Skipping artifact {}: file size {} exceeds limit of {} bytes",
                path.display(),
                metadata.len(),
                MAX_FILE_SIZE_BYTES
            );
            return Ok(None);
        }

        let content = fs::read_to_string(path)?;
        let hash = compute_content_hash(&content);

        Ok(Some(FoundArtifact {
            path: path.to_path_buf(),
            adapter,
            artifact_type,
            scope: Some(scope),
            content_hash: hash,
        }))
    }

    /// Compare desired vs actual to produce a reconciliation plan.
    pub fn plan(&self, desired: &DesiredState, actual: &ActualState) -> ReconcilePlan {
        let mut plan = ReconcilePlan::default();

        // Find paths that should exist but don't (to create)
        for (path_str, expected) in &desired.expected_paths {
            if let Some(found) = actual.found_paths.get(path_str) {
                // Path exists, check if content matches
                if found.content_hash == expected.content_hash {
                    plan.unchanged.push(PathBuf::from(path_str));
                } else {
                    plan.to_update.push(ResolvedArtifact {
                        path: PathBuf::from(path_str),
                        adapter: expected.adapter,
                        artifact_type: expected.artifact_type,
                        scope: expected.scope,
                        repo_root: expected.repo_root.clone(),
                        content_hash: expected.content_hash.clone(),
                        content: expected.content.clone(),
                    });
                }
            } else {
                // Path doesn't exist, needs to be created
                plan.to_create.push(ResolvedArtifact {
                    path: PathBuf::from(path_str),
                    adapter: expected.adapter,
                    artifact_type: expected.artifact_type,
                    scope: expected.scope,
                    repo_root: expected.repo_root.clone(),
                    content_hash: expected.content_hash.clone(),
                    content: expected.content.clone(),
                });
            }
        }

        // Find paths that exist but shouldn't (to remove - stale artifacts)
        for found in actual.found_paths.values() {
            if !desired.expected_paths.contains_key(&found.path.to_string_lossy().to_string()) {
                plan.to_remove.push(found.clone());
            }
        }

        plan
    }

    /// Execute a reconciliation plan.
    ///
    /// If dry_run is true, no actual changes are made.
    pub async fn execute(&self, plan: &ReconcilePlan, dry_run: bool) -> Result<ReconcileResult> {
        let mut result = ReconcileResult {
            success: true,
            ..Default::default()
        };

        // Handle creates
        for artifact in &plan.to_create {
            if dry_run {
                log::info!("[DRY RUN] Would create: {}", artifact.path.display());
                result.created += 1;
            } else {
                match self.create_artifact(artifact).await {
                    Ok(()) => {
                        result.created += 1;
                        self.log_operation(
                            ReconcileOperation::Create,
                            Some(artifact.artifact_type),
                            Some(artifact.adapter),
                            Some(artifact.scope),
                            &artifact.path,
                            ReconcileResultType::Success,
                        )
                        .await;
                    }
                    Err(e) => {
                        result.success = false;
                        result.errors.push(format!("Failed to create {}: {}", artifact.path.display(), e));
                        self.log_operation(
                            ReconcileOperation::Create,
                            Some(artifact.artifact_type),
                            Some(artifact.adapter),
                            Some(artifact.scope),
                            &artifact.path,
                            ReconcileResultType::Failed,
                        )
                        .await;
                    }
                }
            }
        }

        // Handle updates
        for artifact in &plan.to_update {
            if dry_run {
                log::info!("[DRY RUN] Would update: {}", artifact.path.display());
                result.updated += 1;
            } else {
                match self.update_artifact(artifact).await {
                    Ok(()) => {
                        result.updated += 1;
                        self.log_operation(
                            ReconcileOperation::Update,
                            Some(artifact.artifact_type),
                            Some(artifact.adapter),
                            Some(artifact.scope),
                            &artifact.path,
                            ReconcileResultType::Success,
                        )
                        .await;
                    }
                    Err(e) => {
                        result.success = false;
                        result.errors.push(format!("Failed to update {}: {}", artifact.path.display(), e));
                        self.log_operation(
                            ReconcileOperation::Update,
                            Some(artifact.artifact_type),
                            Some(artifact.adapter),
                            Some(artifact.scope),
                            &artifact.path,
                            ReconcileResultType::Failed,
                        )
                        .await;
                    }
                }
            }
        }

        // Handle removes
        for artifact in &plan.to_remove {
            if dry_run {
                log::info!("[DRY RUN] Would remove: {}", artifact.path.display());
                result.removed += 1;
            } else {
                match fs::remove_file(&artifact.path) {
                    Ok(()) => {
                        result.removed += 1;
                        self.log_operation(
                            ReconcileOperation::Remove,
                            artifact.artifact_type,
                            artifact.adapter,
                            artifact.scope,
                            &artifact.path,
                            ReconcileResultType::Success,
                        )
                        .await;
                    }
                    Err(e) => {
                        result.success = false;
                        result.errors.push(format!("Failed to remove {}: {}", artifact.path.display(), e));
                        self.log_operation(
                            ReconcileOperation::Remove,
                            artifact.artifact_type,
                            artifact.adapter,
                            artifact.scope,
                            &artifact.path,
                            ReconcileResultType::Failed,
                        )
                        .await;
                    }
                }
            }
        }

        result.unchanged = plan.unchanged.len();

        Ok(result)
    }

    /// Full reconciliation in one call.
    ///
    /// This computes desired state, scans actual state, generates a plan, and executes it.
    pub async fn reconcile(&self, dry_run: bool) -> Result<ReconcileResult> {
        log::info!("Starting reconciliation (dry_run: {})", dry_run);

        let desired = self.compute_desired_state().await?;
        log::info!("Desired state: {} paths", desired.expected_paths.len());

        let actual = self.scan_actual_state().await?;
        log::info!("Actual state: {} paths", actual.found_paths.len());

        let plan = self.plan(&desired, &actual);
        log::info!(
            "Plan: {} to create, {} to update, {} to remove, {} unchanged",
            plan.to_create.len(),
            plan.to_update.len(),
            plan.to_remove.len(),
            plan.unchanged.len()
        );

        let result = self.execute(&plan, dry_run).await?;

        log::info!(
            "Reconciliation complete: {} created, {} updated, {} removed, {} unchanged",
            result.created,
            result.updated,
            result.removed,
            result.unchanged
        );

        Ok(result)
    }

    /// Create a single artifact with atomic write safety.
    async fn create_artifact(&self, artifact: &ResolvedArtifact) -> Result<()> {
        if let Some(parent) = artifact.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = artifact.content.clone().unwrap_or_else(|| {
            generate_placeholder_content(&artifact.adapter, artifact.artifact_type, artifact.scope)
        });

        write_atomic(&artifact.path, &content)
    }

    /// Update a single artifact with atomic write safety.
    async fn update_artifact(&self, artifact: &ResolvedArtifact) -> Result<()> {
        let content = artifact.content.clone().unwrap_or_else(|| {
            generate_placeholder_content(&artifact.adapter, artifact.artifact_type, artifact.scope)
        });

        write_atomic(&artifact.path, &content)
    }

    /// Repair orphaned artifacts by removing them.
    ///
    /// This scans for files that exist but shouldn't and removes them.
    pub async fn repair(&self, dry_run: bool) -> Result<ReconcileResult> {
        log::info!("Starting repair (dry_run: {})", dry_run);

        let desired = self.compute_desired_state().await?;
        let actual = self.scan_actual_state().await?;
        let plan = self.plan(&desired, &actual);

        let mut result = ReconcileResult {
            success: true,
            ..Default::default()
        };

        for artifact in &plan.to_remove {
            if dry_run {
                log::info!("[DRY RUN] Would remove orphan: {}", artifact.path.display());
                result.removed += 1;
            } else {
                match fs::remove_file(&artifact.path) {
                    Ok(()) => {
                        result.removed += 1;
                        self.log_operation(
                            ReconcileOperation::Remove,
                            artifact.artifact_type,
                            artifact.adapter,
                            artifact.scope,
                            &artifact.path,
                            ReconcileResultType::Success,
                        )
                        .await;
                    }
                    Err(e) => {
                        result.success = false;
                        result.errors.push(format!(
                            "Failed to remove orphan {}: {}",
                            artifact.path.display(),
                            e
                        ));
                    }
                }
            }
        }

        result.unchanged = plan.unchanged.len();
        log::info!("Repair complete: {} orphans removed", result.removed);

        Ok(result)
    }

    /// Check if reconciliation is needed.
    pub async fn needs_reconciliation(&self) -> Result<bool> {
        let desired = self.compute_desired_state().await?;
        let actual = self.scan_actual_state().await?;
        let plan = self.plan(&desired, &actual);
        Ok(!plan.to_create.is_empty() || !plan.to_update.is_empty() || !plan.to_remove.is_empty())
    }

    /// Get stale artifact paths for display.
    pub async fn get_stale_paths(&self) -> Result<Vec<FoundArtifact>> {
        let desired = self.compute_desired_state().await?;
        let actual = self.scan_actual_state().await?;
        let plan = self.plan(&desired, &actual);
        Ok(plan.to_remove)
    }

    /// Log a reconciliation operation to both console and database.
    async fn log_operation(
        &self,
        operation: ReconcileOperation,
        artifact_type: Option<ArtifactType>,
        adapter: Option<AdapterType>,
        scope: Option<Scope>,
        path: &Path,
        result: ReconcileResultType,
    ) {
        let entry = ReconcileLogEntry {
            timestamp: Utc::now(),
            operation,
            artifact_type,
            adapter,
            scope,
            path: path.to_path_buf(),
            result,
        };

        log::debug!("Reconciliation log: {:?}", entry);

        let operation_str = match operation {
            ReconcileOperation::Create => "create",
            ReconcileOperation::Update => "update",
            ReconcileOperation::Remove => "remove",
            ReconcileOperation::Check => "check",
        };

        let result_str = match result {
            ReconcileResultType::Success => "success",
            ReconcileResultType::Failed => "failed",
            ReconcileResultType::Skipped => "skipped",
        };

        let _ = self.db.log_reconciliation(
            operation_str,
            artifact_type.map(|a| a.as_str()),
            adapter.map(|a| a.as_str()),
            scope.map(|s| s.as_str()),
            &path.to_string_lossy(),
            result_str,
            None,
        ).await;
    }
}

/// Compute a content hash.
fn compute_content_hash(content: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Write content to a file atomically using temp file + rename.
///
/// This ensures that:
/// 1. Partial writes don't corrupt existing files
/// 2. Readers never see incomplete content
/// 3. Crashes during write leave either old or new content, never corrupted
fn write_atomic(path: &Path, content: &str) -> Result<()> {
    let temp_path = path.with_extension("tmp");

    fs::write(&temp_path, content).map_err(crate::error::AppError::Io)?;

    fs::rename(&temp_path, path).map_err(|e| {
        let _ = fs::remove_file(&temp_path);
        crate::error::AppError::Io(e)
    })?;

    Ok(())
}

/// Generate placeholder content for an artifact.
fn generate_placeholder_content(adapter: &AdapterType, artifact_type: ArtifactType, scope: Scope) -> String {
    format!(
        "# Generated by RuleWeaver\n# Adapter: {}\n# Artifact: {:?}\n# Scope: {:?}\n",
        adapter.as_str(),
        artifact_type,
        scope
    )
}

/// Sanitize a skill name for use in file paths.
fn sanitize_skill_name_for_path(name: &str) -> String {
    name.to_lowercase()
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
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_hash() {
        let hash1 = compute_content_hash("test content");
        let hash2 = compute_content_hash("test content");
        let hash3 = compute_content_hash("different content");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[tokio::test]
    async fn test_reconcile_plan_empty() {
        let db = crate::database::Database::new_in_memory().await.unwrap();
        let engine = ReconciliationEngine::new(std::sync::Arc::new(db)).unwrap();
        let desired = DesiredState::default();
        let actual = ActualState::default();
        
        let plan = engine.plan(&desired, &actual);
        
        assert!(plan.to_create.is_empty());
        assert!(plan.to_update.is_empty());
        assert!(plan.to_remove.is_empty());
        assert!(plan.unchanged.is_empty());
    }

    #[tokio::test]
    async fn test_plan_detects_creates() {
        let db = std::sync::Arc::new(crate::database::Database::new_in_memory().await.unwrap());
        let engine = ReconciliationEngine::new(db).unwrap();

        let mut desired = DesiredState::default();
        desired.expected_paths.insert(
            "/new/path.md".to_string(),
            ExpectedArtifact {
                adapter: AdapterType::ClaudeCode,
                artifact_type: ArtifactType::Rule,
                scope: Scope::Global,
                repo_root: None,
                content_hash: "hash123".to_string(),
                content: Some("content".to_string()),
            },
        );

        let actual = ActualState::default();
        let plan = engine.plan(&desired, &actual);

        assert_eq!(plan.to_create.len(), 1);
        assert!(plan.to_update.is_empty());
        assert!(plan.to_remove.is_empty());
    }

    #[tokio::test]
    async fn test_plan_detects_updates() {
        let db = std::sync::Arc::new(crate::database::Database::new_in_memory().await.unwrap());
        let engine = ReconciliationEngine::new(db).unwrap();

        let mut desired = DesiredState::default();
        desired.expected_paths.insert(
            "/existing/path.md".to_string(),
            ExpectedArtifact {
                adapter: AdapterType::ClaudeCode,
                artifact_type: ArtifactType::Rule,
                scope: Scope::Global,
                repo_root: None,
                content_hash: "new_hash".to_string(),
                content: Some("new content".to_string()),
            },
        );

        let mut actual = ActualState::default();
        actual.found_paths.insert(
            "/existing/path.md".to_string(),
            FoundArtifact {
                path: PathBuf::from("/existing/path.md"),
                adapter: Some(AdapterType::ClaudeCode),
                artifact_type: Some(ArtifactType::Rule),
                scope: Some(Scope::Global),
                content_hash: "old_hash".to_string(),
            },
        );

        let plan = engine.plan(&desired, &actual);

        assert!(plan.to_create.is_empty());
        assert_eq!(plan.to_update.len(), 1);
        assert!(plan.to_remove.is_empty());
    }

    #[tokio::test]
    async fn test_plan_detects_removes() {
        let db = std::sync::Arc::new(crate::database::Database::new_in_memory().await.unwrap());
        let engine = ReconciliationEngine::new(db).unwrap();

        let desired = DesiredState::default();

        let mut actual = ActualState::default();
        actual.found_paths.insert(
            "/stale/path.md".to_string(),
            FoundArtifact {
                path: PathBuf::from("/stale/path.md"),
                adapter: Some(AdapterType::ClaudeCode),
                artifact_type: Some(ArtifactType::Rule),
                scope: Some(Scope::Global),
                content_hash: "hash".to_string(),
            },
        );

        let plan = engine.plan(&desired, &actual);

        assert!(plan.to_create.is_empty());
        assert!(plan.to_update.is_empty());
        assert_eq!(plan.to_remove.len(), 1);
        assert_eq!(plan.to_remove[0].path, PathBuf::from("/stale/path.md"));
    }

    #[tokio::test]
    async fn test_plan_detects_unchanged() {
        let db = std::sync::Arc::new(crate::database::Database::new_in_memory().await.unwrap());
        let engine = ReconciliationEngine::new(db).unwrap();

        let mut desired = DesiredState::default();
        desired.expected_paths.insert(
            "/existing/path.md".to_string(),
            ExpectedArtifact {
                adapter: AdapterType::ClaudeCode,
                artifact_type: ArtifactType::Rule,
                scope: Scope::Global,
                repo_root: None,
                content_hash: "same_hash".to_string(),
                content: Some("content".to_string()),
            },
        );

        let mut actual = ActualState::default();
        actual.found_paths.insert(
            "/existing/path.md".to_string(),
            FoundArtifact {
                path: PathBuf::from("/existing/path.md"),
                adapter: Some(AdapterType::ClaudeCode),
                artifact_type: Some(ArtifactType::Rule),
                scope: Some(Scope::Global),
                content_hash: "same_hash".to_string(),
            },
        );

        let plan = engine.plan(&desired, &actual);

        assert!(plan.to_create.is_empty());
        assert!(plan.to_update.is_empty());
        assert!(plan.to_remove.is_empty());
        assert_eq!(plan.unchanged.len(), 1);
    }

    #[tokio::test]
    async fn test_plan_mixed_operations() {
        let db = std::sync::Arc::new(crate::database::Database::new_in_memory().await.unwrap());
        let engine = ReconciliationEngine::new(db).unwrap();

        let mut desired = DesiredState::default();
        desired.expected_paths.insert(
            "/new/path.md".to_string(),
            ExpectedArtifact {
                adapter: AdapterType::ClaudeCode,
                artifact_type: ArtifactType::Rule,
                scope: Scope::Global,
                repo_root: None,
                content_hash: "hash1".to_string(),
                content: Some("new".to_string()),
            },
        );
        desired.expected_paths.insert(
            "/update/path.md".to_string(),
            ExpectedArtifact {
                adapter: AdapterType::ClaudeCode,
                artifact_type: ArtifactType::Rule,
                scope: Scope::Global,
                repo_root: None,
                content_hash: "hash2_new".to_string(),
                content: Some("updated".to_string()),
            },
        );
        desired.expected_paths.insert(
            "/unchanged/path.md".to_string(),
            ExpectedArtifact {
                adapter: AdapterType::ClaudeCode,
                artifact_type: ArtifactType::Rule,
                scope: Scope::Global,
                repo_root: None,
                content_hash: "hash3".to_string(),
                content: Some("same".to_string()),
            },
        );

        let mut actual = ActualState::default();
        actual.found_paths.insert(
            "/update/path.md".to_string(),
            FoundArtifact {
                path: PathBuf::from("/update/path.md"),
                adapter: Some(AdapterType::ClaudeCode),
                artifact_type: Some(ArtifactType::Rule),
                scope: Some(Scope::Global),
                content_hash: "hash2_old".to_string(),
            },
        );
        actual.found_paths.insert(
            "/unchanged/path.md".to_string(),
            FoundArtifact {
                path: PathBuf::from("/unchanged/path.md"),
                adapter: Some(AdapterType::ClaudeCode),
                artifact_type: Some(ArtifactType::Rule),
                scope: Some(Scope::Global),
                content_hash: "hash3".to_string(),
            },
        );
        actual.found_paths.insert(
            "/stale/path.md".to_string(),
            FoundArtifact {
                path: PathBuf::from("/stale/path.md"),
                adapter: Some(AdapterType::ClaudeCode),
                artifact_type: Some(ArtifactType::Rule),
                scope: Some(Scope::Global),
                content_hash: "stale_hash".to_string(),
            },
        );

        let plan = engine.plan(&desired, &actual);

        assert_eq!(plan.to_create.len(), 1);
        assert_eq!(plan.to_update.len(), 1);
        assert_eq!(plan.to_remove.len(), 1);
        assert_eq!(plan.unchanged.len(), 1);
    }

    #[test]
    fn test_found_artifact_scope_preserved() {
        let artifact = FoundArtifact {
            path: PathBuf::from("/local/path.md"),
            adapter: Some(AdapterType::ClaudeCode),
            artifact_type: Some(ArtifactType::Rule),
            scope: Some(Scope::Local),
            content_hash: "hash".to_string(),
        };

        assert_eq!(artifact.scope, Some(Scope::Local));
    }

    #[tokio::test]
    async fn test_execute_dry_run_no_changes() {
        let db = crate::database::Database::new_in_memory().await.unwrap();
        let engine = ReconciliationEngine::new(std::sync::Arc::new(db)).unwrap();

        let mut plan = ReconcilePlan::default();
        plan.to_create.push(ResolvedArtifact {
            path: PathBuf::from("/nonexistent/path.md"),
            adapter: AdapterType::ClaudeCode,
            artifact_type: ArtifactType::Rule,
            scope: Scope::Global,
            repo_root: None,
            content_hash: "hash".to_string(),
            content: Some("content".to_string()),
        });

        let result = engine.execute(&plan, true).await.unwrap();

        assert!(result.success);
        assert_eq!(result.created, 1);
        assert!(!PathBuf::from("/nonexistent/path.md").exists());
    }

    #[test]
    fn test_reconcile_log_entry_serialization() {
        let entry = ReconcileLogEntry {
            timestamp: chrono::Utc::now(),
            operation: ReconcileOperation::Create,
            artifact_type: Some(ArtifactType::Rule),
            adapter: Some(AdapterType::ClaudeCode),
            scope: Some(Scope::Global),
            path: PathBuf::from("/test/path.md"),
            result: ReconcileResultType::Success,
        };

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("create"));
        assert!(json.contains("rule"));
        assert!(json.contains("claude") || json.contains("Claude"));
    }

    #[test]
    fn test_sanitize_skill_name_for_path() {
        assert_eq!(sanitize_skill_name_for_path("Test Skill"), "test-skill");
        assert_eq!(sanitize_skill_name_for_path("My-Cool_Skill"), "my-cool_skill");
        assert_eq!(sanitize_skill_name_for_path("Skill With  Spaces"), "skill-with--spaces");
        assert_eq!(sanitize_skill_name_for_path("--Leading--Trailing--"), "leading--trailing");
    }

    #[tokio::test]
    async fn test_compute_desired_state_includes_command_stubs() {
        let db = std::sync::Arc::new(crate::database::Database::new_in_memory().await.unwrap());
        
        db.create_command(crate::models::CreateCommandInput {
            id: None,
            name: "Test Command".to_string(),
            description: "A test command".to_string(),
            script: "echo test".to_string(),
            arguments: vec![],
            expose_via_mcp: true,
            is_placeholder: false,
            generate_slash_commands: false,
            slash_command_adapters: vec![],
            target_paths: vec![],
        }).await.unwrap();

        let engine = ReconciliationEngine::new(db).unwrap();
        let desired = engine.compute_desired_state().await.unwrap();

        let has_command_stub = desired.expected_paths.values().any(|a| {
            a.artifact_type == ArtifactType::CommandStub
        });
        assert!(has_command_stub, "Desired state should include command stub artifacts");
    }

    #[tokio::test]
    async fn test_compute_desired_state_includes_slash_commands() {
        let db = std::sync::Arc::new(crate::database::Database::new_in_memory().await.unwrap());
        
        db.create_command(crate::models::CreateCommandInput {
            id: None,
            name: "Test Slash Command".to_string(),
            description: "A test slash command".to_string(),
            script: "echo test".to_string(),
            arguments: vec![],
            expose_via_mcp: false,
            is_placeholder: false,
            generate_slash_commands: true,
            slash_command_adapters: vec!["claude-code".to_string()],
            target_paths: vec![],
        }).await.unwrap();

        let engine = ReconciliationEngine::new(db).unwrap();
        let desired = engine.compute_desired_state().await.unwrap();

        let has_slash_command = desired.expected_paths.values().any(|a| {
            a.artifact_type == ArtifactType::SlashCommand
        });
        assert!(has_slash_command, "Desired state should include slash command artifacts");
    }

    #[tokio::test]
    async fn test_compute_desired_state_includes_skills() {
        let db = std::sync::Arc::new(crate::database::Database::new_in_memory().await.unwrap());
        
        db.create_skill(crate::models::CreateSkillInput {
            id: None,
            name: "Test Skill".to_string(),
            description: "A test skill".to_string(),
            instructions: "echo test".to_string(),
            scope: crate::models::Scope::Global,
            input_schema: vec![],
            directory_path: "/test/skills".to_string(),
            entry_point: "main.sh".to_string(),
            enabled: true,
        }).await.unwrap();

        let engine = ReconciliationEngine::new(db).unwrap();
        let desired = engine.compute_desired_state().await.unwrap();

        let has_skill = desired.expected_paths.values().any(|a| {
            a.artifact_type == ArtifactType::Skill
        });
        assert!(has_skill, "Desired state should include skill artifacts");
    }

    #[tokio::test]
    async fn test_reconcile_all_artifact_types() {
        let db = std::sync::Arc::new(crate::database::Database::new_in_memory().await.unwrap());
        let engine = ReconciliationEngine::new(db).unwrap();

        let result = engine.reconcile(true).await.unwrap();
        
        assert!(result.success, "Dry-run reconciliation should succeed");
    }

    #[tokio::test]
    async fn test_reconcile_is_idempotent() {
        let db = std::sync::Arc::new(crate::database::Database::new_in_memory().await.unwrap());
        let engine = ReconciliationEngine::new(db.clone()).unwrap();

        // First run on empty database
        let r1 = engine.reconcile(false).await.unwrap();
        assert!(r1.success);

        // Second run should be a no-op (no changes)
        let engine2 = ReconciliationEngine::new(db).unwrap();
        let r2 = engine2.reconcile(false).await.unwrap();
        assert!(r2.success);
        assert_eq!(r2.created, 0, "Second run should not create anything");
        assert_eq!(r2.updated, 0, "Second run should not update anything");
        assert_eq!(r2.removed, 0, "Second run should not remove anything");
    }

    #[tokio::test]
    async fn test_needs_reconciliation_empty_db() {
        let db = std::sync::Arc::new(crate::database::Database::new_in_memory().await.unwrap());
        let engine = ReconciliationEngine::new(db).unwrap();

        // An empty database may still need reconciliation if there are stale files on disk
        // This is expected behavior - we check that the method works without error
        let _needs = engine.needs_reconciliation().await.unwrap();
    }

    #[tokio::test]
    async fn test_get_stale_paths_returns_vec() {
        let db = std::sync::Arc::new(crate::database::Database::new_in_memory().await.unwrap());
        let engine = ReconciliationEngine::new(db).unwrap();

        // Verify the method returns successfully (no panic/error)
        let stale = engine.get_stale_paths().await.unwrap();
        // The result depends on what files exist on disk, we just verify the method works
        println!("Found {} stale paths", stale.len());
    }

    #[tokio::test]
    async fn test_repair_dry_run_safe() {
        let db = std::sync::Arc::new(crate::database::Database::new_in_memory().await.unwrap());
        let engine = ReconciliationEngine::new(db).unwrap();

        // Dry-run should succeed regardless of filesystem state
        let result = engine.repair(true).await.unwrap();
        assert!(result.success, "Dry-run repair should succeed");
    }

    #[test]
    fn test_reconcile_result_has_warnings() {
        let mut result = ReconcileResult::default();
        result.warnings.push("Test warning".to_string());
        assert_eq!(result.warnings.len(), 1);
    }
}
