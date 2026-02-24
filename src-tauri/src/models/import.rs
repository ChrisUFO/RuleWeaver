use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::{AdapterType, Rule, Scope};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ImportSourceType {
    AiTool,
    File,
    Directory,
    Url,
    Clipboard,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ImportConflictMode {
    Skip,
    Rename,
    Replace,
}

impl Default for ImportConflictMode {
    fn default() -> Self {
        Self::Skip
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportCandidate {
    pub id: String,
    pub source_type: ImportSourceType,
    pub source_label: String,
    pub source_path: String,
    pub source_tool: Option<AdapterType>,
    pub name: String,
    pub proposed_name: String,
    pub content: String,
    pub scope: Scope,
    pub target_paths: Option<Vec<String>>,
    pub enabled_adapters: Vec<AdapterType>,
    pub content_hash: String,
    pub file_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportSkip {
    pub candidate_id: String,
    pub name: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportConflict {
    pub candidate_id: String,
    pub candidate_name: String,
    pub existing_rule_id: Option<String>,
    pub existing_rule_name: Option<String>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ImportScanResult {
    pub candidates: Vec<ImportCandidate>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ImportExecutionResult {
    pub imported: Vec<Rule>,
    pub skipped: Vec<ImportSkip>,
    pub conflicts: Vec<ImportConflict>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ImportExecutionOptions {
    #[serde(default)]
    pub conflict_mode: ImportConflictMode,
    pub default_scope: Option<Scope>,
    pub default_adapters: Option<Vec<AdapterType>>,
    pub selected_candidate_ids: Option<Vec<String>>,
    pub max_file_size_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportHistoryEntry {
    pub id: String,
    #[serde(with = "crate::models::timestamp")]
    pub timestamp: DateTime<Utc>,
    pub source_type: ImportSourceType,
    pub imported_count: usize,
    pub skipped_count: usize,
    pub conflict_count: usize,
    pub error_count: usize,
}
