use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SecretScope {
    Global,
    Workspace,
    Command,
    Skill,
}

impl SecretScope {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Global => "global",
            Self::Workspace => "workspace",
            Self::Command => "command",
            Self::Skill => "skill",
        }
    }
}

impl FromStr for SecretScope {
    type Err = AppError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "global" => Ok(Self::Global),
            "workspace" => Ok(Self::Workspace),
            "command" => Ok(Self::Command),
            "skill" => Ok(Self::Skill),
            other => Err(AppError::InvalidInput {
                message: format!("Unknown secret scope: {}", other),
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScopedSecret {
    pub id: String,
    pub key: String,
    pub value: String,
    pub scope: SecretScope,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact_id: Option<String>,
    #[serde(with = "crate::models::timestamp")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "crate::models::timestamp")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertScopedSecretInput {
    pub key: String,
    pub value: String,
    pub scope: SecretScope,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteScopedSecretInput {
    pub key: String,
    pub scope: SecretScope,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolveScopedSecretsInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact_scope: Option<SecretScope>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectiveSecret {
    pub key: String,
    pub value: String,
    pub source_scope: SecretScope,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecretStorageStatus {
    pub backend: String,
    pub stores_secrets_in_os_credential_manager: bool,
    pub exports_include_secrets: bool,
    pub imports_include_secrets: bool,
}
