use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Scope {
    #[default]
    Global,
    Local,
}

impl Scope {
    pub fn as_str(&self) -> &'static str {
        match self {
            Scope::Global => "global",
            Scope::Local => "local",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "global" => Some(Scope::Global),
            "local" => Some(Scope::Local),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AdapterType {
    Antigravity,
    Gemini,
    OpenCode,
    Cline,
    ClaudeCode,
    Codex,
    Kilo,
    Cursor,
    Windsurf,
    RooCode,
}

impl AdapterType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AdapterType::Antigravity => "antigravity",
            AdapterType::Gemini => "gemini",
            AdapterType::OpenCode => "opencode",
            AdapterType::Cline => "cline",
            AdapterType::ClaudeCode => "claude-code",
            AdapterType::Codex => "codex",
            AdapterType::Kilo => "kilo",
            AdapterType::Cursor => "cursor",
            AdapterType::Windsurf => "windsurf",
            AdapterType::RooCode => "roocode",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "antigravity" => Some(AdapterType::Antigravity),
            "gemini" => Some(AdapterType::Gemini),
            "opencode" => Some(AdapterType::OpenCode),
            "cline" => Some(AdapterType::Cline),
            "claude-code" => Some(AdapterType::ClaudeCode),
            "codex" => Some(AdapterType::Codex),
            "kilo" => Some(AdapterType::Kilo),
            "cursor" => Some(AdapterType::Cursor),
            "windsurf" => Some(AdapterType::Windsurf),
            "roocode" => Some(AdapterType::RooCode),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn all() -> Vec<Self> {
        vec![
            AdapterType::Antigravity,
            AdapterType::Gemini,
            AdapterType::OpenCode,
            AdapterType::Cline,
            AdapterType::ClaudeCode,
            AdapterType::Codex,
            AdapterType::Kilo,
            AdapterType::Cursor,
            AdapterType::Windsurf,
            AdapterType::RooCode,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub content: String,
    pub scope: Scope,
    pub target_paths: Option<Vec<String>>,
    pub enabled_adapters: Vec<AdapterType>,
    pub enabled: bool,
    #[serde(with = "crate::models::timestamp")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "crate::models::timestamp")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncHistoryEntry {
    pub id: String,
    #[serde(with = "crate::models::timestamp")]
    pub timestamp: DateTime<Utc>,
    pub files_written: u32,
    pub status: String,
    pub triggered_by: String,
}

impl Rule {
    #[allow(dead_code)]
    pub fn new(name: String, description: String, content: String, scope: Scope) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            content,
            scope,
            target_paths: None,
            enabled_adapters: vec![AdapterType::Gemini, AdapterType::OpenCode],
            enabled: true,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRuleInput {
    pub id: Option<String>,
    pub name: String,
    pub description: String,
    pub content: String,
    pub scope: Scope,
    #[serde(default)]
    pub target_paths: Option<Vec<String>>,
    pub enabled_adapters: Vec<AdapterType>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRuleInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub content: Option<String>,
    pub scope: Option<Scope>,
    pub target_paths: Option<Vec<String>>,
    pub enabled_adapters: Option<Vec<AdapterType>>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncResult {
    pub success: bool,
    pub files_written: Vec<String>,
    pub errors: Vec<SyncError>,
    pub conflicts: Vec<Conflict>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncError {
    pub file_path: String,
    pub adapter_name: String,
    pub message: String,
}

/// Summary of line-level differences between two versions of a file.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DiffSummary {
    /// Lines present in the current file but not in the stored version
    pub added: usize,
    /// Lines present in the stored version but not in the current file
    pub removed: usize,
    /// Lines that exist in both versions but with different content
    pub changed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Conflict {
    pub id: String,
    pub file_path: String,
    pub adapter_name: String,
    pub adapter_id: Option<AdapterType>,
    pub local_hash: String,
    pub current_hash: String,
    /// Scope of the artifact (global or local)
    pub scope: Option<String>,
    /// Line-level diff summary for UI diagnostics
    pub diff_summary: Option<DiffSummary>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_from_str() {
        assert!(matches!(Scope::from_str("global"), Some(Scope::Global)));
        assert!(matches!(Scope::from_str("local"), Some(Scope::Local)));
        assert!(matches!(Scope::from_str("GLOBAL"), Some(Scope::Global)));
        assert!(Scope::from_str("invalid").is_none());
    }

    #[test]
    fn test_scope_as_str() {
        assert_eq!(Scope::Global.as_str(), "global");
        assert_eq!(Scope::Local.as_str(), "local");
    }

    #[test]
    fn test_adapter_type_from_str() {
        assert!(matches!(
            AdapterType::from_str("antigravity"),
            Some(AdapterType::Antigravity)
        ));
        assert!(matches!(
            AdapterType::from_str("gemini"),
            Some(AdapterType::Gemini)
        ));
        assert!(matches!(
            AdapterType::from_str("opencode"),
            Some(AdapterType::OpenCode)
        ));
        assert!(matches!(
            AdapterType::from_str("cline"),
            Some(AdapterType::Cline)
        ));
        assert!(matches!(
            AdapterType::from_str("claude-code"),
            Some(AdapterType::ClaudeCode)
        ));
        assert!(matches!(
            AdapterType::from_str("codex"),
            Some(AdapterType::Codex)
        ));
        assert!(AdapterType::from_str("invalid").is_none());
    }

    #[test]
    fn test_adapter_type_all() {
        let all = AdapterType::all();
        assert_eq!(all.len(), 10);
        assert!(all.contains(&AdapterType::Antigravity));
        assert!(all.contains(&AdapterType::Gemini));
        assert!(all.contains(&AdapterType::OpenCode));
        assert!(all.contains(&AdapterType::Cline));
        assert!(all.contains(&AdapterType::ClaudeCode));
        assert!(all.contains(&AdapterType::Codex));
        assert!(all.contains(&AdapterType::Kilo));
        assert!(all.contains(&AdapterType::Cursor));
        assert!(all.contains(&AdapterType::Windsurf));
        assert!(all.contains(&AdapterType::RooCode));
    }

    #[test]
    fn test_rule_new() {
        let rule = Rule::new(
            "Test Rule".to_string(),
            "Test Description".to_string(),
            "Test content".to_string(),
            Scope::Global,
        );

        assert_eq!(rule.name, "Test Rule");
        assert_eq!(rule.description, "Test Description");
        assert_eq!(rule.content, "Test content");
        assert!(matches!(rule.scope, Scope::Global));
        assert!(rule.enabled);
        assert!(!rule.id.is_empty());
    }

    #[test]
    fn test_create_rule_input_serialization() {
        let input = CreateRuleInput {
            id: None,
            name: "Test".to_string(),
            description: "Desc".to_string(),
            content: "Content".to_string(),
            scope: Scope::Global,
            target_paths: None,
            enabled_adapters: vec![AdapterType::Gemini, AdapterType::OpenCode],
            enabled: true,
        };

        let json = serde_json::to_string(&input).unwrap();
        let parsed: CreateRuleInput = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.name, input.name);
        assert_eq!(parsed.description, input.description);
        assert_eq!(parsed.content, input.content);
        assert!(matches!(parsed.scope, Scope::Global));
        assert_eq!(parsed.enabled_adapters.len(), 2);
        assert!(parsed.enabled);
    }

    #[test]
    fn test_create_rule_input_camel_case_from_frontend() {
        // This is the key test - simulating JSON from frontend with camelCase
        // Adapter types use lowercase (OpenCode -> opencode)
        let json = r#"{
            "name": "Test Rule",
            "description": "Test description",
            "content": "Test content",
            "scope": "global",
            "targetPaths": ["/path/to/repo"],
            "enabledAdapters": ["gemini", "opencode"],
            "enabled": true
        }"#;

        let parsed: CreateRuleInput = serde_json::from_str(json).unwrap();

        assert_eq!(parsed.name, "Test Rule");
        assert_eq!(parsed.description, "Test description");
        assert!(matches!(parsed.scope, Scope::Global));
        assert_eq!(parsed.target_paths, Some(vec!["/path/to/repo".to_string()]));
        assert_eq!(parsed.enabled_adapters.len(), 2);
        assert!(parsed.enabled);
    }

    #[test]
    fn test_update_rule_input_camel_case_from_frontend() {
        // Testing UpdateRuleInput with camelCase from frontend
        let json = r#"{
            "name": "Updated Name",
            "content": "Updated content",
            "scope": "local",
            "targetPaths": ["/new/path"],
            "enabledAdapters": ["cline"],
            "enabled": false
        }"#;

        let parsed: UpdateRuleInput = serde_json::from_str(json).unwrap();

        assert_eq!(parsed.name, Some("Updated Name".to_string()));
        assert_eq!(parsed.content, Some("Updated content".to_string()));
        assert!(matches!(parsed.scope, Some(Scope::Local)));
        assert_eq!(parsed.target_paths, Some(vec!["/new/path".to_string()]));
        assert_eq!(parsed.enabled_adapters, Some(vec![AdapterType::Cline]));
        assert_eq!(parsed.enabled, Some(false));
    }

    #[test]
    fn test_rule_output_camel_case_to_frontend() {
        // Test that Rule serializes to camelCase for frontend
        let rule = Rule {
            id: "test-id".to_string(),
            name: "Test Rule".to_string(),
            description: "Test description".to_string(),
            content: "Content".to_string(),
            scope: Scope::Global,
            target_paths: Some(vec!["/path".to_string()]),
            enabled_adapters: vec![AdapterType::Gemini],
            enabled: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&rule).unwrap();

        // Should contain camelCase keys
        assert!(json.contains("\"name\":\"Test Rule\""));
        assert!(json.contains("\"scope\":\"global\""));
        assert!(json.contains("\"enabledAdapters\""));
        assert!(json.contains("\"targetPaths\""));
        assert!(json.contains("\"createdAt\""));
        assert!(json.contains("\"updatedAt\""));

        // Should NOT contain snake_case
        assert!(!json.contains("\"enabled_adapters\""));
        assert!(!json.contains("\"target_paths\""));
    }
}
