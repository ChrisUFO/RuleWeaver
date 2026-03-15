use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::registry::ArtifactType;
use crate::models::{AdapterType, Scope};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncManifestEntry {
    pub id: String,
    pub path: String,
    pub artifact_id: String,
    pub artifact_type: ArtifactType,
    pub adapter: AdapterType,
    pub scope: Scope,
    #[serde(with = "crate::models::timestamp")]
    pub written_at: DateTime<Utc>,
    pub content_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSyncManifestInput {
    pub id: Option<String>,
    pub path: String,
    pub artifact_id: String,
    pub artifact_type: ArtifactType,
    pub adapter: AdapterType,
    pub scope: Scope,
    pub content_hash: String,
}

impl CreateSyncManifestInput {
    pub fn into_entry(self) -> SyncManifestEntry {
        SyncManifestEntry {
            id: self.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            path: self.path,
            artifact_id: self.artifact_id,
            artifact_type: self.artifact_type,
            adapter: self.adapter,
            scope: self.scope,
            written_at: Utc::now(),
            content_hash: self.content_hash,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SyncManifestFilter {
    pub adapter: Option<AdapterType>,
    pub artifact_type: Option<ArtifactType>,
    pub artifact_id: Option<String>,
    pub scope: Option<Scope>,
}
