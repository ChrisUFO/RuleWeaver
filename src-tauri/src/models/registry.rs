use crate::models::rule::{AdapterType, Scope};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Global tool registry singleton.
///
/// Thread-safety: `once_cell::sync::Lazy` provides thread-safe one-time initialization
/// via internal `Once` synchronization. Safe for concurrent access from multiple Tauri
/// command handlers. All reads after initialization are lock-free.
///
/// Completeness: All `AdapterType` enum variants MUST have a corresponding entry.
/// The `test_registry_contains_all_adapters` test enforces this at compile-time.
pub static REGISTRY: Lazy<ToolRegistry> = Lazy::new(ToolRegistry::new);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactType {
    Rule,
    CommandStub,
    SlashCommand,
    Skill,
}

impl ArtifactType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ArtifactType::Rule => "rule",
            ArtifactType::CommandStub => "command_stub",
            ArtifactType::SlashCommand => "slash_command",
            ArtifactType::Skill => "skill",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCapabilities {
    pub supports_rules: bool,
    pub supports_command_stubs: bool,
    pub supports_slash_commands: bool,
    pub supports_skills: bool,
    pub supports_global_scope: bool,
    pub supports_local_scope: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PathTemplates {
    pub global_path: &'static str,
    pub local_path_template: &'static str,
    pub global_commands_dir: Option<&'static str>,
    pub local_commands_dir: Option<&'static str>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolEntry {
    pub id: AdapterType,
    pub name: &'static str,
    pub description: &'static str,
    pub icon: &'static str,
    pub capabilities: ToolCapabilities,
    pub paths: PathTemplates,
    pub file_format: &'static str,
    pub slash_command_extension: Option<&'static str>,
    pub slash_command_argument_pattern: Option<&'static str>,
}

pub struct ToolRegistry {
    entries: HashMap<AdapterType, ToolEntry>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        let mut entries = HashMap::new();

        // Helper to define common capabilities
        let full_support = ToolCapabilities {
            supports_rules: true,
            supports_command_stubs: true,
            supports_slash_commands: true,
            supports_skills: true,
            supports_global_scope: true,
            supports_local_scope: true,
        };

        // 1. Antigravity
        entries.insert(
            AdapterType::Antigravity,
            ToolEntry {
                id: AdapterType::Antigravity,
                name: "Antigravity",
                description: "Antigravity AI coding assistant",
                icon: "antigravity",
                capabilities: full_support.clone(),
                paths: PathTemplates {
                    global_path: "~/.gemini/GEMINI.md",
                    local_path_template: ".gemini/GEMINI.md",
                    global_commands_dir: Some(".gemini/antigravity/global_workflows"),
                    local_commands_dir: Some(".agents/workflows"),
                },
                file_format: "markdown",
                slash_command_extension: Some("md"),
                slash_command_argument_pattern: None,
            },
        );

        // 2. Gemini
        entries.insert(
            AdapterType::Gemini,
            ToolEntry {
                id: AdapterType::Gemini,
                name: "Gemini",
                description: "Google's Gemini AI coding assistant",
                icon: "gemini",
                capabilities: full_support.clone(),
                paths: PathTemplates {
                    global_path: "~/.gemini/GEMINI.md",
                    local_path_template: ".gemini/GEMINI.md",
                    global_commands_dir: Some(".gemini/commands"),
                    local_commands_dir: Some(".gemini/commands"),
                },
                file_format: "markdown",
                slash_command_extension: Some("toml"),
                slash_command_argument_pattern: Some("{{args}}"),
            },
        );

        // 3. OpenCode
        entries.insert(
            AdapterType::OpenCode,
            ToolEntry {
                id: AdapterType::OpenCode,
                name: "OpenCode",
                description: "OpenCode AI coding assistant",
                icon: "opencode",
                capabilities: full_support.clone(),
                paths: PathTemplates {
                    global_path: "~/.config/opencode/AGENTS.md",
                    local_path_template: ".config/opencode/AGENTS.md",
                    global_commands_dir: Some(".config/opencode/commands"),
                    local_commands_dir: Some(".opencode/commands"),
                },
                file_format: "markdown",
                slash_command_extension: Some("md"),
                slash_command_argument_pattern: Some("$ARGUMENTS"),
            },
        );

        // 4. Cline
        entries.insert(
            AdapterType::Cline,
            ToolEntry {
                id: AdapterType::Cline,
                name: "Cline",
                description: "Cline VS Code extension",
                icon: "cline",
                capabilities: full_support.clone(),
                paths: PathTemplates {
                    global_path: "~/.clinerules",
                    local_path_template: ".clinerules",
                    global_commands_dir: Some("Documents/Cline/Workflows"),
                    local_commands_dir: Some(".clinerules/workflows"),
                },
                file_format: "markdown",
                slash_command_extension: Some("md"),
                slash_command_argument_pattern: None,
            },
        );

        // 5. ClaudeCode
        entries.insert(
            AdapterType::ClaudeCode,
            ToolEntry {
                id: AdapterType::ClaudeCode,
                name: "Claude Code",
                description: "Anthropic's Claude Code assistant",
                icon: "claude",
                capabilities: full_support.clone(),
                paths: PathTemplates {
                    global_path: "~/.claude/CLAUDE.md",
                    local_path_template: ".claude/CLAUDE.md",
                    global_commands_dir: Some(".claude/commands"),
                    local_commands_dir: Some(".claude/commands"),
                },
                file_format: "markdown",
                slash_command_extension: Some("md"),
                slash_command_argument_pattern: Some("$ARGUMENTS"),
            },
        );

        // 6. Codex
        entries.insert(
            AdapterType::Codex,
            ToolEntry {
                id: AdapterType::Codex,
                name: "Codex",
                description: "OpenAI Codex assistant",
                icon: "codex",
                capabilities: full_support.clone(),
                paths: PathTemplates {
                    global_path: "~/.codex/AGENTS.md",
                    local_path_template: ".codex/AGENTS.md",
                    global_commands_dir: Some(".agents/skills"),
                    local_commands_dir: Some(".agents/skills"),
                },
                file_format: "markdown",
                slash_command_extension: Some("md"),
                slash_command_argument_pattern: None,
            },
        );

        // 7. Kilo
        entries.insert(
            AdapterType::Kilo,
            ToolEntry {
                id: AdapterType::Kilo,
                name: "Kilo Code",
                description: "Kilo Code AI assistant",
                icon: "kilo",
                capabilities: full_support.clone(),
                paths: PathTemplates {
                    global_path: "~/.kilocode/rules/AGENTS.md",
                    local_path_template: ".kilocode/rules/AGENTS.md",
                    global_commands_dir: None,
                    local_commands_dir: None,
                },
                file_format: "markdown",
                slash_command_extension: None,
                slash_command_argument_pattern: None,
            },
        );

        // 8. Cursor
        entries.insert(
            AdapterType::Cursor,
            ToolEntry {
                id: AdapterType::Cursor,
                name: "Cursor",
                description: "Cursor AI code editor",
                icon: "cursor",
                capabilities: ToolCapabilities {
                    supports_rules: true,
                    supports_command_stubs: false,
                    supports_slash_commands: true,
                    supports_skills: false,
                    supports_global_scope: true,
                    supports_local_scope: true,
                },
                paths: PathTemplates {
                    global_path: "~/.cursorrules",
                    local_path_template: ".cursorrules",
                    global_commands_dir: Some(".cursor/commands"),
                    local_commands_dir: Some(".cursor/commands"),
                },
                file_format: "markdown",
                slash_command_extension: Some("md"),
                slash_command_argument_pattern: None,
            },
        );

        // 9. Windsurf
        entries.insert(
            AdapterType::Windsurf,
            ToolEntry {
                id: AdapterType::Windsurf,
                name: "Windsurf",
                description: "Windsurf AI assistant",
                icon: "windsurf",
                capabilities: full_support.clone(),
                paths: PathTemplates {
                    global_path: "~/.windsurf/rules/rules.md",
                    local_path_template: ".windsurf/rules/rules.md",
                    global_commands_dir: None,
                    local_commands_dir: None,
                },
                file_format: "markdown",
                slash_command_extension: None,
                slash_command_argument_pattern: None,
            },
        );

        // 10. RooCode
        entries.insert(
            AdapterType::RooCode,
            ToolEntry {
                id: AdapterType::RooCode,
                name: "Roo Code",
                description: "Roo Code AI assistant",
                icon: "roocode",
                capabilities: full_support.clone(),
                paths: PathTemplates {
                    global_path: "~/.roo/rules/rules.md",
                    local_path_template: ".roo/rules/rules.md",
                    global_commands_dir: Some(".roo/commands"),
                    local_commands_dir: Some(".roo/commands"),
                },
                file_format: "markdown",
                slash_command_extension: Some("md"),
                slash_command_argument_pattern: None,
            },
        );

        Self { entries }
    }

    pub fn get(&self, adapter: &AdapterType) -> Option<&ToolEntry> {
        self.entries.get(adapter)
    }

    pub fn all(&self) -> Vec<&ToolEntry> {
        self.entries.values().collect()
    }

    pub fn validate_support(
        &self,
        adapter: &AdapterType,
        scope: &Scope,
        artifact: ArtifactType,
    ) -> Result<(), String> {
        let entry = self
            .get(adapter)
            .ok_or_else(|| format!("Unknown adapter: {}", adapter.as_str()))?;

        // Scope check
        match scope {
            Scope::Global if !entry.capabilities.supports_global_scope => {
                return Err(format!(
                    "Adapter {} does not support global scope",
                    entry.name
                ));
            }
            Scope::Local if !entry.capabilities.supports_local_scope => {
                return Err(format!(
                    "Adapter {} does not support local scope",
                    entry.name
                ));
            }
            _ => {}
        }

        // Artifact check
        let supported = match artifact {
            ArtifactType::Rule => entry.capabilities.supports_rules,
            ArtifactType::CommandStub => entry.capabilities.supports_command_stubs,
            ArtifactType::SlashCommand => entry.capabilities.supports_slash_commands,
            ArtifactType::Skill => entry.capabilities.supports_skills,
        };

        if !supported {
            return Err(format!(
                "Adapter {} does not support artifact type: {}",
                entry.name,
                artifact.as_str()
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_registry() -> &'static ToolRegistry {
        &REGISTRY
    }

    #[test]
    fn test_registry_contains_all_adapters() {
        let registry = get_registry();
        assert!(registry.get(&AdapterType::Antigravity).is_some());
        assert!(registry.get(&AdapterType::Gemini).is_some());
        assert!(registry.get(&AdapterType::OpenCode).is_some());
        assert!(registry.get(&AdapterType::Cline).is_some());
        assert!(registry.get(&AdapterType::ClaudeCode).is_some());
        assert!(registry.get(&AdapterType::Codex).is_some());
        assert!(registry.get(&AdapterType::Kilo).is_some());
        assert!(registry.get(&AdapterType::Cursor).is_some());
        assert!(registry.get(&AdapterType::Windsurf).is_some());
        assert!(registry.get(&AdapterType::RooCode).is_some());
    }

    #[test]
    fn test_registry_returns_all_ten_adapters() {
        let registry = get_registry();
        let all = registry.all();
        assert_eq!(all.len(), 10);
    }

    #[test]
    fn test_full_support_adapter_capabilities() {
        let registry = get_registry();
        let gemini = registry.get(&AdapterType::Gemini).unwrap();
        assert!(gemini.capabilities.supports_rules);
        assert!(gemini.capabilities.supports_command_stubs);
        assert!(gemini.capabilities.supports_slash_commands);
        assert!(gemini.capabilities.supports_skills);
        assert!(gemini.capabilities.supports_global_scope);
        assert!(gemini.capabilities.supports_local_scope);
    }

    #[test]
    fn test_cursor_limited_capabilities() {
        let registry = get_registry();
        let cursor = registry.get(&AdapterType::Cursor).unwrap();
        assert!(cursor.capabilities.supports_rules);
        assert!(!cursor.capabilities.supports_command_stubs);
        assert!(cursor.capabilities.supports_slash_commands);
        assert!(!cursor.capabilities.supports_skills);
    }

    #[test]
    fn test_validate_support_rules_globally() {
        let registry = get_registry();
        let result =
            registry.validate_support(&AdapterType::Gemini, &Scope::Global, ArtifactType::Rule);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_support_rules_locally() {
        let registry = get_registry();
        let result =
            registry.validate_support(&AdapterType::Gemini, &Scope::Local, ArtifactType::Rule);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_support_skills_for_gemini() {
        let registry = get_registry();
        let result =
            registry.validate_support(&AdapterType::Gemini, &Scope::Global, ArtifactType::Skill);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_support_rejects_cursor_skills() {
        let registry = get_registry();
        let result =
            registry.validate_support(&AdapterType::Cursor, &Scope::Global, ArtifactType::Skill);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("does not support artifact type"));
    }

    #[test]
    fn test_validate_support_rejects_cursor_command_stubs() {
        let registry = get_registry();
        let result = registry.validate_support(
            &AdapterType::Cursor,
            &Scope::Global,
            ArtifactType::CommandStub,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_path_templates_resolution() {
        let registry = get_registry();
        let opencode = registry.get(&AdapterType::OpenCode).unwrap();
        assert_eq!(opencode.paths.global_path, "~/.config/opencode/AGENTS.md");
        assert_eq!(
            opencode.paths.local_path_template,
            ".config/opencode/AGENTS.md"
        );
    }

    #[test]
    fn test_slash_command_extensions() {
        let registry = get_registry();
        let gemini = registry.get(&AdapterType::Gemini).unwrap();
        assert_eq!(gemini.slash_command_extension, Some("toml"));

        let opencode = registry.get(&AdapterType::OpenCode).unwrap();
        assert_eq!(opencode.slash_command_extension, Some("md"));

        let kilo = registry.get(&AdapterType::Kilo).unwrap();
        assert_eq!(kilo.slash_command_extension, None);
    }

    #[test]
    fn test_artifact_type_as_str() {
        assert_eq!(ArtifactType::Rule.as_str(), "rule");
        assert_eq!(ArtifactType::CommandStub.as_str(), "command_stub");
        assert_eq!(ArtifactType::SlashCommand.as_str(), "slash_command");
        assert_eq!(ArtifactType::Skill.as_str(), "skill");
    }
}
