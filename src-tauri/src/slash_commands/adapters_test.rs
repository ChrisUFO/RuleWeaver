use crate::models::{Command, CommandArgument, CreateCommandInput};
use crate::slash_commands::SlashCommandAdapter;
use crate::slash_commands::{
    AntigravitySlashAdapter, ClaudeCodeSlashAdapter, ClineSlashAdapter, CodexSlashAdapter,
    CursorSlashAdapter, GeminiSlashAdapter, OpenCodeSlashAdapter, RooCodeSlashAdapter,
};

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
            arg_type: crate::models::ArgumentType::Enum,
            required: true,
            default_value: Some("staging".to_string()),
            options: Some(vec!["staging".to_string(), "production".to_string()]),
        },
        CommandArgument {
            name: "version".to_string(),
            description: "Version to deploy".to_string(),
            arg_type: crate::models::ArgumentType::String,
            required: false,
            default_value: None,
            options: None,
        },
    ];
    cmd
}

#[test]
fn test_opencode_adapter_name() {
    let adapter = OpenCodeSlashAdapter;
    assert_eq!(adapter.name(), "opencode");
}

#[test]
fn test_opencode_adapter_extension() {
    let adapter = OpenCodeSlashAdapter;
    assert_eq!(adapter.file_extension(), "md");
}

#[test]
fn test_opencode_adapter_paths() {
    let adapter = OpenCodeSlashAdapter;
    assert_eq!(adapter.global_dir(), ".config/opencode/commands");
    assert_eq!(adapter.local_dir(), ".opencode/commands");
}

#[test]
fn test_opencode_format_basic() {
    let adapter = OpenCodeSlashAdapter;
    let command = create_test_command();
    let content = adapter.format_command(&command);

    assert!(content.contains("name: test-cmd"));
    assert!(content.contains("description: Test command description"));
    assert!(content.contains("npm test"));
}

#[test]
fn test_opencode_format_with_args() {
    let adapter = OpenCodeSlashAdapter;
    let command = create_test_command_with_args();
    let content = adapter.format_command(&command);

    assert!(content.contains("name: deploy"));
    assert!(content.contains("arguments:"));
    assert!(content.contains("  - name: environment"));
    assert!(content.contains("./scripts/deploy.sh"));
}

#[test]
fn test_opencode_argument_substitution() {
    let adapter = OpenCodeSlashAdapter;
    assert!(adapter.supports_argument_substitution());
    assert_eq!(adapter.argument_pattern(), Some("$ARGUMENTS"));
}

#[test]
fn test_claude_adapter_name() {
    let adapter = ClaudeCodeSlashAdapter;
    assert_eq!(adapter.name(), "claude-code");
}

#[test]
fn test_claude_adapter_extension() {
    let adapter = ClaudeCodeSlashAdapter;
    assert_eq!(adapter.file_extension(), "md");
}

#[test]
fn test_claude_format_basic() {
    let adapter = ClaudeCodeSlashAdapter;
    let command = create_test_command();
    let content = adapter.format_command(&command);

    assert!(content.contains("name: test-cmd"));
    assert!(content.contains("description: Test command description"));
    assert!(content.contains("tools:"));
    assert!(content.contains("npm test"));
}

#[test]
fn test_claude_argument_substitution() {
    let adapter = ClaudeCodeSlashAdapter;
    assert!(adapter.supports_argument_substitution());
    assert_eq!(adapter.argument_pattern(), Some("$ARGUMENTS"));
}

#[test]
fn test_cline_adapter_name() {
    let adapter = ClineSlashAdapter;
    assert_eq!(adapter.name(), "cline");
}

#[test]
fn test_cline_adapter_paths() {
    let adapter = ClineSlashAdapter;
    assert_eq!(adapter.global_dir(), "Documents/Cline/Workflows");
    assert_eq!(adapter.local_dir(), ".clinerules/workflows");
}

#[test]
fn test_cline_format_basic() {
    let adapter = ClineSlashAdapter;
    let command = create_test_command();
    let content = adapter.format_command(&command);

    assert!(content.contains("# test-cmd"));
    assert!(content.contains("Test command description"));
    assert!(content.contains("npm test"));
}

#[test]
fn test_cline_no_argument_substitution() {
    let adapter = ClineSlashAdapter;
    assert!(!adapter.supports_argument_substitution());
}

#[test]
fn test_gemini_adapter_name() {
    let adapter = GeminiSlashAdapter;
    assert_eq!(adapter.name(), "gemini");
}

#[test]
fn test_gemini_adapter_extension() {
    let adapter = GeminiSlashAdapter;
    assert_eq!(adapter.file_extension(), "toml");
}

