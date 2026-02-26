use std::time::Duration;
use tokio::process::Command as TokioCommand;
use tokio::time::timeout;

use crate::constants::limits::{MAX_ARG_LENGTH, MAX_SCRIPT_LENGTH};
use crate::database::{Database, ExecutionLogInput};
use crate::error::{AppError, Result};
use crate::models::FailureClass;
use crate::redaction::redact;

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
    if value.contains('\n') || value.contains('\r') || value.contains('\t') {
        return Err(AppError::InvalidInput {
            message: "Argument contains forbidden control characters".to_string(),
        });
    }

    let sanitized = {
        #[cfg(target_os = "windows")]
        {
            escape_cmd_argument(value)
        }

        #[cfg(not(target_os = "windows"))]
        {
            let mut escaped = String::with_capacity(value.len() + 2);
            escaped.push('\'');
            escaped.push_str(&value.replace('\'', "'\\''"));
            escaped.push('\'');
            escaped
        }
    };

    if sanitized.len() > MAX_ARG_LENGTH {
        return Err(AppError::InvalidInput {
            message: format!("Argument too long (max {} chars)", MAX_ARG_LENGTH),
        });
    }

    Ok(sanitized)
}

#[cfg(target_os = "windows")]
fn escape_cmd_argument(value: &str) -> String {
    if !value.chars().any(|c| matches!(c, '^' | '&' | '<' | '>' | '|' | '(' | ')' | '%' | '!' | '"')) {
        return value.to_string();
    }

    let mut escaped = String::with_capacity(value.len() + 16);
    for c in value.chars() {
        match c {
            '^' | '&' | '<' | '>' | '|' | '(' | ')' | '%' | '!' | '"' => {
                escaped.push('^');
                escaped.push(c);
            }
            _ => escaped.push(c),
        }
    }
    escaped
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
        ("| bash", "shell pipe", "pipe to bash"),
        ("| zsh", "shell pipe", "pipe to zsh"),
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

pub fn classify_failure(exit_code: i32, stderr: &str, is_timeout: bool) -> FailureClass {
    if is_timeout {
        return FailureClass::Timeout;
    }

    if exit_code == 0 {
        return FailureClass::Success;
    }

    let stderr_lower = stderr.to_lowercase();

    if stderr_lower.contains("permission denied")
        || stderr_lower.contains("access is denied")
        || stderr_lower.contains("access denied")
        || stderr_lower.contains("eacces")
    {
        return FailureClass::PermissionDenied;
    }

    if stderr_lower.contains("command not found")
        || stderr_lower.contains("is not recognized")
        || stderr_lower.contains("' is not recognized")
        || stderr_lower.contains("no such file or directory")
        || stderr_lower.contains("enoent")
    {
        return FailureClass::MissingBinary;
    }

    if stderr_lower.contains("invalid argument")
        || stderr_lower.contains("invalid option")
        || stderr_lower.contains("validation")
        || stderr_lower.contains("invalid input")
    {
        return FailureClass::ValidationError;
    }

    FailureClass::NonZeroExit
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
    pub max_retries: Option<u8>,
    pub adapter_context: Option<&'a str>,
}

pub async fn execute_and_log(input: ExecuteAndLogInput<'_>) -> Result<(i32, String, String, u64)> {
    let max_attempts = input.max_retries.map(|r| (r as u32) + 1).unwrap_or(1).min(4);
    
    let mut last_exit_code: i32 = 0;
    let mut last_stdout = String::new();
    let mut last_stderr = String::new();
    let mut last_duration_ms: u64 = 0;

    for attempt in 1..=max_attempts {
        let attempt_start = std::time::Instant::now();
        
        match execute_shell_with_timeout_env(input.script, input.timeout_dur, input.envs).await {
            Ok((exit_code, stdout, stderr)) => {
                let (stdout_redacted, stdout_was_redacted) = redact(&stdout);
                let (stderr_redacted, stderr_was_redacted) = redact(&stderr);
                let is_redacted = stdout_was_redacted || stderr_was_redacted;
                let is_timeout = false;
                let failure_class = classify_failure(exit_code, &stderr_redacted, is_timeout);
                let duration_ms = attempt_start.elapsed().as_millis() as u64;

                last_exit_code = exit_code;
                last_stdout = stdout_redacted.clone();
                last_stderr = stderr_redacted.clone();
                last_duration_ms = duration_ms;

                if let Some(db) = input.db {
                    let _ = db
                        .add_execution_log(&ExecutionLogInput {
                            command_id: input.command_id,
                            command_name: input.command_name,
                            arguments_json: input.arguments_json,
                            stdout: &stdout_redacted,
                            stderr: &stderr_redacted,
                            exit_code,
                            duration_ms,
                            triggered_by: input.triggered_by,
                            failure_class: Some(failure_class.as_str()),
                            adapter_context: input.adapter_context,
                            is_redacted,
                            attempt_number: attempt as u8,
                        })
                        .await;
                }

                if exit_code == 0 {
                    return Ok((exit_code, stdout_redacted, stderr_redacted, duration_ms));
                }

                let should_retry = attempt < max_attempts && failure_class.is_retryable();
                if !should_retry {
                    return Ok((exit_code, stdout_redacted, stderr_redacted, duration_ms));
                }
            }
            Err(AppError::InvalidInput { message }) if message.contains("timed out") => {
                let failure_class = FailureClass::Timeout;
                let duration_ms = input.timeout_dur.as_millis() as u64;

                last_exit_code = -1;
                last_stdout = String::new();
                last_stderr = message.clone();
                last_duration_ms = duration_ms;

                if let Some(db) = input.db {
                    let _ = db
                        .add_execution_log(&ExecutionLogInput {
                            command_id: input.command_id,
                            command_name: input.command_name,
                            arguments_json: input.arguments_json,
                            stdout: "",
                            stderr: &message,
                            exit_code: -1,
                            duration_ms,
                            triggered_by: input.triggered_by,
                            failure_class: Some(failure_class.as_str()),
                            adapter_context: input.adapter_context,
                            is_redacted: false,
                            attempt_number: attempt as u8,
                        })
                        .await;
                }

                let should_retry = attempt < max_attempts && failure_class.is_retryable();
                if !should_retry {
                    return Err(AppError::InvalidInput { message });
                }
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    Ok((last_exit_code, last_stdout, last_stderr, last_duration_ms))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "windows")]
    #[test]
    fn test_escape_cmd_argument() {
        assert_eq!(escape_cmd_argument("normal"), "normal");
        assert_eq!(escape_cmd_argument("foo & bar"), "foo ^& bar");
        assert_eq!(escape_cmd_argument("foo | bar"), "foo ^| bar");
        assert_eq!(escape_cmd_argument("echo (1)"), "echo ^(1^)");
        assert_eq!(escape_cmd_argument("%path%"), "^%path^%");
        assert_eq!(escape_cmd_argument("\"quoted\""), "^\"quoted^\"");
    }

    #[test]
    fn test_sanitize_argument_value() {
        let input = "foo & bar";
        #[cfg(target_os = "windows")]
        assert_eq!(sanitize_argument_value(input).unwrap(), "foo ^& bar");
        #[cfg(not(target_os = "windows"))]
        assert_eq!(sanitize_argument_value(input).unwrap(), "'foo & bar'");
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_sanitize_argument_value_unix_injection() {
        let input = "foo'; rm -rf /; echo 'bar";
        assert_eq!(sanitize_argument_value(input).unwrap(), "'foo'\\'; rm -rf /; echo 'bar'");
    }

    #[test]
    fn test_classify_failure_timeout() {
        assert_eq!(classify_failure(0, "", true), FailureClass::Timeout);
    }

    #[test]
    fn test_classify_failure_success() {
        assert_eq!(classify_failure(0, "", false), FailureClass::Success);
    }

    #[test]
    fn test_classify_failure_permission_denied() {
        assert_eq!(classify_failure(1, "permission denied", false), FailureClass::PermissionDenied);
    }

    #[test]
    fn test_classify_failure_missing_binary() {
        assert_eq!(classify_failure(1, "command not found", false), FailureClass::MissingBinary);
    }

    #[test]
    fn test_classify_failure_validation_error() {
        assert_eq!(classify_failure(1, "invalid argument", false), FailureClass::ValidationError);
    }

    #[test]
    fn test_classify_failure_non_zero_exit() {
        assert_eq!(classify_failure(1, "some other error", false), FailureClass::NonZeroExit);
    }

    #[test]
    fn test_failure_class_is_retryable() {
        assert!(FailureClass::Timeout.is_retryable());
        assert!(FailureClass::NonZeroExit.is_retryable());
        assert!(FailureClass::PermissionDenied.is_retryable());
        assert!(!FailureClass::ValidationError.is_retryable());
        assert!(!FailureClass::MissingBinary.is_retryable());
    }
}
