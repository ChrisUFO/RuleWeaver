use chrono::{DateTime, Utc};
use regex::Regex;
use serde::Deserialize;
use std::path::Path;
use std::sync::OnceLock;

use crate::error::{AppError, Result};
use crate::models::{AdapterType, Rule, Scope};

const FRONTMATTER_DELIMITER: &str = "---";

#[derive(Debug, Clone, Deserialize)]
pub struct RuleFrontmatter {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub scope: String,
    #[serde(default, rename = "targetPaths")]
    pub target_paths: Option<Vec<String>>,
    #[serde(default, rename = "enabledAdapters")]
    pub enabled_adapters: Vec<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

fn default_true() -> bool {
    true
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ParsedRuleFile {
    pub frontmatter: RuleFrontmatter,
    pub content: String,
    pub file_path: String,
}

impl ParsedRuleFile {
    pub fn to_rule(&self) -> Result<Rule> {
        let scope = Scope::from_str(&self.frontmatter.scope).unwrap_or(Scope::Global);

        let enabled_adapters: Vec<AdapterType> = self
            .frontmatter
            .enabled_adapters
            .iter()
            .filter_map(|s| AdapterType::from_str(s))
            .collect();

        if enabled_adapters.is_empty() {
            return Err(AppError::InvalidInput {
                message: format!(
                    "Rule '{}' has no valid enabled adapters",
                    self.frontmatter.name
                ),
            });
        }

        let created_at = parse_iso_datetime(&self.frontmatter.created_at)?;
        let updated_at = parse_iso_datetime(&self.frontmatter.updated_at)?;

        Ok(Rule {
            id: self.frontmatter.id.clone(),
            name: self.frontmatter.name.clone(),
            content: self.content.clone(),
            scope,
            target_paths: self.frontmatter.target_paths.clone(),
            enabled_adapters,
            enabled: self.frontmatter.enabled,
            created_at,
            updated_at,
        })
    }
}

fn parse_iso_datetime(s: &str) -> Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S")
                .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc))
        })
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc))
        })
        .or_else(|_| {
            chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .map(|d| d.and_hms_opt(0, 0, 0).unwrap_or_default())
                .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc))
        })
        .map_err(|e| AppError::InvalidInput {
            message: format!("Invalid datetime format '{}': {}", s, e),
        })
}

pub fn parse_rule_file(file_path: &Path, raw_content: &str) -> Result<ParsedRuleFile> {
    let content = raw_content.trim_start();

    if !content.starts_with(FRONTMATTER_DELIMITER) {
        return Err(AppError::InvalidInput {
            message: format!(
                "File '{}' does not contain valid YAML frontmatter (missing opening ---)",
                file_path.display()
            ),
        });
    }

    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| {
        Regex::new(r"^---\s*\n([\s\S]*?)\n---\s*\n?([\s\S]*)$").expect("Invalid rule regex")
    });

    let caps = re.captures(content).ok_or_else(|| AppError::InvalidInput {
        message: format!(
            "File '{}' has invalid frontmatter format (missing closing --- or malformed structure)",
            file_path.display()
        ),
    })?;

    let yaml_str = caps.get(1).map(|m| m.as_str()).unwrap_or("");
    let body = caps.get(2).map(|m| m.as_str()).unwrap_or("").to_string();

    let frontmatter: RuleFrontmatter = serde_yaml::from_str(yaml_str).map_err(|e| {
        let error_msg = match e.location() {
            Some(loc) => format!(" at line {}, column {}", loc.line(), loc.column()),
            None => String::new(),
        };
        AppError::InvalidInput {
            message: format!(
                "Failed to parse YAML frontmatter in '{}'{}: {}",
                file_path.display(),
                error_msg,
                e
            ),
        }
    })?;

    validate_frontmatter(&frontmatter, file_path)?;

    Ok(ParsedRuleFile {
        frontmatter,
        content: body,
        file_path: file_path.to_string_lossy().to_string(),
    })
}

