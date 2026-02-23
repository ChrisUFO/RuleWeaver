use super::SlashCommandAdapter;
use crate::models::Command;
use std::path::PathBuf;

/// OpenCode slash command adapter
/// Format: YAML frontmatter + markdown content
/// Arguments: $ARGUMENTS, $1-$9, !`command` for shell, @filename for file refs
pub struct OpenCodeSlashAdapter;

impl SlashCommandAdapter for OpenCodeSlashAdapter {
    fn name(&self) -> &'static str {
        "opencode"
    }

    fn file_extension(&self) -> &'static str {
        "md"
    }

    fn global_dir(&self) -> &'static str {
        ".config/opencode/commands"
    }

    fn local_dir(&self) -> &'static str {
        ".opencode/commands"
    }

    fn format_command(&self, command: &Command) -> String {
        let mut output = String::new();

        // YAML frontmatter
        output.push_str("---\n");
        output.push_str(&format!("name: {}\n", command.name));
        output.push_str(&format!("description: {}\n", command.description));

        // Add arguments to frontmatter if present
        if !command.arguments.is_empty() {
            output.push_str("arguments:\n");
            for arg in &command.arguments {
                output.push_str(&format!("  - name: {}\n", arg.name));
                output.push_str(&format!("    description: {}\n", arg.description));
                output.push_str(&format!("    required: {}\n", arg.required));
            }
        }

        output.push_str("---\n\n");

        // Content with argument substitution
        let script = command.script.clone();
        output.push_str(&script);

        output
    }

    fn supports_argument_substitution(&self) -> bool {
        true
    }

    fn argument_pattern(&self) -> Option<&'static str> {
        Some("$ARGUMENTS")
    }
}

/// Claude Code slash command adapter
/// Format: YAML frontmatter + markdown (Agent Skills standard)
/// Arguments: $ARGUMENTS, $1-$9
pub struct ClaudeCodeSlashAdapter;

impl SlashCommandAdapter for ClaudeCodeSlashAdapter {
    fn name(&self) -> &'static str {
        "claude-code"
    }

    fn file_extension(&self) -> &'static str {
        "md"
    }

    fn global_dir(&self) -> &'static str {
        ".claude/commands"
    }

    fn local_dir(&self) -> &'static str {
        ".claude/commands"
    }

    fn format_command(&self, command: &Command) -> String {
        let mut output = String::new();

        // YAML frontmatter
        output.push_str("---\n");
        output.push_str(&format!("name: {}\n", command.name));
        output.push_str(&format!("description: {}\n", command.description));

        // Optional tools
        output.push_str("tools:\n");
        output.push_str("  - bash\n");

        output.push_str("---\n\n");

        // Content
        output.push_str(&command.script);

        output
    }

    fn supports_argument_substitution(&self) -> bool {
        true
    }

    fn argument_pattern(&self) -> Option<&'static str> {
        Some("$ARGUMENTS")
    }
}

/// Cline slash command adapter (Workflows)
/// Format: Markdown with numbered steps
/// Arguments: Natural language (no substitution)
pub struct ClineSlashAdapter;

impl SlashCommandAdapter for ClineSlashAdapter {
    fn name(&self) -> &'static str {
        "cline"
    }

    fn file_extension(&self) -> &'static str {
        "md"
    }

    fn global_dir(&self) -> &'static str {
        "Documents/Cline/Workflows"
    }

    fn local_dir(&self) -> &'static str {
        ".clinerules/workflows"
    }

    fn format_command(&self, command: &Command) -> String {
        let mut output = String::new();

        output.push_str(&format!("# {}\n\n", command.name));
        output.push_str(&format!("{}", command.description));
        output.push_str("\n\n");
        output.push_str(&command.script);

        output
    }

    fn supports_argument_substitution(&self) -> bool {
        false
    }
}

/// Gemini CLI slash command adapter
/// Format: TOML files
/// Arguments: {{args}}
pub struct GeminiSlashAdapter;

impl SlashCommandAdapter for GeminiSlashAdapter {
    fn name(&self) -> &'static str {
        "gemini"
    }

    fn file_extension(&self) -> &'static str {
        "toml"
    }

    fn global_dir(&self) -> &'static str {
        ".gemini/commands"
    }

    fn local_dir(&self) -> &'static str {
        ".gemini/commands"
    }

    fn format_command(&self, command: &Command) -> String {
        let mut output = String::new();

        output.push_str(&format!("description = \"{}\"\n", command.description));
        output.push_str("prompt = \"\"\"\n");
        output.push_str(&command.script);
        output.push_str("\n{{args}}\n");
        output.push_str("\"\"\"\n");

        output
    }

    fn supports_argument_substitution(&self) -> bool {
        true
    }

    fn argument_pattern(&self) -> Option<&'static str> {
        Some("{{args}}")
    }
}

