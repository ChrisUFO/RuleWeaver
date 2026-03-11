use std::path::Path;

use crate::database::{Database, ExecutionLogInput, ObservabilityEventInput};
use crate::error::Result;
use crate::models::{
    ObservabilityEventFilter, ObservabilityEventStatus, ObservabilityEventType,
    ObservabilityExport, Skill,
};

const MAX_EXCERPT_CHARS: usize = 2000;

pub struct SkillExecutionRecordInput<'a> {
    pub skill: &'a Skill,
    pub source: &'a str,
    pub arguments_json: &'a str,
    pub output: &'a str,
    pub duration_ms: u64,
    pub exit_code: i32,
    pub is_redacted: bool,
}

pub struct McpEventRecordInput<'a> {
    pub event_type: ObservabilityEventType,
    pub status: ObservabilityEventStatus,
    pub source: &'a str,
    pub entity_name: Option<&'a str>,
    pub summary: &'a str,
    pub metadata: Option<&'a str>,
    pub duration_ms: Option<u64>,
}

pub fn build_excerpt(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.chars().count() <= MAX_EXCERPT_CHARS {
        return Some(trimmed.to_string());
    }
    Some(trimmed.chars().take(MAX_EXCERPT_CHARS).collect::<String>() + "…")
}

pub async fn record_command_execution(db: &Database, input: &ExecutionLogInput<'_>) -> Result<()> {
    let metadata = serde_json::json!({
        "arguments": input.arguments_json,
        "triggeredBy": input.triggered_by,
        "adapterContext": input.adapter_context,
    })
    .to_string();
    let stdout_excerpt = build_excerpt(input.stdout);
    let stderr_excerpt = build_excerpt(input.stderr);
    let summary = if input.exit_code == 0 {
        "Command execution succeeded"
    } else {
        "Command execution failed"
    };

    db.add_observability_event(&ObservabilityEventInput {
        event_type: ObservabilityEventType::CommandRun,
        status: if input.exit_code == 0 {
            ObservabilityEventStatus::Success
        } else {
            ObservabilityEventStatus::Error
        },
        source: input.triggered_by,
        entity_kind: Some("command"),
        entity_id: Some(input.command_id),
        entity_name: Some(input.command_name),
        summary,
        metadata: Some(&metadata),
        stdout_excerpt: stdout_excerpt.as_deref(),
        stderr_excerpt: stderr_excerpt.as_deref(),
        duration_ms: Some(input.duration_ms),
        exit_code: Some(input.exit_code),
        failure_class: input.failure_class,
        attempt_number: Some(input.attempt_number),
        is_redacted: input.is_redacted,
    })
    .await
}

pub async fn record_skill_execution(
    db: &Database,
    input: &SkillExecutionRecordInput<'_>,
) -> Result<()> {
    let metadata = serde_json::json!({
        "arguments": input.arguments_json,
        "triggeredBy": input.source,
        "directoryPath": input.skill.directory_path,
        "entryPoint": input.skill.entry_point,
    })
    .to_string();
    let stdout_excerpt = build_excerpt(input.output);
    let summary = if input.exit_code == 0 {
        "Skill execution succeeded"
    } else {
        "Skill execution failed"
    };

    db.add_observability_event(&ObservabilityEventInput {
        event_type: ObservabilityEventType::SkillRun,
        status: if input.exit_code == 0 {
            ObservabilityEventStatus::Success
        } else {
            ObservabilityEventStatus::Error
        },
        source: input.source,
        entity_kind: Some("skill"),
        entity_id: Some(&input.skill.id),
        entity_name: Some(&input.skill.name),
        summary,
        metadata: Some(&metadata),
        stdout_excerpt: stdout_excerpt.as_deref(),
        stderr_excerpt: None,
        duration_ms: Some(input.duration_ms),
        exit_code: Some(input.exit_code),
        failure_class: None,
        attempt_number: Some(1),
        is_redacted: input.is_redacted,
    })
    .await
}

pub async fn record_mcp_event(db: &Database, input: &McpEventRecordInput<'_>) -> Result<()> {
    db.add_observability_event(&ObservabilityEventInput {
        event_type: input.event_type.clone(),
        status: input.status.clone(),
        source: input.source,
        entity_kind: Some("mcp"),
        entity_id: None,
        entity_name: input.entity_name,
        summary: input.summary,
        metadata: input.metadata,
        stdout_excerpt: None,
        stderr_excerpt: None,
        duration_ms: input.duration_ms,
        exit_code: None,
        failure_class: None,
        attempt_number: None,
        is_redacted: true,
    })
    .await
}

pub async fn export_events(
    db: &Database,
    path: &Path,
    selected_ids: Option<&[String]>,
    filter: &ObservabilityEventFilter,
) -> Result<usize> {
    let events = match selected_ids.filter(|ids| !ids.is_empty()) {
        Some(ids) => db.get_observability_events_by_ids(ids).await?,
        None => db.list_observability_events(filter).await?,
    };
    let export = ObservabilityExport {
        version: "1".to_string(),
        exported_at: chrono::Utc::now(),
        event_count: events.len(),
        events,
    };
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serde_json::to_string_pretty(&export)?)?;
    Ok(export.event_count)
}
