use std::time::Duration;
use tokio::process::Command as TokioCommand;
use tokio::time::timeout;

use crate::constants::{MAX_ARG_LENGTH, MAX_SCRIPT_LENGTH, REGEX_DFA_SIZE_LIMIT};
use crate::database::{Database, ExecutionLogInput};
use crate::error::{AppError, Result};

pub fn template_token(arg_name: &str) -> String {
    format!("{{{{{}}}}}", arg_name)
}

pub fn argument_env_var_name(arg_name: &str) -> String {
    let mut slug = slugify(arg_name);
    if slug.is_empty() {
        slug = "arg".to_string();
    }
    format!("RW_ARG_{}", slug.replace('-', "_").to_uppercase())
}

pub fn replace_template_with_env_ref(script: &str, arg_name: &str) -> String {
    let token = template_token(arg_name);
    let env_name = argument_env_var_name(arg_name);

    #[cfg(target_os = "windows")]
    let reference = format!("%{}%", env_name);

    #[cfg(not(target_os = "windows"))]
    let reference = format!("${}", env_name);

    script.replace(&token, &reference)
}

pub fn slugify(input: &str) -> String {
    let mut out = String::new();
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if ch.is_whitespace() || ch == '-' || ch == '_' {
            out.push('-');
        }
    }
    while out.contains("--") {
        out = out.replace("--", "-");
    }
    out.trim_matches('-').to_string()
}

pub fn sanitize_argument_value(value: &str) -> Result<String> {
    if value.len() > MAX_ARG_LENGTH {
        return Err(AppError::InvalidInput {
            message: format!("Argument too long (max {} chars)", MAX_ARG_LENGTH),
        });
    }

    if value.contains('\n') || value.contains('\r') || value.contains('\t') {
        return Err(AppError::InvalidInput {
            message: "Argument contains forbidden control characters".to_string(),
        });
    }

    // Use regex to catch dangerous tokens even with internal whitespace (e.g. $( pwd ))
    // We use \b for eval and exec to avoid catching words like "evaluation"
    static RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let re = RE.get_or_init(|| {
        regex::RegexBuilder::new(r"(?i);|&&|&|\|\||\||`|\$\s*\(|\$\s*\{|\)|<|>|<<|<&|>&|\beval\b|\bexec\b")
            .size_limit(100_000)
            .dfa_size_limit(REGEX_DFA_SIZE_LIMIT)
            .build()
            .expect("Invalid dangerous tokens regex")
    });

    if let Some(m) = re.find(value) {
        return Err(AppError::InvalidInput {
            message: format!("Argument contains forbidden token: {}", m.as_str()),
        });
    }

    Ok(value.to_string())
}

pub fn contains_disallowed_pattern(script: &str) -> Option<&'static str> {
    let lower = script.to_lowercase();
    let patterns: [(&str, &str); 18] = [
        ("rm -rf", "rm -rf"),
        ("del /f", "del /f"),
        ("format ", "format"),
        ("mkfs", "mkfs"),
        ("shutdown", "shutdown"),
        ("reboot", "reboot"),
        ("curl |", "curl pipe"),
        ("wget |", "wget pipe"),
        ("base64 -d", "base64 decode"),
        ("base64 --decode", "base64 decode"),
        ("| sh", "pipe to shell"),
        ("| bash", "pipe to shell"),
        ("| zsh", "pipe to shell"),
        ("`", "backticks"),
        ("$(", "command substitution"),
        ("eval ", "eval"),
        ("exec ", "exec"),
        ("<<", "heredoc"),
    ];

    for (needle, name) in patterns {
        if lower.contains(needle) {
            return Some(name);
        }
    }

    None
}

pub async fn execute_shell_with_timeout_env(
    script: &str,
    timeout_dur: Duration,
    envs: &[(String, String)],
) -> Result<(i32, String, String)> {
    if script.trim().is_empty() {
        return Err(AppError::InvalidInput {
            message: "Cannot execute empty script".to_string(),
        });
    }

    if script.len() > MAX_SCRIPT_LENGTH {
        return Err(AppError::InvalidInput {
            message: format!("Script too long (max {} chars)", MAX_SCRIPT_LENGTH),
        });
    }

    #[cfg(target_os = "windows")]
    let mut cmd = TokioCommand::new("cmd");
    #[cfg(target_os = "windows")]
    cmd.args(["/C", script]);

    #[cfg(not(target_os = "windows"))]
    let mut cmd = TokioCommand::new("sh");
    #[cfg(not(target_os = "windows"))]
    cmd.args(["-c", script]);

    cmd.envs(envs.iter().cloned());

    let future = cmd.output();

    match timeout(timeout_dur, future).await {
        Ok(Ok(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Ok((output.status.code().unwrap_or(-1), stdout, stderr))
        }
        Ok(Err(e)) => Err(AppError::Io(e)),
        Err(_) => Err(AppError::InvalidInput {
            message: format!("Execution timed out after {}s", timeout_dur.as_secs()),
        }),
    }
}

pub struct ExecuteAndLogInput<'a> {
    pub db: Option<&'a Database>,
    pub command_id: &'a str,
    pub command_name: &'a str,
    pub script: &'a str,
    pub timeout_dur: Duration,
    pub envs: &'a [(String, String)],
    pub arguments_json: &'a str,
    pub triggered_by: &'a str,
}

pub async fn execute_and_log(
    input: ExecuteAndLogInput<'_>,
) -> Result<(i32, String, String, u64)> {
    let start = std::time::Instant::now();
    let result = execute_shell_with_timeout_env(input.script, input.timeout_dur, input.envs).await;
    let duration_ms = start.elapsed().as_millis() as u64;

    match result {
        Ok((exit_code, stdout, stderr)) => {
            if let Some(db) = input.db {
                let _ = db.add_execution_log(&ExecutionLogInput {
                    command_id: input.command_id,
                    command_name: input.command_name,
                    arguments_json: input.arguments_json,
                    stdout: &stdout,
                    stderr: &stderr,
                    exit_code,
                    duration_ms,
                    triggered_by: input.triggered_by,
                });
            }
            Ok((exit_code, stdout, stderr, duration_ms))
        }
        Err(e) => Err(e),
    }
}
