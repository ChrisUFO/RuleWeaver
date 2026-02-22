use chrono::{DateTime, Utc};
use serde::Serialize;
use std::path::Path;

use crate::error::Result;
use crate::models::Rule;

#[derive(Debug, Clone, Serialize)]
pub struct RuleFrontmatter {
    pub id: String,
    pub name: String,
    pub scope: String,
    #[serde(skip_serializing_if = "Option::is_none", rename = "targetPaths")]
    pub target_paths: Option<Vec<String>>,
    #[serde(rename = "enabledAdapters")]
    pub enabled_adapters: Vec<String>,
    pub enabled: bool,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

impl From<&Rule> for RuleFrontmatter {
    fn from(rule: &Rule) -> Self {
        Self {
            id: rule.id.clone(),
            name: rule.name.clone(),
            scope: rule.scope.as_str().to_string(),
            target_paths: rule.target_paths.clone(),
            enabled_adapters: rule
                .enabled_adapters
                .iter()
                .map(|a| a.as_str().to_string())
                .collect(),
            enabled: rule.enabled,
            created_at: format_datetime(rule.created_at),
            updated_at: format_datetime(rule.updated_at),
        }
    }
}

fn format_datetime(dt: DateTime<Utc>) -> String {
    dt.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

pub fn serialize_rule_to_file_content(rule: &Rule) -> Result<String> {
    let frontmatter = RuleFrontmatter::from(rule);
    let yaml =
        serde_yaml::to_string(&frontmatter).map_err(|e| crate::error::AppError::InvalidInput {
            message: format!("Failed to serialize rule to YAML: {}", e),
        })?;

    let content = rule.content.trim();

    Ok(format!(
        "---\n{}---\n{}{}\n",
        yaml,
        content,
        if content.is_empty() { "" } else { "\n" }
    ))
}

pub fn generate_filename(rule: &Rule) -> String {
    let safe_name = sanitize_filename(&rule.name);
    let prefix = match rule.scope {
        crate::models::Scope::Global => "",
        crate::models::Scope::Local => "local-",
    };
    format!("{}{}.md", prefix, safe_name)
}

fn sanitize_filename(name: &str) -> String {
    let mut result = String::new();
    for c in name.chars() {
        match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => result.push(c.to_ascii_lowercase()),
            ' ' | '.' | ',' | ':' | ';' | '!' | '?' | '(' | ')' | '[' | ']' | '{' | '}' => {
                result.push('-')
            }
            _ => {
                let id_char = format!("-{:x}-", c as u32);
                result.push_str(&id_char);
            }
        }
    }

    while result.contains("--") {
        result = result.replace("--", "-");
    }

    let trimmed = result.trim_matches('-').to_string();
    if trimmed.is_empty() {
        "unnamed-rule".to_string()
    } else {
        trimmed
    }
}

pub fn generate_rule_file_path(base_dir: &Path, rule: &Rule) -> std::path::PathBuf {
    base_dir.join(generate_filename(rule))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::AdapterType;
    use crate::models::Scope;

    fn create_test_rule(name: &str, scope: Scope) -> Rule {
        Rule {
            id: "test-id-123".to_string(),
            name: name.to_string(),
            content: "Test content".to_string(),
            scope,
            target_paths: None,
            enabled_adapters: vec![AdapterType::Gemini],
            enabled: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_serialize_rule_basic() {
        let rule = create_test_rule("Test Rule", Scope::Global);
        let result = serialize_rule_to_file_content(&rule);
        assert!(result.is_ok());

        let content = result.unwrap();
        assert!(content.starts_with("---\n"));
        assert!(content.contains("id: test-id-123"));
        assert!(content.contains("name: Test Rule"));
        assert!(content.contains("scope: global"));
        assert!(content.contains("---\nTest content"));
    }

    #[test]
    fn test_serialize_rule_with_target_paths() {
        let mut rule = create_test_rule("Local Rule", Scope::Local);
        rule.target_paths = Some(vec!["/path/to/project".to_string()]);

        let content = serialize_rule_to_file_content(&rule).unwrap();
        assert!(content.contains("targetPaths:"));
        assert!(content.contains("/path/to/project"));
    }

    #[test]
    fn test_serialize_rule_empty_content() {
        let mut rule = create_test_rule("Empty", Scope::Global);
        rule.content = String::new();

        let content = serialize_rule_to_file_content(&rule).unwrap();
        assert!(content.ends_with("---\n\n"));
    }

    #[test]
    fn test_sanitize_filename_simple() {
        assert_eq!(sanitize_filename("Simple Name"), "simple-name");
        assert_eq!(sanitize_filename("AnotherTest"), "anothertest");
        assert_eq!(sanitize_filename("Test 123"), "test-123");
    }

    #[test]
    fn test_sanitize_filename_special_chars() {
        assert_eq!(sanitize_filename("Test (Copy)"), "test-copy");
        assert_eq!(sanitize_filename("Hello! World?"), "hello-world");
        assert_eq!(sanitize_filename("A & B"), "a-26-b");
    }

    #[test]
    fn test_sanitize_filename_unicode() {
        let result = sanitize_filename("日本語ルール");
        assert!(!result.is_empty());
        assert!(!result.contains("日本語"));
    }

    #[test]
    fn test_sanitize_filename_multiple_spaces() {
        assert_eq!(
            sanitize_filename("Test   Multiple   Spaces"),
            "test-multiple-spaces"
        );
    }

    #[test]
    fn test_sanitize_filename_leading_trailing_dashes() {
        assert_eq!(sanitize_filename("---Test---"), "test");
        assert_eq!(sanitize_filename("- - Test - -"), "test");
    }

    #[test]
    fn test_sanitize_filename_empty() {
        assert_eq!(sanitize_filename(""), "unnamed-rule");
        assert_eq!(sanitize_filename("   "), "unnamed-rule");
        assert_eq!(sanitize_filename("---"), "unnamed-rule");
    }

    #[test]
    fn test_generate_filename_global() {
        let rule = create_test_rule("My Global Rule", Scope::Global);
        let filename = generate_filename(&rule);
        assert_eq!(filename, "my-global-rule.md");
    }

    #[test]
    fn test_generate_filename_local() {
        let rule = create_test_rule("My Local Rule", Scope::Local);
        let filename = generate_filename(&rule);
        assert_eq!(filename, "local-my-local-rule.md");
    }

    #[test]
    fn test_roundtrip_parse_serialize() {
        let original_content = "---\nid: roundtrip-123\nname: Roundtrip Test\nscope: global\nenabledAdapters:\n- gemini\n- opencode\nenabled: true\ntargetPaths:\n- /src\ncreatedAt: 2024-01-15T10:30:00Z\nupdatedAt: 2024-01-16T14:20:00Z\n---\n\nThis is the body content.\n";

        let parsed = crate::file_storage::parser::parse_rule_file(
            std::path::Path::new("test.md"),
            original_content,
        )
        .unwrap();
        let rule = parsed.to_rule().unwrap();

        let serialized = serialize_rule_to_file_content(&rule).unwrap();

        let reparsed = crate::file_storage::parser::parse_rule_file(
            std::path::Path::new("test.md"),
            &serialized,
        )
        .unwrap();
        let rerule = reparsed.to_rule().unwrap();

        assert_eq!(rule.id, rerule.id);
        assert_eq!(rule.name, rerule.name);
        assert_eq!(rule.scope, rerule.scope);
        assert_eq!(rule.content.trim(), rerule.content.trim());
    }

    #[test]
    fn test_format_datetime() {
        let dt = DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let formatted = format_datetime(dt);
        assert_eq!(formatted, "2024-01-15T10:30:00Z");
    }
}
