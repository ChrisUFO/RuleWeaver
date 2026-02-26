//! Secret redaction for execution logs.
//!
//! This module provides functionality to redact sensitive information from command output
//! before storing in execution logs.

use once_cell::sync::Lazy;
use regex::Regex;

static BEARER_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)bearer\s+[a-zA-Z0-9_.-]+").expect("bearer pattern"));

static API_KEY_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)api[_-]?key\s*[=:]\s*[a-zA-Z0-9_.-]{20,}").expect("api key pattern")
});

static AWS_ACCESS_KEY_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"A(KIA|SIA)[0-9A-Z]{16}").expect("aws access key pattern"));

static AWS_SECRET_KEY_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)aws[_-]?secret[_-]?key\s*[=:]\s*[a-zA-Z0-9/+=]{40}")
        .expect("aws secret key pattern")
});

static PRIVATE_KEY_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"-----BEGIN\s+(?:RSA\s+)?PRIVATE\s+KEY-----[\s\S]*?-----END\s+(?:RSA\s+)?PRIVATE\s+KEY-----").expect("private key pattern")
});

static CONNECTION_STRING_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)password\s*=\s*[^;\s]+").expect("connection string password pattern")
});

static GITHUB_TOKEN_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(ghp_[a-zA-Z0-9]{36})|(gho_[a-zA-Z0-9]{36})|(ghu_[a-zA-Z0-9]{36})|(ghs_[a-zA-Z0-9]{36})|(ghr_[a-zA-Z0-9]{36})").expect("github token pattern")
});

static SLACK_TOKEN_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"xox[baprs]-[0-9]{10,}-[0-9]{10,}-[a-zA-Z0-9]{24}").expect("slack token pattern")
});

static GENERIC_SECRET_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(secret|token|password|passwd|pwd)\s*[=:]\s*[a-zA-Z0-9_.-]{16,}")
        .expect("generic secret pattern")
});

const REDACTED: &str = "[REDACTED]";

pub fn redact(input: &str) -> (String, bool) {
    let mut result = input.to_string();
    let mut was_redacted = false;

    let patterns: &[&Lazy<Regex>] = &[
        &BEARER_PATTERN,
        &API_KEY_PATTERN,
        &AWS_ACCESS_KEY_PATTERN,
        &AWS_SECRET_KEY_PATTERN,
        &PRIVATE_KEY_PATTERN,
        &CONNECTION_STRING_PATTERN,
        &GITHUB_TOKEN_PATTERN,
        &SLACK_TOKEN_PATTERN,
        &GENERIC_SECRET_PATTERN,
    ];

    for pattern in patterns {
        let new_result = pattern.replace_all(&result, REDACTED);
        if new_result != result {
            was_redacted = true;
        }
        result = new_result.to_string();
    }

    (result, was_redacted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redact_bearer_token() {
        let input = "Authorization: Bearer abc123xyz789token";
        let (output, redacted) = redact(input);
        assert!(redacted);
        assert!(output.contains(REDACTED));
        assert!(!output.contains("abc123xyz789token"));
    }

    #[test]
    fn test_redact_api_key() {
        let input = "api_key=sk_test_FAKEKEYFORTESTINGONLYabcdefghijklmnop";
        let (output, redacted) = redact(input);
        assert!(redacted);
        assert!(output.contains(REDACTED));
        assert!(!output.contains("sk_test_FAKEKEYFORTESTINGONLYabcdefghijklmnop"));
    }

    #[test]
    fn test_redact_aws_access_key() {
        let input = "AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE";
        let (output, redacted) = redact(input);
        assert!(redacted);
        assert!(!output.contains("AKIAIOSFODNN7EXAMPLE"));
    }

    #[test]
    fn test_redact_private_key() {
        let input = "-----BEGIN PRIVATE KEY-----
MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQC7VJTUt9Us8cKj
-----END PRIVATE KEY-----";
        let (output, redacted) = redact(input);
        assert!(redacted);
        assert!(!output.contains("MIIEvgIBADANBgkqhkiG9w0BAQEFA"));
    }

    #[test]
    fn test_redact_github_token() {
        let input = "GITHUB_TOKEN=ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx";
        let (output, redacted) = redact(input);
        assert!(redacted);
        assert!(!output.contains("ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"));
    }

    #[test]
    fn test_redact_slack_token() {
        let input = "SLACK_TOKEN=xoxb-FAKETOKEN123456-FAKETOKEN1234567-FakeForTestingOnly";
        let (output, redacted) = redact(input);
        assert!(redacted);
        assert!(!output.contains("xoxb-FAKETOKEN123456-FAKETOKEN1234567-FakeForTestingOnly"));
    }

    #[test]
    fn test_redact_connection_string_password() {
        let input = "Server=myserver;Database=mydb;Password=supersecret123;";
        let (output, redacted) = redact(input);
        assert!(redacted);
        assert!(output.contains(REDACTED));
        assert!(!output.contains("supersecret123"));
    }

    #[test]
    fn test_no_redaction_for_normal_content() {
        let input = "The quick brown fox jumps over the lazy dog";
        let (output, redacted) = redact(input);
        assert!(!redacted);
        assert_eq!(output, input);
    }

    #[test]
    fn test_no_redaction_for_short_values() {
        let input = "api_key=abc";
        let (output, redacted) = redact(input);
        assert!(!redacted);
        assert_eq!(output, input);
    }

    #[test]
    fn test_redact_multiple_secrets() {
        let input = "Bearer token123 and api_key=sk_live_abcdefghijklmnop123456";
        let (output, redacted) = redact(input);
        assert!(redacted);
        assert!(output.contains(REDACTED));
        assert!(!output.contains("token123"));
        assert!(!output.contains("sk_live_abcdefghijklmnop123456"));
    }
}
