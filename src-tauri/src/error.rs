use serde::Serialize;
use std::sync::PoisonError;
use thiserror::Error;

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Rule not found: {id}")]
    RuleNotFound { id: String },

    #[error("Sync conflict detected in: {file_path}")]
    SyncConflict { file_path: String },

    #[error("Invalid input: {message}")]
    InvalidInput { message: String },

    #[error("Failed to serialize data: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Path error: {0}")]
    Path(String),

    #[error("Database lock poisoned")]
    DatabasePoisoned,
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
