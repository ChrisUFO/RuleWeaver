use std::time::Duration;
use tokio::process::Command as TokioCommand;
use tokio::time::timeout;

use crate::error::{AppError, Result};

const MAX_ARG_LENGTH: usize = 2000;
const MAX_SCRIPT_LENGTH: usize = 20000;

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

    let dangerous_tokens = [
        ";", "&&", "&", "||", "|", "`", "$(", ")", "<", ">", "<<", "<&", ">&", "eval", "exec",
    ];
    let lower = value.to_lowercase();
    for token in dangerous_tokens {
        if lower.contains(token) {
            return Err(AppError::InvalidInput {
                message: format!("Argument contains forbidden token: {}", token),
            });
        }
    }

    Ok(value.to_string())
}

pub fn contains_disallowed_pattern(script: &str) -> Option<&'static str> {
    let lower = script.to_lowercase();
    let patterns: [(&str, &str); 15] = [
        ("rm -rf", "rm -rf"),
        ("del /f", "del /f"),
        ("format ", "format"),
        ("mkfs", "mkfs"),
        ("shutdown", "shutdown"),
        ("reboot", "reboot"),
        ("curl |", "curl pipe"),
        ("wget |", "wget pipe"),
        ("`", "backticks"),
        ("$(", "command substitution"),
        ("eval ", "eval"),
        ("exec ", "exec"),
        ("<(", "process substitution"),
        (">(", "process substitution"),
        ("<<", "heredoc"),
    ];

    for (needle, name) in patterns {
        if lower.contains(needle) {
            return Some(name);
        }
    }

    None
}

pub async fn execute_shell_with_timeout(
    script: &str,
    timeout_dur: Duration,
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