#[test]
fn test_gemini_format_basic() {
    let adapter = GeminiSlashAdapter;
    let command = create_test_command();
    let content = adapter.format_command(&command);

    assert!(content.contains("description = \"Test command description\""));
    assert!(content.contains("prompt = \"\"\""));
    assert!(content.contains("npm test"));
    assert!(content.contains("{{args}}"));
}

#[test]
fn test_gemini_argument_substitution() {
    let adapter = GeminiSlashAdapter;
    assert!(adapter.supports_argument_substitution());
    assert_eq!(adapter.argument_pattern(), Some("{{args}}"));
}

#[test]
fn test_cursor_adapter_name() {
    let adapter = CursorSlashAdapter;
    assert_eq!(adapter.name(), "cursor");
}

#[test]
fn test_cursor_format_basic() {
    let adapter = CursorSlashAdapter;
    let command = create_test_command();
    let content = adapter.format_command(&command);

    assert!(content.contains("# test-cmd"));
    assert!(content.contains("npm test"));
}

#[test]
fn test_cursor_no_argument_substitution() {
    let adapter = CursorSlashAdapter;
    assert!(!adapter.supports_argument_substitution());
}

#[test]
fn test_roo_adapter_name() {
    let adapter = RooCodeSlashAdapter;
    assert_eq!(adapter.name(), "roo");
}

#[test]
fn test_roo_format_basic() {
    let adapter = RooCodeSlashAdapter;
    let command = create_test_command();
    let content = adapter.format_command(&command);

    assert!(content.contains("name: test-cmd"));
    assert!(content.contains("description: Test command description"));
    assert!(content.contains("npm test"));
}

#[test]
fn test_roo_format_with_args() {
    let adapter = RooCodeSlashAdapter;
    let command = create_test_command_with_args();
    let content = adapter.format_command(&command);

    assert!(content.contains("argument-hint: <environment> <version>"));
}

#[test]
fn test_roo_no_argument_substitution() {
    let adapter = RooCodeSlashAdapter;
    assert!(!adapter.supports_argument_substitution());
}

#[test]
fn test_antigravity_adapter_name() {
    let adapter = AntigravitySlashAdapter;
    assert_eq!(adapter.name(), "antigravity");
}

#[test]
fn test_antigravity_adapter_paths() {
    let adapter = AntigravitySlashAdapter;
    assert_eq!(adapter.global_dir(), ".gemini/antigravity/global_workflows");
    assert_eq!(adapter.local_dir(), ".agents/workflows");
}

#[test]
fn test_antigravity_format_basic() {
    let adapter = AntigravitySlashAdapter;
    let command = create_test_command();
    let content = adapter.format_command(&command);

    assert!(content.contains("name: test-cmd"));
    assert!(content.contains("description: Test command description"));
    assert!(content.contains("npm test"));
}

#[test]
fn test_codex_adapter_name() {
    let adapter = CodexSlashAdapter;
    assert_eq!(adapter.name(), "codex");
}

#[test]
fn test_codex_adapter_paths() {
    let adapter = CodexSlashAdapter;
    assert_eq!(adapter.global_dir(), ".agents/skills");
    assert_eq!(adapter.local_dir(), ".agents/skills");
}

#[test]
fn test_codex_filename_format() {
    let adapter = CodexSlashAdapter;
    let filename = adapter.get_filename("my-skill");
    assert_eq!(filename, "my-skill/SKILL.md");
}

#[test]
fn test_codex_command_path() {
    let adapter = CodexSlashAdapter;
    let path = adapter.get_command_path("my-skill", true);
    assert_eq!(path.to_string_lossy(), ".agents/skills/my-skill/SKILL.md");
}

#[test]
fn test_codex_format_basic() {
    let adapter = CodexSlashAdapter;
    let command = create_test_command();
    let content = adapter.format_command(&command);

    assert!(content.contains("name: test-cmd"));
    assert!(content.contains("description: Test command description"));
    assert!(content.contains("npm test"));
}

#[test]
fn test_all_adapters_implement_trait() {
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

    for adapter in adapters {
        let command = create_test_command();
        let content = adapter.format_command(&command);
        assert!(
            !content.is_empty(),
            "Adapter {} produced empty content",
            adapter.name()
        );
    }
}

#[test]
fn test_unique_adapter_names() {
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

    let unique_names: std::collections::HashSet<_> = names.iter().cloned().collect();
    assert_eq!(
        names.len(),
        unique_names.len(),
        "Adapter names should be unique"
    );
}
