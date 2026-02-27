//! Unified Artifact Status Engine for RuleWeaver.
//!
//! This module provides a unified status layer for all artifact types (rules, commands, skills),
//! built on top of the reconciliation engine outputs. It provides a status projection and
//! repair action surface without duplicating truth sources.

use std::str::FromStr;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::database::Database;
use crate::error::Result;
use crate::models::registry::ArtifactType;
use crate::models::{AdapterType, ParseEnumError, Scope};
use crate::reconciliation::{FoundArtifact, ReconciliationEngine};

pub mod commands;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactSyncStatus {
    Synced,
    OutOfDate,
    Missing,
    Conflicted,
    Unsupported,
    Error,
}

#[allow(dead_code)]
impl ArtifactSyncStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ArtifactSyncStatus::Synced => "synced",
            ArtifactSyncStatus::OutOfDate => "out_of_date",
            ArtifactSyncStatus::Missing => "missing",
            ArtifactSyncStatus::Conflicted => "conflicted",
            ArtifactSyncStatus::Unsupported => "unsupported",
            ArtifactSyncStatus::Error => "error",
        }
    }
}

impl FromStr for ArtifactSyncStatus {
    type Err = ParseEnumError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "synced" => Ok(ArtifactSyncStatus::Synced),
            "out_of_date" => Ok(ArtifactSyncStatus::OutOfDate),
            "missing" => Ok(ArtifactSyncStatus::Missing),
            "conflicted" => Ok(ArtifactSyncStatus::Conflicted),
            "unsupported" => Ok(ArtifactSyncStatus::Unsupported),
            "error" => Ok(ArtifactSyncStatus::Error),
            _ => Err(ParseEnumError),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactStatusEntry {
    pub id: String,
    pub artifact_id: String,
    pub artifact_name: String,
    pub artifact_type: ArtifactType,
    pub adapter: AdapterType,
    pub scope: Scope,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_root: Option<String>,
    pub status: ArtifactSyncStatus,
    pub expected_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_operation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_operation_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact_type: Option<ArtifactType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adapter: Option<AdapterType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<Scope>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_root: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ArtifactSyncStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepairResult {
    pub entry_id: String,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub updated_entry: Option<ArtifactStatusEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusSummary {
    pub total: usize,
    pub synced: usize,
    pub out_of_date: usize,
    pub missing: usize,
    pub conflicted: usize,
    pub unsupported: usize,
    pub error: usize,
}

impl StatusSummary {
    pub fn from_entries(entries: &[ArtifactStatusEntry]) -> Self {
        let mut summary = StatusSummary {
            total: entries.len(),
            synced: 0,
            out_of_date: 0,
            missing: 0,
            conflicted: 0,
            unsupported: 0,
            error: 0,
        };

        for entry in entries {
            match entry.status {
                ArtifactSyncStatus::Synced => summary.synced += 1,
                ArtifactSyncStatus::OutOfDate => summary.out_of_date += 1,
                ArtifactSyncStatus::Missing => summary.missing += 1,
                ArtifactSyncStatus::Conflicted => summary.conflicted += 1,
                ArtifactSyncStatus::Unsupported => summary.unsupported += 1,
                ArtifactSyncStatus::Error => summary.error += 1,
            }
        }

        summary
    }
}

pub struct StatusEngine {
    db: Arc<Database>,
    reconciliation_engine: ReconciliationEngine,
}

impl StatusEngine {
    pub fn new(db: Arc<Database>) -> Result<Self> {
        let reconciliation_engine = ReconciliationEngine::new(db.clone())?;
        Ok(Self {
            db,
            reconciliation_engine,
        })
    }

    pub async fn compute_status(&self, filter: &StatusFilter) -> Result<Vec<ArtifactStatusEntry>> {
        let desired = self.reconciliation_engine.compute_desired_state().await?;
        let actual = self.reconciliation_engine.scan_actual_state().await?;
        let last_ops = self.get_last_operations_by_path().await?;

        let mut entries = Vec::new();

        for (path_str, expected) in &desired.expected_paths {
            if !self.matches_filter(
                expected.adapter,
                expected.artifact_type,
                expected.scope,
                expected
                    .repo_root
                    .as_ref()
                    .map(|p| p.to_string_lossy().to_string())
                    .as_ref(),
                filter,
            ) {
                continue;
            }

            let artifact_info = self.get_artifact_info(expected);

            let (status, detail) = if let Some(found) = actual.found_paths.get(path_str) {
                if found.content_hash == expected.content_hash {
                    (ArtifactSyncStatus::Synced, None)
                } else {
                    (
                        ArtifactSyncStatus::OutOfDate,
                        Some("Content differs from expected".to_string()),
                    )
                }
            } else {
                (
                    ArtifactSyncStatus::Missing,
                    Some("File not found on disk".to_string()),
                )
            };

            let last_op_key = path_str.clone();
            let (last_operation, last_operation_at) = last_ops
                .get(&last_op_key)
                .map(|(op, ts)| (Some(op.clone()), Some(*ts)))
                .unwrap_or((None, None));

            let entry_id = format!(
                "{}-{}-{}",
                expected.artifact_type.as_str(),
                expected.adapter.as_str(),
                path_str.replace(['/', '\\', '.'], "-")
            );

            entries.push(ArtifactStatusEntry {
                id: entry_id,
                artifact_id: artifact_info.0.clone(),
                artifact_name: artifact_info.1.clone(),
                artifact_type: expected.artifact_type,
                adapter: expected.adapter,
                scope: expected.scope,
                repo_root: expected
                    .repo_root
                    .as_ref()
                    .map(|p| p.to_string_lossy().to_string()),
                status,
                expected_path: path_str.clone(),
                last_operation,
                last_operation_at,
                detail,
            });
        }

        for (path_str, found) in &actual.found_paths {
            if desired.expected_paths.contains_key(path_str) {
                continue;
            }

            let artifact_type = match found.artifact_type {
                Some(t) => t,
                None => continue,
            };
            let adapter = match found.adapter {
                Some(a) => a,
                None => continue,
            };
            let scope = found.scope.unwrap_or(Scope::Global);

            if !self.matches_filter(
                adapter,
                artifact_type,
                scope,
                found.scope.map(|_| String::new()).as_ref(),
                filter,
            ) {
                continue;
            }

            if let Some(ref filter_status) = filter.status {
                if *filter_status != ArtifactSyncStatus::Conflicted {
                    continue;
                }
            }

            let artifact_info = self.get_artifact_info_from_found(found).await;

            let last_op_key = path_str.clone();
            let (last_operation, last_operation_at) = last_ops
                .get(&last_op_key)
                .map(|(op, ts)| (Some(op.clone()), Some(*ts)))
                .unwrap_or((None, None));

            let entry_id = format!(
                "{}-{}-{}",
                artifact_type.as_str(),
                adapter.as_str(),
                path_str.replace(['/', '\\', '.'], "-")
            );

            entries.push(ArtifactStatusEntry {
                id: entry_id,
                artifact_id: artifact_info.0.clone(),
                artifact_name: artifact_info.1.clone(),
                artifact_type,
                adapter,
                scope,
                repo_root: None,
                status: ArtifactSyncStatus::Conflicted,
                expected_path: path_str.clone(),
                last_operation,
                last_operation_at,
                detail: Some("Orphaned file not in desired state".to_string()),
            });
        }

        let mut filtered_entries = Vec::new();
        if let Some(ref filter_status) = filter.status {
            for entry in entries {
                if entry.status == *filter_status {
                    filtered_entries.push(entry);
                }
            }
        } else {
            filtered_entries = entries;
        }

        Ok(filtered_entries)
    }

    fn matches_filter(
        &self,
        adapter: AdapterType,
        artifact_type: ArtifactType,
        scope: Scope,
        repo_root: Option<&String>,
        filter: &StatusFilter,
    ) -> bool {
        if let Some(ref ft) = filter.artifact_type {
            if *ft != artifact_type {
                return false;
            }
        }
        if let Some(ref fa) = filter.adapter {
            if *fa != adapter {
                return false;
            }
        }
        if let Some(ref fs) = filter.scope {
            if *fs != scope {
                return false;
            }
        }
        if let Some(ref fr) = filter.repo_root {
            if repo_root.map(|r| !r.contains(fr)).unwrap_or(true) {
                return false;
            }
        }
        true
    }

    fn get_artifact_info(
        &self,
        expected: &crate::reconciliation::ExpectedArtifact,
    ) -> (String, String) {
        (expected.id.clone(), expected.name.clone())
    }

    async fn get_artifact_info_from_found(&self, found: &FoundArtifact) -> (String, String) {
        let file_name = found
            .path
            .file_stem()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        let artifact_id = format!(
            "{}-{}",
            found.artifact_type.map(|t| t.as_str()).unwrap_or("unknown"),
            &file_name
        );

        (artifact_id, file_name)
    }

    async fn get_last_operations_by_path(
        &self,
    ) -> Result<std::collections::HashMap<String, (String, DateTime<Utc>)>> {
        self.db.get_last_reconciliation_op_per_path().await
    }

    pub async fn repair_artifact(&self, entry_id: &str) -> Result<RepairResult> {
        let filter = StatusFilter::default();
        let entries = self.compute_status(&filter).await?;

        let entry = entries.iter().find(|e| e.id == entry_id);

        match entry {
            Some(e) => {
                let result = self
                    .reconciliation_engine
                    .reconcile(false, Some(e.expected_path.clone()))
                    .await;

                match result {
                    Ok(_) => {
                        let updated_entries = self.compute_status(&filter).await?;
                        let updated_entry =
                            updated_entries.iter().find(|ue| ue.id == entry_id).cloned();

                        Ok(RepairResult {
                            entry_id: entry_id.to_string(),
                            success: true,
                            error: None,
                            updated_entry,
                        })
                    }
                    Err(e) => Ok(RepairResult {
                        entry_id: entry_id.to_string(),
                        success: false,
                        error: Some(e.to_string()),
                        updated_entry: None,
                    }),
                }
            }
            None => Ok(RepairResult {
                entry_id: entry_id.to_string(),
                success: false,
                error: Some("Artifact entry not found".to_string()),
                updated_entry: None,
            }),
        }
    }

    pub async fn repair_all_artifacts(&self, filter: &StatusFilter) -> Result<Vec<RepairResult>> {
        let entries = self.compute_status(filter).await?;
        let needs_repair: Vec<&ArtifactStatusEntry> = entries
            .iter()
            .filter(|e| {
                e.status != ArtifactSyncStatus::Synced
                    && e.status != ArtifactSyncStatus::Unsupported
            })
            .collect();

        if needs_repair.is_empty() {
            return Ok(vec![]);
        }

        let result = self.reconciliation_engine.reconcile(false, None).await;

        let updated_entries = self.compute_status(filter).await?;

        let results: Vec<RepairResult> = needs_repair
            .iter()
            .map(|entry| {
                let updated_entry = updated_entries.iter().find(|ue| ue.id == entry.id).cloned();

                RepairResult {
                    entry_id: entry.id.clone(),
                    success: result.is_ok(),
                    error: result.as_ref().err().map(|e| e.to_string()),
                    updated_entry,
                }
            })
            .collect();

        Ok(results)
    }

    pub async fn get_summary(&self, filter: &StatusFilter) -> Result<StatusSummary> {
        let entries = self.compute_status(filter).await?;
        Ok(StatusSummary::from_entries(&entries))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_artifact_sync_status_as_str() {
        assert_eq!(ArtifactSyncStatus::Synced.as_str(), "synced");
        assert_eq!(ArtifactSyncStatus::OutOfDate.as_str(), "out_of_date");
        assert_eq!(ArtifactSyncStatus::Missing.as_str(), "missing");
        assert_eq!(ArtifactSyncStatus::Conflicted.as_str(), "conflicted");
        assert_eq!(ArtifactSyncStatus::Unsupported.as_str(), "unsupported");
        assert_eq!(ArtifactSyncStatus::Error.as_str(), "error");
    }

    #[test]
    fn test_artifact_sync_status_from_str() {
        assert_eq!(
            ArtifactSyncStatus::from_str("synced"),
            Ok(ArtifactSyncStatus::Synced)
        );
        assert_eq!(
            ArtifactSyncStatus::from_str("out_of_date"),
            Ok(ArtifactSyncStatus::OutOfDate)
        );
        assert_eq!(
            ArtifactSyncStatus::from_str("missing"),
            Ok(ArtifactSyncStatus::Missing)
        );
        assert_eq!(
            ArtifactSyncStatus::from_str("conflicted"),
            Ok(ArtifactSyncStatus::Conflicted)
        );
        assert_eq!(
            ArtifactSyncStatus::from_str("unsupported"),
            Ok(ArtifactSyncStatus::Unsupported)
        );
        assert_eq!(
            ArtifactSyncStatus::from_str("error"),
            Ok(ArtifactSyncStatus::Error)
        );
        assert!(ArtifactSyncStatus::from_str("invalid").is_err());
    }

    #[test]
    fn test_status_summary_from_entries() {
        let entries = vec![
            ArtifactStatusEntry {
                id: "1".to_string(),
                artifact_id: "a1".to_string(),
                artifact_name: "Test 1".to_string(),
                artifact_type: ArtifactType::Rule,
                adapter: AdapterType::ClaudeCode,
                scope: Scope::Global,
                repo_root: None,
                status: ArtifactSyncStatus::Synced,
                expected_path: "/path/1".to_string(),
                last_operation: None,
                last_operation_at: None,
                detail: None,
            },
            ArtifactStatusEntry {
                id: "2".to_string(),
                artifact_id: "a2".to_string(),
                artifact_name: "Test 2".to_string(),
                artifact_type: ArtifactType::Rule,
                adapter: AdapterType::ClaudeCode,
                scope: Scope::Global,
                repo_root: None,
                status: ArtifactSyncStatus::Missing,
                expected_path: "/path/2".to_string(),
                last_operation: None,
                last_operation_at: None,
                detail: None,
            },
            ArtifactStatusEntry {
                id: "3".to_string(),
                artifact_id: "a3".to_string(),
                artifact_name: "Test 3".to_string(),
                artifact_type: ArtifactType::Skill,
                adapter: AdapterType::Gemini,
                scope: Scope::Global,
                repo_root: None,
                status: ArtifactSyncStatus::OutOfDate,
                expected_path: "/path/3".to_string(),
                last_operation: None,
                last_operation_at: None,
                detail: None,
            },
        ];

        let summary = StatusSummary::from_entries(&entries);
        assert_eq!(summary.total, 3);
        assert_eq!(summary.synced, 1);
        assert_eq!(summary.missing, 1);
        assert_eq!(summary.out_of_date, 1);
        assert_eq!(summary.conflicted, 0);
        assert_eq!(summary.unsupported, 0);
        assert_eq!(summary.error, 0);
    }

    #[test]
    fn test_status_filter_default() {
        let filter = StatusFilter::default();
        assert!(filter.artifact_type.is_none());
        assert!(filter.adapter.is_none());
        assert!(filter.scope.is_none());
        assert!(filter.repo_root.is_none());
        assert!(filter.status.is_none());
    }
}