/// Cursor slash command adapter
/// Format: Plain Markdown
/// Arguments: Natural language (auto-included after command)
pub struct CursorSlashAdapter;

impl SlashCommandAdapter for CursorSlashAdapter {
    fn name(&self) -> &'static str {
        "cursor"
    }

    fn file_extension(&self) -> &'static str {
        "md"
    }

    fn global_dir(&self) -> &'static str {
        ".cursor/commands"
    }

    fn local_dir(&self) -> &'static str {
        ".cursor/commands"
    }

    fn format_command(&self, command: &Command) -> String {
        let mut output = String::new();

        output.push_str(&format!("# {}\n\n", command.name));
        output.push_str(&command.script);

        output
    }

    fn supports_argument_substitution(&self) -> bool {
        false
    }
}

/// Roo Code slash command adapter
/// Format: YAML frontmatter + Markdown
/// Arguments: argument-hint in frontmatter
pub struct RooCodeSlashAdapter;

impl SlashCommandAdapter for RooCodeSlashAdapter {
    fn name(&self) -> &'static str {
        "roo"
    }

    fn file_extension(&self) -> &'static str {
        "md"
    }

    fn global_dir(&self) -> &'static str {
        ".roo/commands"
    }

    fn local_dir(&self) -> &'static str {
        ".roo/commands"
    }

    fn format_command(&self, command: &Command) -> String {
        let mut output = String::new();

        // YAML frontmatter
        output.push_str("---\n");
        output.push_str(&format!("name: {}\n", command.name));
        output.push_str(&format!("description: {}\n", command.description));

        // argument-hint if there are arguments
        if !command.arguments.is_empty() {
            let hints: Vec<String> = command
                .arguments
                .iter()
                .map(|arg| format!("<{}>", arg.name))
                .collect();
            output.push_str(&format!("argument-hint: {}\n", hints.join(" ")));
        }

        output.push_str("---\n\n");

        // Content
        output.push_str(&command.script);

        output
    }

    fn supports_argument_substitution(&self) -> bool {
        false
    }
}

/// Antigravity slash command adapter (Workflows)
/// Format: YAML frontmatter + markdown
/// Arguments: Natural language
pub struct AntigravitySlashAdapter;

impl SlashCommandAdapter for AntigravitySlashAdapter {
    fn name(&self) -> &'static str {
        "antigravity"
    }

    fn file_extension(&self) -> &'static str {
        "md"
    }

    fn global_dir(&self) -> &'static str {
        ".gemini/antigravity/global_workflows"
    }

    fn local_dir(&self) -> &'static str {
        ".agents/workflows"
    }

    fn format_command(&self, command: &Command) -> String {
        let mut output = String::new();

        // YAML frontmatter
        output.push_str("---\n");
        output.push_str(&format!("name: {}\n", command.name));
        output.push_str(&format!("description: {}\n", command.description));
        output.push_str("---\n\n");

        // Content
        output.push_str(&command.script);

        output
    }

    fn supports_argument_substitution(&self) -> bool {
        false
    }
}

/// Codex slash command adapter (Skills)
/// Format: Agent Skills directory structure
/// Note: Codex uses Skills AS slash commands
pub struct CodexSlashAdapter;

impl SlashCommandAdapter for CodexSlashAdapter {
    fn name(&self) -> &'static str {
        "codex"
    }

    fn file_extension(&self) -> &'static str {
        "md"
    }

    fn global_dir(&self) -> &'static str {
        ".agents/skills"
    }

    fn local_dir(&self) -> &'static str {
        ".agents/skills"
    }

    fn format_command(&self, command: &Command) -> String {
        let mut output = String::new();

        // YAML frontmatter
        output.push_str("---\n");
        output.push_str(&format!("name: {}\n", command.name));
        output.push_str(&format!("description: {}\n", command.description));
        output.push_str("---\n\n");

        // Content
        output.push_str(&command.script);

        output
    }

    fn get_filename(&self, command_name: &str) -> String {
        // Skills use a directory structure: {name}/SKILL.md
        format!("{}/{}", command_name, "SKILL.md")
    }

    fn get_command_path(&self, command_name: &str, is_global: bool) -> PathBuf {
        let dir = if is_global {
            self.global_dir()
        } else {
            self.local_dir()
        };
        PathBuf::from(dir).join(command_name).join("SKILL.md")
    }

    fn supports_argument_substitution(&self) -> bool {
        false
    }
}

