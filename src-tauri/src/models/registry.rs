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
    pub command_stub_filename: &'static str,
    pub global_skills_dir: Option<&'static str>,
    pub local_skills_dir: Option<&'static str>,
    pub skill_filename: &'static str,
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
                    command_stub_filename: "COMMANDS.md",
                    global_skills_dir: Some(".gemini/antigravity/skills"),
                    local_skills_dir: Some(".agents/skills"),
                    skill_filename: "SKILL.md",
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
                    command_stub_filename: "COMMANDS.md",
                    global_skills_dir: Some(".gemini/skills"),
                    local_skills_dir: Some(".gemini/skills"),
                    skill_filename: "SKILL.md",
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
                    command_stub_filename: "COMMANDS.md",
                    global_skills_dir: Some(".config/opencode/skills"),
                    local_skills_dir: Some(".opencode/skills"),
                    skill_filename: "SKILL.md",
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
                    command_stub_filename: "COMMANDS.md",
                    global_skills_dir: Some("Documents/Cline/Skills"),
                    local_skills_dir: Some(".clinerules/skills"),
                    skill_filename: "SKILL.md",
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
                    command_stub_filename: "COMMANDS.md",
                    global_skills_dir: Some(".claude/skills"),
                    local_skills_dir: Some(".claude/skills"),
                    skill_filename: "SKILL.md",
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
                    command_stub_filename: "COMMANDS.md",
                    global_skills_dir: Some(".codex/skills"),
                    local_skills_dir: Some(".codex/skills"),
                    skill_filename: "SKILL.md",
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
                // Kilo Code supports rules only. Command stubs, slash commands, and skills are
                // not yet distributed because paths are not configured.
                capabilities: ToolCapabilities {
                    supports_rules: true,
                    supports_command_stubs: false,
                    supports_slash_commands: false,
                    supports_skills: false,
                    supports_global_scope: true,
                    supports_local_scope: true,
                },
                paths: PathTemplates {
                    global_path: "~/.kilocode/rules/AGENTS.md",
                    local_path_template: ".kilocode/rules/AGENTS.md",
                    global_commands_dir: None,
                    local_commands_dir: None,
                    command_stub_filename: "COMMANDS.md",
                    global_skills_dir: None,
                    local_skills_dir: None,
                    skill_filename: "SKILL.md",
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
                    command_stub_filename: "COMMANDS.md",
                    global_skills_dir: None,
                    local_skills_dir: None,
                    skill_filename: "SKILL.md",
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
                // Windsurf supports rules and skills. Command stubs and slash commands are not
                // distributed because no path or extension is configured.
                capabilities: ToolCapabilities {
                    supports_rules: true,
                    supports_command_stubs: false,
                    supports_slash_commands: false,
                    supports_skills: true,
                    supports_global_scope: true,
                    supports_local_scope: true,
                },
                paths: PathTemplates {
                    global_path: "~/.windsurf/rules/rules.md",
                    local_path_template: ".windsurf/rules/rules.md",
                    global_commands_dir: None,
                    local_commands_dir: None,
                    command_stub_filename: "COMMANDS.md",
                    global_skills_dir: Some(".windsurf/skills"),
                    local_skills_dir: Some(".windsurf/skills"),
                    skill_filename: "SKILL.md",
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
                    command_stub_filename: "COMMANDS.md",
                    global_skills_dir: Some(".roo/skills"),
                    local_skills_dir: Some(".roo/skills"),
                    skill_filename: "SKILL.md",
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

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Render the capability-flags section of the support matrix.
fn capability_flags_table(sorted_adapters: &[crate::models::rule::AdapterType], registry: &ToolRegistry) -> String {
    let yn = |b: bool| if b { "✅" } else { "❌" };
    let mut out = String::new();
    out.push_str("## Capability Flags\n\n");
    out.push_str("| Tool | Rules | Command Stubs | Slash Commands | Skills | Global Scope | Local Scope |\n");
    out.push_str("| ---- | :---: | :-----------: | :------------: | :----: | :----------: | :---------: |\n");
    for adapter in sorted_adapters {
        if let Some(entry) = registry.get(adapter) {
            let c = &entry.capabilities;
            out.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} | {} |\n",
                entry.name,
                yn(c.supports_rules),
                yn(c.supports_command_stubs),
                yn(c.supports_slash_commands),
                yn(c.supports_skills),
                yn(c.supports_global_scope),
                yn(c.supports_local_scope),
            ));
        }
    }
    out
}

