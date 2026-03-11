use std::path::Path;

use crate::database::{Database, ExecutionLogInput, ObservabilityEventInput};
use crate::error::{AppError, Result};
use crate::models::{
    ObservabilityEventFilter, ObservabilityEventStatus, ObservabilityEventType,
    ObservabilityExport, Skill,
};
use crate::redaction::redact;

const MAX_EXCERPT_CHARS: usize = 2000;

pub struct SkillExecutionRecordInput<'a> {
    pub skill: &'a Skill,
    pub source: &'a str,
    pub arguments_json: &'a str,
    pub output: &'a str,
    pub duration_ms: u64,
    pub exit_code: i32,
    pub workspace_path: Option<&'a str>,
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

fn redact_text(value: &str) -> (String, bool) {
    redact(value)
}

fn redact_optional(value: Option<&str>) -> (Option<String>, bool) {
    match value {
        Some(content) => {
            let (redacted_value, was_redacted) = redact_text(content);
            (Some(redacted_value), was_redacted)
        }
        None => (None, false),
    }
}

fn redact_excerpt(value: &str) -> (Option<String>, bool) {
    match build_excerpt(value) {
        Some(excerpt) => {
            let (redacted_excerpt, was_redacted) = redact_text(&excerpt);
            (Some(redacted_excerpt), was_redacted)
        }
        None => (None, false),
    }
}

pub async fn record_command_execution(
    db: &Database,
    input: &ExecutionLogInput<'_>,
) -> Result<String> {
    let metadata = serde_json::json!({
        "arguments": input.arguments_json,
        "triggeredBy": input.triggered_by,
        "adapterContext": input.adapter_context,
        "workspacePath": input.workspace_path,
    })
    .to_string();
    let (metadata, metadata_redacted) = redact_optional(Some(&metadata));
    let (stdout_excerpt, stdout_redacted) = redact_excerpt(input.stdout);
    let (stderr_excerpt, stderr_redacted) = redact_excerpt(input.stderr);
    let summary = if input.exit_code == 0 {
        "Command execution succeeded"
    } else {
        "Command execution failed"
    };
    let (summary, summary_redacted) = redact_text(summary);

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
        workspace_path: input.workspace_path,
        summary: &summary,
        metadata: metadata.as_deref(),
        stdout_excerpt: stdout_excerpt.as_deref(),
        stderr_excerpt: stderr_excerpt.as_deref(),
        duration_ms: Some(input.duration_ms),
        exit_code: Some(input.exit_code),
        failure_class: input.failure_class,
        attempt_number: Some(input.attempt_number),
        is_redacted: input.is_redacted
            || metadata_redacted
            || stdout_redacted
            || stderr_redacted
            || summary_redacted,
    })
    .await
}

pub async fn record_skill_execution(
    db: &Database,
    input: &SkillExecutionRecordInput<'_>,
) -> Result<String> {
    let metadata = serde_json::json!({
        "arguments": input.arguments_json,
        "triggeredBy": input.source,
        "directoryPath": input.skill.directory_path,
        "entryPoint": input.skill.entry_point,
        "workspacePath": input.workspace_path,
    })
    .to_string();
    let (metadata, metadata_redacted) = redact_optional(Some(&metadata));
    let (stdout_excerpt, stdout_redacted) = redact_excerpt(input.output);
    let summary = if input.exit_code == 0 {
        "Skill execution succeeded"
    } else {
        "Skill execution failed"
    };
    let (summary, summary_redacted) = redact_text(summary);

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
        workspace_path: input.workspace_path,
        summary: &summary,
        metadata: metadata.as_deref(),
        stdout_excerpt: stdout_excerpt.as_deref(),
        stderr_excerpt: None,
        duration_ms: Some(input.duration_ms),
        exit_code: Some(input.exit_code),
        failure_class: None,
        attempt_number: Some(1),
        is_redacted: input.is_redacted || metadata_redacted || stdout_redacted || summary_redacted,
    })
    .await
}

pub async fn record_mcp_event(db: &Database, input: &McpEventRecordInput<'_>) -> Result<String> {
    let (summary, summary_redacted) = redact_text(input.summary);
    let (metadata, metadata_redacted) = redact_optional(input.metadata);

    db.add_observability_event(&ObservabilityEventInput {
        event_type: input.event_type.clone(),
        status: input.status.clone(),
        source: input.source,
        entity_kind: Some("mcp"),
        entity_id: None,
        entity_name: input.entity_name,
        workspace_path: None,
        summary: &summary,
        metadata: metadata.as_deref(),
        stdout_excerpt: None,
        stderr_excerpt: None,
        duration_ms: input.duration_ms,
        exit_code: None,
        failure_class: None,
        attempt_number: None,
        is_redacted: true || summary_redacted || metadata_redacted,
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
    if !path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
    {
        return Err(AppError::InvalidInput {
            message: format!(
                "Observability exports must use a .json file path: {}",
                path.display()
            ),
        });
    }
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent).map_err(|error| AppError::InvalidInput {
            message: format!(
                "Failed to prepare the export directory '{}': {}",
                parent.display(),
                error
            ),
        })?;
    }

    let export_body = serde_json::to_string_pretty(&export).map_err(AppError::Serialization)?;

    // Atomic write using a temporary file
    let temp_name = format!(".{}.tmp", uuid::Uuid::new_v4());
    let temp_path = path.with_file_name(&temp_name);

    std::fs::write(&temp_path, export_body).map_err(AppError::Io)?;

    if let Err(e) = std::fs::rename(&temp_path, path) {
        let _ = std::fs::remove_file(&temp_path);
        return Err(AppError::Io(e));
    }

    Ok(export.event_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::Database;

    #[tokio::test]
    async fn record_command_execution_redacts_metadata_and_tracks_workspace() {
        let db = Database::new_in_memory().await.unwrap();
        let log_input = ExecutionLogInput {
            command_id: "cmd-1",
            command_name: "deploy",
            arguments_json: r#"{"token":"ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"}"#,
            stdout: "api_key=sk_test_FAKEKEYFORTESTINGONLYabcdefghijklmnop",
            stderr: "",
            exit_code: 1,
            duration_ms: 250,
            triggered_by: "mcp",
            failure_class: Some("non_zero_exit"),
            adapter_context: Some("mcp"),
            workspace_path: Some("c:/repos/docs"),
            is_redacted: false,
            attempt_number: 1,
        };

        record_command_execution(&db, &log_input).await.unwrap();

        let events = db
            .list_observability_events(&ObservabilityEventFilter {
                event_type: Some(ObservabilityEventType::CommandRun),
                limit: Some(10),
                ..Default::default()
            })
            .await
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].workspace_path.as_deref(), Some("c:/repos/docs"));
        assert!(events[0].is_redacted);
        assert!(events[0]
            .metadata
            .as_deref()
            .is_some_and(|value| value.contains("[REDACTED]")));
        assert!(events[0]
            .stdout_excerpt
            .as_deref()
            .is_some_and(|value| value.contains("[REDACTED]")));
    }

    #[tokio::test]
    async fn export_events_requires_json_extension() {
        let db = Database::new_in_memory().await.unwrap();
        let path = std::env::temp_dir().join(format!(
            "ruleweaver-observability-{}.txt",
            uuid::Uuid::new_v4()
        ));

        let error = export_events(&db, &path, None, &ObservabilityEventFilter::default())
            .await
            .unwrap_err();

        assert!(error.to_string().contains(".json"));
    }
}
