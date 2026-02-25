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
}

/// Log entry for reconciliation operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReconcileLogEntry {
    pub timestamp: DateTime<Utc>,
    pub operation: ReconcileOperation,
    pub artifact_type: Option<ArtifactType>,
    pub adapter: Option<AdapterType>,
    pub scope: Scope,
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

    /// Compute desired state from database rules.
    ///
    /// This scans all rules in the database and computes what paths should exist.
    ///
    /// TODO: Currently only handles `ArtifactType::Rule`. Support for other artifact types
    /// (`CommandStub`, `SlashCommand`, `Skill`) is pending and will be added in future phases.
    pub async fn compute_desired_state(&self) -> Result<DesiredState> {
        let mut desired = DesiredState::default();

        // Get all rules from database
        let rules = self.db.get_all_rules().await?;

        for rule in rules {
            if !rule.enabled {
                continue;
            }

            // For each enabled adapter
            for adapter in &rule.enabled_adapters {
                // Skip adapters that don't support rules
                if REGISTRY
                    .validate_support(adapter, &rule.scope, ArtifactType::Rule)
                    .is_err()
                {
                    continue;
                }

                let content_hash = compute_content_hash(&rule.content);

                match rule.scope {
                    Scope::Global => {
                        // Global rules go to a single path per adapter
                        let resolved = self.path_resolver.global_path(*adapter, ArtifactType::Rule)?;
                        let path_str = resolved.path.to_string_lossy().to_string();

                        desired.expected_paths.insert(
                            path_str.clone(),
                            ExpectedArtifact {
                                adapter: *adapter,
                                artifact_type: ArtifactType::Rule,
                                scope: Scope::Global,
                                repo_root: None,
                                content_hash,
                                content: Some(rule.content.clone()),
                            },
                        );
                    }
                    Scope::Local => {
                        // Local rules go to each target path
                        if let Some(target_paths) = &rule.target_paths {
                            for target_path in target_paths {
                                let resolved = self.path_resolver.local_path(
                                    *adapter,
                                    ArtifactType::Rule,
                                    Path::new(target_path),
                                )?;
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

        Ok(desired)
    }

    /// Scan filesystem for actual state.
    ///
    /// This scans known paths for all adapters to find what artifacts currently exist.
    pub async fn scan_actual_state(&self) -> Result<ActualState> {
        let mut actual = ActualState::default();

        // Scan global paths for all adapters
        for adapter in AdapterType::all() {
            if let Ok(resolved) = self.path_resolver.global_path(adapter, ArtifactType::Rule) {
                if let Some(found) = self.scan_artifact_file(&resolved.path, Some(adapter), Some(ArtifactType::Rule))? {
                    actual.found_paths.insert(resolved.path.to_string_lossy().to_string(), found);
                }
            }
        }

        // Scan local paths for configured repository roots
        let repo_roots = self.path_resolver.repository_roots();
        for repo_root in repo_roots {
            for adapter in AdapterType::all() {
                if let Ok(resolved) = self.path_resolver.local_path(adapter, ArtifactType::Rule, repo_root) {
                    if let Some(found) = self.scan_artifact_file(&resolved.path, Some(adapter), Some(ArtifactType::Rule))? {
                        actual.found_paths.insert(resolved.path.to_string_lossy().to_string(), found);
                    }
                }
            }
        }

        Ok(actual)
    }

    fn scan_artifact_file(&self, path: &Path, adapter: Option<AdapterType>, artifact_type: Option<ArtifactType>) -> Result<Option<FoundArtifact>> {
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
                            artifact.scope,
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
                            artifact.scope,
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
                            artifact.scope,
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
                            artifact.scope,
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
                            Scope::Global,
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
                            Scope::Global,
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

    /// Create a single artifact.
    async fn create_artifact(&self, artifact: &ResolvedArtifact) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = artifact.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = artifact.content.clone().unwrap_or_else(|| {
            generate_placeholder_content(&artifact.adapter, artifact.artifact_type, artifact.scope)
        });

        fs::write(&artifact.path, content)?;

        Ok(())
    }

    /// Update a single artifact.
    async fn update_artifact(&self, artifact: &ResolvedArtifact) -> Result<()> {
        let content = artifact.content.clone().unwrap_or_else(|| {
            generate_placeholder_content(&artifact.adapter, artifact.artifact_type, artifact.scope)
        });

        fs::write(&artifact.path, content)?;

        Ok(())
    }

    /// Log a reconciliation operation.
    async fn log_operation(
        &self,
        operation: ReconcileOperation,
        artifact_type: Option<ArtifactType>,
        adapter: Option<AdapterType>,
        scope: Scope,
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
    }
}

/// Compute a content hash.
fn compute_content_hash(content: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
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
}