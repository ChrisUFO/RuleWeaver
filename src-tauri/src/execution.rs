use std::time::Duration;
use tokio::process::Command as TokioCommand;
use tokio::time::timeout;

use crate::constants::{
    limits::{MAX_ARG_LENGTH, MAX_SCRIPT_LENGTH},
    security::REGEX_DFA_SIZE_LIMIT,
};
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
    static RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let re = RE.get_or_init(|| regex::Regex::new(r"[a-z0-9]+").expect("Invalid slugify regex"));

    let lower = input.to_lowercase();
    let parts: Vec<&str> = re.find_iter(&lower).map(|m| m.as_str()).collect();
    parts.join("-")
}

pub fn validate_enum_argument(
    arg_name: &str,
    value: &str,
    options: &Option<Vec<String>>,
) -> Result<()> {
    if let Some(opts) = options {
        if !opts.contains(&value.to_string()) {
            return Err(AppError::InvalidInput {
                message: format!(
                    "Argument '{}' must be one of: {}",
                    arg_name,
                    opts.join(", ")
                ),
            });
        }
    }
    Ok(())
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
        regex::RegexBuilder::new(
            r"(?i);|&&|&|\|\||\||`|\$\s*\(|\$\s*\{|\)|<|>|<<|<&|>&|\beval\b|\bexec\b",
        )
        .size_limit(100_000)
        .dfa_size_limit(REGEX_DFA_SIZE_LIMIT)
        .build()
        .expect("Invalid dangerous tokens regex")
    });

    if let Some(m) = re.find(value) {
        let matched = m.as_str();
        let category = match matched {
            ";" | "&&" | "&" | "||" | "|" => "command chaining",
            "`" | "$(" | "${" | ")" => "command substitution",
            "<" | ">" | "<<" | "<&" | ">&" => "I/O redirection",
            "eval" | "exec" => "dynamic execution",
            _ => "suspicious pattern",
        };
        return Err(AppError::InvalidInput {
            message: format!(
                "Argument contains forbidden {} token: {}",
                category, matched
            ),
        });
    }

    Ok(value.to_string())
}

pub fn contains_disallowed_pattern(script: &str) -> Option<String> {
    let lower = script.to_lowercase();
    let patterns: [(&str, &str, &str); 18] = [
        ("rm -rf", "destructive", "rm -rf"),
        ("del /f", "destructive", "del /f"),
        ("format ", "destructive", "format"),
        ("mkfs", "destructive", "mkfs"),
        ("shutdown", "system control", "shutdown"),
        ("reboot", "system control", "reboot"),
        ("curl |", "network pipe", "curl pipe"),
        ("wget |", "network pipe", "wget pipe"),
        ("base64 -d", "encoding", "base64 decode"),
        ("base64 --decode", "encoding", "base64 decode"),
        ("| sh", "shell pipe", "pipe to shell"),
        ("| bash", "shell pipe", "pipe to shell"),
        ("| zsh", "shell pipe", "pipe to shell"),
        ("`", "substitution", "backticks"),
        ("$(", "substitution", "command substitution"),
        ("eval ", "dynamic execution", "eval"),
        ("exec ", "dynamic execution", "exec"),
        ("<<", "I/O", "heredoc"),
    ];

    for (needle, category, name) in patterns {
        if lower.contains(needle) {
            return Some(format!("matched {} pattern: {}", category, name));
        }
    }

    None
}

pub async fn execute_shell_with_timeout_env(
    script: &str,
    timeout_dur: Duration,
    envs: &[(String, String)],
) -> Result<(i32, String, String)> {
    execute_shell_with_timeout_env_dir(script, timeout_dur, envs, None).await
}

pub async fn execute_shell_with_timeout_env_dir(
    script: &str,
    timeout_dur: Duration,
    envs: &[(String, String)],
    dir: Option<std::path::PathBuf>,
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

    // Validate that the current working directory is accessible
    if let Err(e) = std::env::current_dir() {
        return Err(AppError::Io(e));
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

    if let Some(d) = dir {
        cmd.current_dir(d);
    }

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

pub async fn execute_and_log(input: ExecuteAndLogInput<'_>) -> Result<(i32, String, String, u64)> {
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
