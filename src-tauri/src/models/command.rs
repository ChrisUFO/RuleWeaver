use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Command {
    pub id: String,
    pub name: String,
    pub description: String,
    pub script: String,
    pub arguments: Vec<CommandArgument>,
    pub expose_via_mcp: bool,
    #[serde(default)]
    pub is_placeholder: bool,
    #[serde(default)]
    pub generate_slash_commands: bool,
    #[serde(default)]
    pub slash_command_adapters: Vec<String>,
    #[serde(default)]
    pub target_paths: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_retries: Option<u8>,
    #[serde(with = "crate::models::timestamp")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "crate::models::timestamp")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandArgument {
    pub name: String,
    pub description: String,
    #[serde(default = "default_arg_type")]
    pub arg_type: ArgumentType,
    pub required: bool,
    pub default_value: Option<String>,
    pub options: Option<Vec<String>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ArgumentType {
    String,
    Number,
    Boolean,
    Enum,
}

fn default_arg_type() -> ArgumentType {
    ArgumentType::String
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FailureClass {
    Success,
    ValidationError,
    Timeout,
    PermissionDenied,
    MissingBinary,
    NonZeroExit,
    UnknownError,
}

impl FailureClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            FailureClass::Success => "success",
            FailureClass::ValidationError => "validation_error",
            FailureClass::Timeout => "timeout",
            FailureClass::PermissionDenied => "permission_denied",
            FailureClass::MissingBinary => "missing_binary",
            FailureClass::NonZeroExit => "non_zero_exit",
            FailureClass::UnknownError => "unknown_error",
        }
    }

    #[allow(dead_code)]
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "success" => Some(FailureClass::Success),
            "validation_error" => Some(FailureClass::ValidationError),
            "timeout" => Some(FailureClass::Timeout),
            "permission_denied" => Some(FailureClass::PermissionDenied),
            "missing_binary" => Some(FailureClass::MissingBinary),
            "non_zero_exit" => Some(FailureClass::NonZeroExit),
            "unknown_error" => Some(FailureClass::UnknownError),
            _ => None,
        }
    }

    pub fn is_retryable(&self) -> bool {
        !matches!(
            self,
            FailureClass::ValidationError
                | FailureClass::MissingBinary
                | FailureClass::PermissionDenied
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionLog {
    pub id: String,
    pub command_id: String,
    pub command_name: String,
    pub arguments: String,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub duration_ms: u64,
    #[serde(with = "crate::models::timestamp")]
    pub executed_at: DateTime<Utc>,
    pub triggered_by: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adapter_context: Option<String>,
    #[serde(default)]
    pub is_redacted: bool,
    #[serde(default)]
    pub attempt_number: u8,
}

impl Command {
    #[allow(dead_code)]
    pub fn new(name: String, description: String, script: String, is_placeholder: bool) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            script,
            arguments: Vec::new(),
            expose_via_mcp: true,
            is_placeholder,
            generate_slash_commands: false,
            slash_command_adapters: Vec::new(),
            target_paths: Vec::new(),
            timeout_ms: None,
            max_retries: None,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CreateCommandInput {
    pub id: Option<String>,
    pub name: String,
    pub description: String,
    pub script: String,
    #[serde(default)]
    pub arguments: Vec<CommandArgument>,
    #[serde(default = "default_true")]
    pub expose_via_mcp: bool,
    #[serde(default)]
    pub is_placeholder: bool,
    #[serde(default)]
    pub generate_slash_commands: bool,
    #[serde(default)]
    pub slash_command_adapters: Vec<String>,
    #[serde(default)]
    pub target_paths: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_retries: Option<u8>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCommandInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub script: Option<String>,
    pub arguments: Option<Vec<CommandArgument>>,
    pub expose_via_mcp: Option<bool>,
    pub is_placeholder: Option<bool>,
    pub generate_slash_commands: Option<bool>,
    pub slash_command_adapters: Option<Vec<String>>,
    pub target_paths: Option<Vec<String>>,
    pub timeout_ms: Option<u64>,
    pub max_retries: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestCommandResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub duration_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_command_defaults() {
        let command = Command::new(
            "Fmt".to_string(),
            "Format".to_string(),
            "npm run fmt".to_string(),
            false,
        );
        assert_eq!(command.name, "Fmt");
        assert!(command.expose_via_mcp);
        assert!(command.arguments.is_empty());
        assert!(!command.id.is_empty());
    }

    #[test]
    fn test_command_roundtrip() {
        let input = CreateCommandInput {
            id: None,
            name: "Run Tests".to_string(),
            description: "Run unit tests".to_string(),
            script: "npm test".to_string(),
            arguments: vec![CommandArgument {
                name: "path".to_string(),
                description: "Test path".to_string(),
                arg_type: ArgumentType::String,
                required: false,
                default_value: None,
                options: None,
            }],
            expose_via_mcp: true,
            is_placeholder: false,
            generate_slash_commands: false,
            slash_command_adapters: vec![],
            target_paths: vec![],
            timeout_ms: None,
            max_retries: None,
        };

        let json = serde_json::to_string(&input).expect("serialize create input");
        let parsed: CreateCommandInput =
            serde_json::from_str(&json).expect("deserialize create input");
        assert_eq!(parsed.name, input.name);
        assert_eq!(parsed.arguments.len(), 1);
    }
}
