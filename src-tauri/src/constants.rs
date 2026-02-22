use std::time::Duration;

pub const CMD_EXEC_TIMEOUT: Duration = Duration::from_secs(60);
pub const SKILL_EXEC_TIMEOUT: Duration = Duration::from_secs(60);
pub const TEST_CMD_TIMEOUT: Duration = Duration::from_secs(30);

pub const MAX_SKILL_STEPS: usize = 10;
pub const MAX_STEP_LENGTH: usize = 4000;
pub const MAX_ARG_LENGTH: usize = 2000;
pub const MAX_SCRIPT_LENGTH: usize = 20000;
pub const MAX_OUTPUT_SIZE: usize = 10 * 1024 * 1024; // 10MB

pub const REGEX_DFA_SIZE_LIMIT: usize = 10_000;

pub const MCP_RATE_LIMIT_WINDOW: Duration = Duration::from_secs(10);
pub const MCP_RATE_LIMIT_MAX_CALLS: usize = 10;

pub const LOG_LIMIT: usize = 500;

pub const DEFAULT_MCP_PORT: u16 = 8080;

pub const MCP_SERVER_RETRY_COUNT: u32 = 5;
pub const MCP_SERVER_BACKOFF_INITIAL_MS: u64 = 100;

pub const MAX_RULE_NAME_LENGTH: usize = 200;
pub const MAX_RULE_CONTENT_LENGTH: usize = 1_000_000;
pub const MAX_COMMAND_NAME_LENGTH: usize = 120;
pub const MAX_COMMAND_SCRIPT_LENGTH: usize = 10_000;
pub const MAX_SKILL_NAME_LENGTH: usize = 160;
pub const MAX_SKILL_INSTRUCTIONS_LENGTH: usize = 200_000;
