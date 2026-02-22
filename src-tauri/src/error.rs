use serde::Serialize;
use std::sync::PoisonError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Rule not found: {id}")]
    RuleNotFound { id: String },

    #[error("Command not found: {id}")]
    CommandNotFound { id: String },

    #[error("Skill not found: {id}")]
    SkillNotFound { id: String },

    #[error("Sync conflict detected in: {file_path}")]
    #[allow(dead_code)]
    SyncConflict { file_path: String },

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Unauthorized: {0}")]
    Auth(String),

    #[error("MCP server error: {0}")]
    Mcp(String),

    #[error("Invalid input: {message}")]
    InvalidInput { message: String },

    #[error("Failed to serialize data: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Path error: {0}")]
    Path(String),

    #[error("Database internal state error (poisoned lock). Please restart the application.")]
    DatabasePoisoned,

    #[error("Lock error")]
    LockError,

    #[error("YAML parsing error: {message}")]
    #[allow(dead_code)]
    Yaml { message: String },

    #[error("Migration error: {message}")]
    #[allow(dead_code)]
    Migration { message: String },

    #[error("File watcher error: {message}")]
    #[allow(dead_code)]
    Watcher { message: String },
}

impl<T> From<PoisonError<T>> for AppError {
    fn from(_: PoisonError<T>) -> Self {
        AppError::DatabasePoisoned
    }
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
