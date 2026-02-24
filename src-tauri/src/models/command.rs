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
    pub generate_slash_commands: bool,
    #[serde(default)]
    pub slash_command_adapters: Vec<String>,
    #[serde(default)]
    pub target_paths: Vec<String>,
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
}

impl Command {
    #[allow(dead_code)]
    pub fn new(name: String, description: String, script: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            script,
            arguments: Vec::new(),
            expose_via_mcp: true,
            generate_slash_commands: false,
            slash_command_adapters: Vec::new(),
            target_paths: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCommandInput {
    pub name: String,
    pub description: String,
    pub script: String,
    #[serde(default)]
    pub arguments: Vec<CommandArgument>,
    #[serde(default = "default_true")]
    pub expose_via_mcp: bool,
    #[serde(default)]
    pub generate_slash_commands: bool,
    #[serde(default)]
    pub slash_command_adapters: Vec<String>,
    #[serde(default)]
    pub target_paths: Vec<String>,
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
    pub generate_slash_commands: Option<bool>,
    pub slash_command_adapters: Option<Vec<String>>,
    pub target_paths: Option<Vec<String>>,
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
        );
        assert_eq!(command.name, "Fmt");
        assert!(command.expose_via_mcp);
        assert!(command.arguments.is_empty());
        assert!(!command.id.is_empty());
    }

    #[test]
    fn test_command_roundtrip() {
        let input = CreateCommandInput {
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
            generate_slash_commands: false,
            slash_command_adapters: vec![],
            target_paths: vec![],
        };

        let json = serde_json::to_string(&input).expect("serialize create input");
        let parsed: CreateCommandInput =
            serde_json::from_str(&json).expect("deserialize create input");
        assert_eq!(parsed.name, input.name);
        assert_eq!(parsed.arguments.len(), 1);
    }
}