/// Render the path-configuration section of the support matrix.
fn path_configuration_table(sorted_adapters: &[crate::models::rule::AdapterType], registry: &ToolRegistry) -> String {
    let opt = |o: Option<&'static str>| o.unwrap_or("—");
    let mut out = String::new();
    out.push_str("## Path Configuration\n\n");
    out.push_str("Paths prefixed with `~/` expand to the user home directory at runtime.\n\n");
    out.push_str("| Tool | Rules (Global) | Rules (Local) | Commands Dir (Global) | Commands Dir (Local) | Skills Dir (Global) | Skills Dir (Local) |\n");
    out.push_str("| ---- | -------------- | ------------- | --------------------- | -------------------- | ------------------- | ------------------ |\n");
    for adapter in sorted_adapters {
        if let Some(entry) = registry.get(adapter) {
            let p = &entry.paths;
            out.push_str(&format!(
                "| {} | `{}` | `{}` | {} | {} | {} | {} |\n",
                entry.name,
                p.global_path,
                p.local_path_template,
                opt(p.global_commands_dir).replace('|', "\\|"),
                opt(p.local_commands_dir).replace('|', "\\|"),
                opt(p.global_skills_dir).replace('|', "\\|"),
                opt(p.local_skills_dir).replace('|', "\\|"),
            ));
        }
    }
    out
}

/// Render the slash-command-extensions section of the support matrix.
fn slash_command_extensions_table(sorted_adapters: &[crate::models::rule::AdapterType], registry: &ToolRegistry) -> String {
    let mut out = String::new();
    out.push_str("## Slash Command Extensions\n\n");
    out.push_str("| Tool | File Extension | Argument Pattern |\n");
    out.push_str("| ---- | -------------- | ---------------- |\n");
    for adapter in sorted_adapters {
        if let Some(entry) = registry.get(adapter) {
            let ext = entry.slash_command_extension.unwrap_or("—");
            let pattern = entry.slash_command_argument_pattern.unwrap_or("—");
            out.push_str(&format!("| {} | `{}` | `{}` |\n", entry.name, ext, pattern));
        }
    }
    out
}

