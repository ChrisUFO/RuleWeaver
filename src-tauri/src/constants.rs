use std::time::Duration;

pub mod timing {
    use super::*;
    pub const CMD_EXEC_TIMEOUT: Duration = Duration::from_secs(60);
    pub const SKILL_EXEC_TIMEOUT: Duration = Duration::from_secs(60);
    pub const TEST_CMD_TIMEOUT: Duration = Duration::from_secs(30);
    pub const MCP_RATE_LIMIT_WINDOW: Duration = Duration::from_secs(10);
    pub const TEST_CMD_RATE_LIMIT_WINDOW: Duration = Duration::from_secs(60);
    pub const MCP_SERVER_BACKOFF_INITIAL_MS: u64 = 100;
}

pub mod limits {
    pub const MAX_ARG_LENGTH: usize = 2000;
    pub const MAX_SCRIPT_LENGTH: usize = 20000;
    pub const MAX_OUTPUT_SIZE: usize = 10 * 1024 * 1024; // 10MB
    pub const LOG_LIMIT: usize = 500;
    pub const MCP_RATE_LIMIT_MAX_CALLS: usize = 10;
    pub const TEST_CMD_RATE_LIMIT_MAX: usize = 5;
    pub const MAX_RULE_NAME_LENGTH: usize = 200;
    pub const MAX_RULE_CONTENT_LENGTH: usize = 1_000_000;
    pub const MAX_COMMAND_NAME_LENGTH: usize = 120;
    pub const MAX_COMMAND_SCRIPT_LENGTH: usize = 10_000;
    pub const MAX_SKILL_NAME_LENGTH: usize = 160;
    pub const MAX_SKILL_INSTRUCTIONS_LENGTH: usize = 200_000;
    pub const MAX_SKILL_OUTPUT_PER_STREAM: usize = 1024 * 1024; // 1MB per step stream
    pub const MCP_SERVER_RETRY_COUNT: u32 = 5;
}

pub mod security {
    pub const REGEX_DFA_SIZE_LIMIT: usize = 10_000;
}

pub mod skills {
    pub const SKILL_PARAM_PREFIX: &str = "SKILL_PARAM_";
    pub const SKILL_SECRET_PREFIX: &str = "SKILL_SECRET_";
    pub const RULEWEAVER_SKILL_ID: &str = "RULEWEAVER_SKILL_ID";
    pub const RULEWEAVER_SKILL_NAME: &str = "RULEWEAVER_SKILL_NAME";
    pub const RULEWEAVER_SKILL_DIR: &str = "RULEWEAVER_SKILL_DIR";
}

pub const DEFAULT_MCP_PORT: u16 = 8080;

pub const SKILLS_DIR_NAME: &str = "skills";
pub const SKILL_METADATA_FILE: &str = "skill.json";
pub const SKILL_INSTRUCTIONS_FILE: &str = "SKILL.md";

pub const ANTIGRAVITY_FILENAME: &str = "GEMINI.md";
pub const GEMINI_FILENAME: &str = "GEMINI.md";
pub const OPENCODE_FILENAME: &str = "AGENTS.md";
pub const CLINE_FILENAME: &str = ".clinerules";
pub const CLAUDE_CODE_FILENAME: &str = "CLAUDE.md";
pub const CODEX_FILENAME: &str = "AGENTS.md";
pub const KILO_FILENAME: &str = "AGENTS.md";
pub const CURSOR_FILENAME: &str = "COMMANDS.md";
pub const WINDSURF_FILENAME: &str = "AGENTS.md";
pub const ROO_CODE_FILENAME: &str = "AGENTS.md";

pub const LEGACY_ANTIGRAVITY_DIR: &str = ".antigravity";
pub const LEGACY_OPENCODE_DIR: &str = ".opencode";

pub const NEW_GEMINI_DIR: &str = ".gemini";
pub const NEW_OPENCODE_DIR: &str = ".config/opencode";
pub const NEW_KILO_DIR: &str = ".kilocode";
pub const NEW_CURSOR_DIR: &str = ".cursor";
pub const NEW_WINDSURF_DIR: &str = ".windsurf";
pub const NEW_ROO_CODE_DIR: &str = ".roo";

// Slash command directories (8 supported tools)
pub const OPENCODE_COMMANDS_DIR: &str = ".config/opencode/commands";
pub const CLAUDE_COMMANDS_DIR: &str = ".claude/commands";
pub const CLINE_WORKFLOWS_DIR: &str = ".clinerules/workflows";
pub const GEMINI_COMMANDS_DIR: &str = ".gemini/commands";
pub const CURSOR_COMMANDS_DIR: &str = ".cursor/commands";
pub const ROO_COMMANDS_DIR: &str = ".roo/commands";
pub const ANTIGRAVITY_WORKFLOWS_DIR: &str = ".gemini/antigravity/global_workflows";
pub const CODEX_SKILLS_DIR: &str = ".agents/skills";
