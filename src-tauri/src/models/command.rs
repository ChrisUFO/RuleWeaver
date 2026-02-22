use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    pub id: String,
    pub name: String,
    pub description: String,
    pub script: String,
    pub arguments: Vec<CommandArgument>,
    pub expose_via_mcp: bool,
    #[serde(with = "ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandArgument {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub default_value: Option<String>,
}

mod ts_seconds {
    use chrono::{DateTime, TimeZone, Utc};
    use serde::{self, Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        date.timestamp().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let ts = i64::deserialize(deserializer)?;
        Utc.timestamp_opt(ts, 0)
            .single()
            .ok_or_else(|| serde::de::Error::custom(format!("Invalid timestamp: {}", ts)))
    }
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
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCommandInput {
    pub name: String,
    pub description: String,
    pub script: String,
    #[serde(default)]
    pub arguments: Vec<CommandArgument>,
    #[serde(default = "default_true")]
    pub expose_via_mcp: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateCommandInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub script: Option<String>,
    pub arguments: Option<Vec<CommandArgument>>,
    pub expose_via_mcp: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCommandResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionLog {
    pub id: String,
    pub command_id: String,
    pub command_name: String,
    pub arguments: String,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub duration_ms: u64,
    #[serde(with = "ts_seconds")]
    pub executed_at: DateTime<Utc>,
    pub triggered_by: String,
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
                required: false,
                default_value: None,
            }],
            expose_via_mcp: true,
        };

        let json = serde_json::to_string(&input).expect("serialize create input");
        let parsed: CreateCommandInput =
            serde_json::from_str(&json).expect("deserialize create input");
        assert_eq!(parsed.name, input.name);
        assert_eq!(parsed.arguments.len(), 1);
    }
}
