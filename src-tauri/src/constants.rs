use std::time::Duration;

pub mod timing {
    use super::*;
    pub const TEST_CMD_TIMEOUT: Duration = Duration::from_secs(30);
    pub const TEST_CMD_RATE_LIMIT_WINDOW: Duration = Duration::from_secs(60);
}

pub mod limits {
    pub const MAX_ARG_LENGTH: usize = 2000;
    pub const MAX_SCRIPT_LENGTH: usize = 20000;
    pub const TEST_CMD_RATE_LIMIT_MAX: usize = 5;
    pub const MAX_RULE_NAME_LENGTH: usize = 200;
    pub const MAX_RULE_CONTENT_LENGTH: usize = 1_000_000;
    pub const MAX_COMMAND_NAME_LENGTH: usize = 120;
    pub const MAX_COMMAND_SCRIPT_LENGTH: usize = 10_000;
    pub const MAX_SKILL_NAME_LENGTH: usize = 160;
    pub const MAX_SKILL_INSTRUCTIONS_LENGTH: usize = 200_000;
}

pub mod skills {
    pub const SKILL_PARAM_PREFIX: &str = "SKILL_PARAM_";
}

pub const SKILLS_DIR_NAME: &str = "skills";
pub const SKILL_METADATA_FILE: &str = "skill.json";
pub const SKILL_INSTRUCTIONS_FILE: &str = "SKILL.md";

pub const ANTIGRAVITY_FILENAME: &str = "GEMINI.md";
pub const GEMINI_FILENAME: &str = "GEMINI.md";
pub const OPENCODE_FILENAME: &str = "AGENTS.md";

pub const LEGACY_ANTIGRAVITY_DIR: &str = ".antigravity";
pub const LEGACY_OPENCODE_DIR: &str = ".opencode";

pub const NEW_GEMINI_DIR: &str = ".gemini";
pub const NEW_OPENCODE_DIR: &str = ".config/opencode";
pub const NEW_KILO_DIR: &str = ".kilocode";
pub const NEW_CURSOR_DIR: &str = ".cursor";
pub const NEW_WINDSURF_DIR: &str = ".windsurf";
pub const NEW_ROO_CODE_DIR: &str = ".roo";