/// Generate the canonical support matrix markdown content from the live REGISTRY.
///
/// This function is the single source of truth for `docs/SUPPORT_MATRIX.md`.
/// Both the `gen_docs` binary and the `test_support_matrix_is_current` test call
/// this function to ensure the committed file always matches the registry.
pub fn generate_support_matrix() -> String {
    use crate::models::rule::AdapterType;

    let registry = &REGISTRY;
    let mut sorted_adapters = AdapterType::all();
    sorted_adapters.sort_by_key(|a| a.as_str());

    let mut out = String::new();
    out.push_str("<!-- AUTO-GENERATED: do not edit manually. Run `cargo run --bin gen_docs` to regenerate. -->\n");
    out.push_str("# RuleWeaver Tool Support Matrix\n\n");
    out.push_str("Generated from `src-tauri/src/models/registry.rs`. Any change to adapter capabilities or paths must be followed by running `cargo run --bin gen_docs` and committing the updated file.\n\n");
    out.push_str("---\n\n");

    out.push_str(&capability_flags_table(&sorted_adapters, registry));
    out.push('\n');
    out.push_str("---\n\n");

    out.push_str(&path_configuration_table(&sorted_adapters, registry));
    out.push('\n');
    out.push_str("---\n\n");

    out.push_str(&slash_command_extensions_table(&sorted_adapters, registry));
    out.push('\n');
    out.push_str("---\n\n");
    out.push_str("*See `docs/PARITY.md` for documented divergences and known unsupported combinations.*\n");

    out
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

    #[test]
    fn test_skill_paths_defined_for_supporting_adapters() {
        let registry = get_registry();

        let claude = registry.get(&AdapterType::ClaudeCode).unwrap();
        assert!(claude.capabilities.supports_skills);
        assert!(claude.paths.global_skills_dir.is_some());
        assert!(claude.paths.local_skills_dir.is_some());

        let opencode = registry.get(&AdapterType::OpenCode).unwrap();
        assert!(opencode.capabilities.supports_skills);
        assert!(opencode.paths.global_skills_dir.is_some());
    }

    #[test]
    fn test_skill_paths_not_defined_for_non_supporting_adapters() {
        let registry = get_registry();

        let cursor = registry.get(&AdapterType::Cursor).unwrap();
        assert!(!cursor.capabilities.supports_skills);
        assert!(cursor.paths.global_skills_dir.is_none());
        assert!(cursor.paths.local_skills_dir.is_none());

        let kilo = registry.get(&AdapterType::Kilo).unwrap();
        assert!(kilo.paths.global_skills_dir.is_none());
        assert!(kilo.paths.local_skills_dir.is_none());
    }

    #[test]
    fn test_all_adapters_have_skill_filename() {
        let registry = get_registry();

        for entry in registry.all() {
            assert!(!entry.paths.skill_filename.is_empty());
        }
    }

    /// Verifies that `docs/SUPPORT_MATRIX.md` matches what `generate_support_matrix()` produces.
    ///
    /// If this test fails, run `cargo run --bin gen_docs` from the workspace root to regenerate
    /// the file, then commit the updated `docs/SUPPORT_MATRIX.md`.
    #[test]
    fn test_support_matrix_is_current() {
        let generated = generate_support_matrix();

        // Locate docs/SUPPORT_MATRIX.md relative to the workspace root.
        // CARGO_MANIFEST_DIR points to src-tauri/; workspace root is one level up.
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let workspace_root = manifest_dir.parent().expect("workspace root must exist");
        let matrix_path = workspace_root.join("docs").join("SUPPORT_MATRIX.md");

        let on_disk = std::fs::read_to_string(&matrix_path)
            .expect(
                "docs/SUPPORT_MATRIX.md does not exist or is not readable. \
                Run `cargo run --bin gen_docs` to generate it.",
            )
            // Normalize CRLF → LF so the test passes on Windows where git
            // core.autocrlf may convert line endings on checkout.
            .replace("\r\n", "\n");

        assert_eq!(
            generated, on_disk,
            "\ndocs/SUPPORT_MATRIX.md is stale.\n\
            Run `cargo run --bin gen_docs` from the workspace root and commit the result.\n"
        );
    }

    #[test]
    fn test_generate_support_matrix_contains_all_adapter_names() {
        let matrix = generate_support_matrix();
        assert!(matrix.contains("Claude Code"), "Matrix must contain Claude Code");
        assert!(matrix.contains("Cursor"), "Matrix must contain Cursor");
        assert!(matrix.contains("Windsurf"), "Matrix must contain Windsurf");
        assert!(matrix.contains("Kilo Code"), "Matrix must contain Kilo Code");
        assert!(matrix.contains("Antigravity"), "Matrix must contain Antigravity");
        assert!(matrix.contains("Gemini"), "Matrix must contain Gemini");
        assert!(matrix.contains("OpenCode"), "Matrix must contain OpenCode");
        assert!(matrix.contains("Cline"), "Matrix must contain Cline");
        assert!(matrix.contains("Codex"), "Matrix must contain Codex");
        assert!(matrix.contains("Roo Code"), "Matrix must contain Roo Code");
    }

    #[test]
    fn test_generate_support_matrix_cursor_shows_no_skills() {
        let matrix = generate_support_matrix();
        // Find Cursor row and verify it has ❌ in the Skills column (4th ❌/✅ in the row)
        let cursor_line = matrix
            .lines()
            .find(|l| l.starts_with("| Cursor"))
            .expect("Cursor row must be in matrix");
        assert!(
            cursor_line.contains("❌"),
            "Cursor row must contain ❌ for unsupported capabilities: {}",
            cursor_line
        );
    }

    #[test]
    fn test_generate_support_matrix_windsurf_has_skill_paths() {
        let matrix = generate_support_matrix();
        // Windsurf's path row should contain the windsurf/skills path
        let windsurf_path_line = matrix
            .lines()
            .find(|l| l.starts_with("| Windsurf") && l.contains("windsurf/skills"))
            .is_some();
        assert!(
            windsurf_path_line,
            "Windsurf must have windsurf/skills paths in the matrix"
        );
    }
}
