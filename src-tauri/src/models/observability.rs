use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ObservabilityEventType {
    McpLifecycle,
    McpClient,
    CommandRun,
    SkillRun,
}

impl ObservabilityEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::McpLifecycle => "mcp_lifecycle",
            Self::McpClient => "mcp_client",
            Self::CommandRun => "command_run",
            Self::SkillRun => "skill_run",
        }
    }
}

impl std::str::FromStr for ObservabilityEventType {
    type Err = AppError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "mcp_lifecycle" => Ok(Self::McpLifecycle),
            "mcp_client" => Ok(Self::McpClient),
            "command_run" => Ok(Self::CommandRun),
            "skill_run" => Ok(Self::SkillRun),
            other => Err(AppError::InvalidInput {
                message: format!("Unknown observability event type: {other}"),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ObservabilityEventStatus {
    Info,
    Started,
    Success,
    Warning,
    Error,
    Stopped,
}

impl ObservabilityEventStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Started => "started",
            Self::Success => "success",
            Self::Warning => "warning",
            Self::Error => "error",
            Self::Stopped => "stopped",
        }
    }
}

impl std::str::FromStr for ObservabilityEventStatus {
    type Err = AppError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "info" => Ok(Self::Info),
            "started" => Ok(Self::Started),
            "success" => Ok(Self::Success),
            "warning" => Ok(Self::Warning),
            "error" => Ok(Self::Error),
            "stopped" => Ok(Self::Stopped),
            other => Err(AppError::InvalidInput {
                message: format!("Unknown observability event status: {other}"),
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObservabilityEvent {
    pub id: String,
    #[serde(with = "crate::models::timestamp")]
    pub timestamp: DateTime<Utc>,
    pub event_type: ObservabilityEventType,
    pub status: ObservabilityEventStatus,
    pub source: String,
    pub entity_kind: Option<String>,
    pub entity_id: Option<String>,
    pub entity_name: Option<String>,
    pub workspace_path: Option<String>,
    pub summary: String,
    pub metadata: Option<String>,
    pub stdout_excerpt: Option<String>,
    pub stderr_excerpt: Option<String>,
    pub duration_ms: Option<u64>,
    pub exit_code: Option<i32>,
    pub failure_class: Option<String>,
    pub attempt_number: Option<u8>,
    pub is_redacted: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObservabilityEventFilter {
    pub event_type: Option<ObservabilityEventType>,
    pub status: Option<ObservabilityEventStatus>,
    pub source: Option<String>,
    pub entity_name: Option<String>,
    pub workspace_path: Option<String>,
    pub query: Option<String>,
    #[serde(default, with = "crate::models::timestamp::optional")]
    pub from_timestamp: Option<DateTime<Utc>>,
    #[serde(default, with = "crate::models::timestamp::optional")]
    pub to_timestamp: Option<DateTime<Utc>>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObservabilityExport {
    pub version: String,
    #[serde(with = "crate::models::timestamp")]
    pub exported_at: DateTime<Utc>,
    pub event_count: usize,
    pub events: Vec<ObservabilityEvent>,
}