fn validate_frontmatter(fm: &RuleFrontmatter, file_path: &Path) -> Result<()> {
    if fm.id.trim().is_empty() {
        return Err(AppError::InvalidInput {
            message: format!(
                "Rule in '{}' has empty or missing 'id' field",
                file_path.display()
            ),
        });
    }

    if fm.name.trim().is_empty() {
        return Err(AppError::InvalidInput {
            message: format!(
                "Rule in '{}' has empty or missing 'name' field",
                file_path.display()
            ),
        });
    }

    if fm.created_at.trim().is_empty() {
        return Err(AppError::InvalidInput {
            message: format!(
                "Rule in '{}' has empty or missing 'createdAt' field",
                file_path.display()
            ),
        });
    }

    if fm.updated_at.trim().is_empty() {
        return Err(AppError::InvalidInput {
            message: format!(
                "Rule in '{}' has empty or missing 'updatedAt' field",
                file_path.display()
            ),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_path(name: &str) -> PathBuf {
        PathBuf::from(format!("/test/{}.md", name))
    }

    #[test]
    fn test_parse_rule_file_success() {
        let content = r#"---
id: test-123
name: "Test Rule"
scope: global
targetPaths: null
enabledAdapters: [gemini, opencode]
enabled: true
createdAt: 2024-01-15T10:30:00Z
updatedAt: 2024-01-15T10:30:00Z
---

# Test Rule Content

This is the rule body.
"#;

        let result = parse_rule_file(&create_test_path("test"), content);
        assert!(result.is_ok());

        let parsed = result.unwrap();
        assert_eq!(parsed.frontmatter.id, "test-123");
        assert_eq!(parsed.frontmatter.name, "Test Rule");
        assert!(parsed.content.contains("Test Rule Content"));
    }

    #[test]
    fn test_parse_rule_file_missing_frontmatter() {
        let content = "# Just markdown\nNo frontmatter here.";

        let result = parse_rule_file(&create_test_path("test"), content);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("missing opening ---"));
    }

    #[test]
    fn test_parse_rule_file_invalid_yaml() {
        let content = "---\nid: [invalid yaml\n---\nContent";

        let result = parse_rule_file(&create_test_path("test"), content);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_rule_file_missing_required_field() {
        let content = "---\nname: Test\n---\nContent";

        let result = parse_rule_file(&create_test_path("test"), content);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_rule_file_missing_closing_delimiter() {
        let content = "---\nid: test-123\nname: Test\n\nNo closing delimiter";

        let result = parse_rule_file(&create_test_path("test"), content);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("missing closing ---"));
    }

    #[test]
    fn test_to_rule_conversion() {
        let content = r#"---
id: test-456
name: "Conversion Test"
scope: local
targetPaths: ["/src"]
enabledAdapters: [gemini, cline]
enabled: false
createdAt: 2024-01-15T10:30:00Z
updatedAt: 2024-01-16T14:20:00Z
---

Test body.
"#;

        let parsed = parse_rule_file(&create_test_path("test"), content).unwrap();
        let rule = parsed.to_rule().unwrap();

        assert_eq!(rule.id, "test-456");
        assert_eq!(rule.name, "Conversion Test");
        assert!(matches!(rule.scope, Scope::Local));
        assert_eq!(rule.target_paths, Some(vec!["/src".to_string()]));
        assert_eq!(rule.enabled_adapters.len(), 2);
        assert!(!rule.enabled);
    }

    #[test]
    fn test_to_rule_no_valid_adapters() {
        let content = r#"---
id: test-789
name: "Invalid Adapters"
scope: global
enabledAdapters: [invalid, unknown]
createdAt: 2024-01-15T10:30:00Z
updatedAt: 2024-01-15T10:30:00Z
---
Content
"#;

        let parsed = parse_rule_file(&create_test_path("test"), content).unwrap();
        let result = parsed.to_rule();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("no valid enabled adapters"));
    }

    #[test]
    fn test_parse_datetime_formats() {
        let iso_result = parse_iso_datetime("2024-01-15T10:30:00Z");
        assert!(iso_result.is_ok());

        let no_tz_result = parse_iso_datetime("2024-01-15T10:30:00");
        assert!(no_tz_result.is_ok());

        let date_only_result = parse_iso_datetime("2024-01-15");
        assert!(date_only_result.is_ok());

        let space_format_result = parse_iso_datetime("2024-01-15 10:30:00");
        assert!(space_format_result.is_ok());
    }

    #[test]
    fn test_parse_rule_file_empty_body() {
        let content = "---\nid: test-empty\nname: Empty\nscope: global\nenabledAdapters: [gemini]\ncreatedAt: 2024-01-15T10:30:00Z\nupdatedAt: 2024-01-15T10:30:00Z\n---\n";

        let result = parse_rule_file(&create_test_path("test"), content);
        assert!(result.is_ok());

        let parsed = result.unwrap();
        assert_eq!(parsed.content, "");
    }

    #[test]
    fn test_parse_rule_file_whitespace_handling() {
        let content = "   \n  ---\nid: test-ws\nname: Whitespace Test\nscope: global\nenabledAdapters: [gemini]\ncreatedAt: 2024-01-15T10:30:00Z\nupdatedAt: 2024-01-15T10:30:00Z\n---\n  \n  Body content  \n";

        let result = parse_rule_file(&create_test_path("test"), content);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_rule_file_default_values() {
        let content = "---\nid: test-defaults\nname: Defaults Test\nenabledAdapters: [gemini]\ncreatedAt: 2024-01-15T10:30:00Z\nupdatedAt: 2024-01-15T10:30:00Z\n---\nBody";

        let parsed = parse_rule_file(&create_test_path("test"), content).unwrap();
        let rule = parsed.to_rule().unwrap();

        assert!(matches!(rule.scope, Scope::Global));
        assert!(rule.enabled);
        assert!(rule.target_paths.is_none());
    }
}