/// Factory function to get adapter by name
pub fn get_adapter(name: &str) -> Option<Box<dyn SlashCommandAdapter>> {
    match name {
        "opencode" => Some(Box::new(OpenCodeSlashAdapter)),
        "claude-code" => Some(Box::new(ClaudeCodeSlashAdapter)),
        "cline" => Some(Box::new(ClineSlashAdapter)),
        "gemini" => Some(Box::new(GeminiSlashAdapter)),
        "cursor" => Some(Box::new(CursorSlashAdapter)),
        "roo" => Some(Box::new(RooCodeSlashAdapter)),
        "antigravity" => Some(Box::new(AntigravitySlashAdapter)),
        "codex" => Some(Box::new(CodexSlashAdapter)),
        _ => None,
    }
}

/// Get all available adapters
pub fn get_all_adapters() -> Vec<Box<dyn SlashCommandAdapter>> {
    vec![
        Box::new(OpenCodeSlashAdapter),
        Box::new(ClaudeCodeSlashAdapter),
        Box::new(ClineSlashAdapter),
        Box::new(GeminiSlashAdapter),
        Box::new(CursorSlashAdapter),
        Box::new(RooCodeSlashAdapter),
        Box::new(AntigravitySlashAdapter),
        Box::new(CodexSlashAdapter),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ArgumentType, Command, CommandArgument};

    fn create_test_command() -> Command {
        Command::new(
            "test-cmd".to_string(),
            "Test command description".to_string(),
            "npm test".to_string(),
        )
    }

    fn create_test_command_with_args() -> Command {
        let mut cmd = Command::new(
            "deploy".to_string(),
            "Deploy to environment".to_string(),
            "./scripts/deploy.sh".to_string(),
        );
        cmd.arguments = vec![
            CommandArgument {
                name: "environment".to_string(),
                description: "Target environment".to_string(),
                arg_type: ArgumentType::Enum,
                required: true,
                default_value: Some("staging".to_string()),
                options: Some(vec!["staging".to_string(), "production".to_string()]),
            },
            CommandArgument {
                name: "version".to_string(),
                description: "Version to deploy".to_string(),
                arg_type: ArgumentType::String,
                required: false,
                default_value: None,
                options: None,
            },
        ];
        cmd
    }

    #[test]
    fn test_opencode_adapter() {
        let adapter = OpenCodeSlashAdapter;
        assert_eq!(adapter.name(), "opencode");
        assert_eq!(adapter.file_extension(), "md");
        assert!(adapter.supports_argument_substitution());

        let command = create_test_command();
        let content = adapter.format_command(&command);
        assert!(content.contains("name: test-cmd"));
        assert!(content.contains("npm test"));
    }

    #[test]
    fn test_claude_adapter() {
        let adapter = ClaudeCodeSlashAdapter;
        assert_eq!(adapter.name(), "claude-code");
        assert!(adapter.supports_argument_substitution());

        let command = create_test_command();
        let content = adapter.format_command(&command);
        assert!(content.contains("tools:"));
    }

    #[test]
    fn test_gemini_adapter_toml_format() {
        let adapter = GeminiSlashAdapter;
        assert_eq!(adapter.file_extension(), "toml");

        let command = create_test_command();
        let content = adapter.format_command(&command);
        assert!(content.contains("description = "));
        assert!(content.contains("{{args}}"));
    }

    #[test]
    fn test_codex_skill_structure() {
        let adapter = CodexSlashAdapter;
        let path = adapter.get_command_path("my-skill", true);
        let path_str = path.to_string_lossy();
        assert!(path_str.contains("SKILL.md"));
    }

    #[test]
    fn test_all_adapters_produce_content() {
        let adapters: Vec<Box<dyn SlashCommandAdapter>> = vec![
            Box::new(OpenCodeSlashAdapter),
            Box::new(ClaudeCodeSlashAdapter),
            Box::new(ClineSlashAdapter),
            Box::new(GeminiSlashAdapter),
            Box::new(CursorSlashAdapter),
            Box::new(RooCodeSlashAdapter),
            Box::new(AntigravitySlashAdapter),
            Box::new(CodexSlashAdapter),
        ];

        let command = create_test_command();

        for adapter in adapters {
            let content = adapter.format_command(&command);
            assert!(
                !content.is_empty(),
                "Adapter {} should produce non-empty content",
                adapter.name()
            );
        }
    }

    #[test]
    fn test_adapter_name_uniqueness() {
        let names = vec![
            OpenCodeSlashAdapter.name(),
            ClaudeCodeSlashAdapter.name(),
            ClineSlashAdapter.name(),
            GeminiSlashAdapter.name(),
            CursorSlashAdapter.name(),
            RooCodeSlashAdapter.name(),
            AntigravitySlashAdapter.name(),
            CodexSlashAdapter.name(),
        ];

        let unique: std::collections::HashSet<_> = names.iter().cloned().collect();
        assert_eq!(names.len(), unique.len(), "Adapter names must be unique");
    }
}
