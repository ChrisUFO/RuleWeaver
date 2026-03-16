use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, Serialize)]
pub enum AiClientError {
    #[error("API key not configured")]
    ApiKeyNotSet,

    #[error("Invalid API key")]
    InvalidApiKey,

    #[error("Rate limited: {0}")]
    RateLimited(String),

    #[error("Context too long: {0} tokens")]
    ContextTooLong(u32),

    #[error("Model not available: {0}")]
    ModelNotAvailable(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Request timeout")]
    Timeout,

    #[error("Invalid response from API: {0}")]
    InvalidResponse(String),

    #[error("AI feature not enabled")]
    NotEnabled,

    #[error("Base URL is required for custom provider")]
    BaseUrlRequired,

    #[error("Model name is required")]
    ModelRequired,

    #[error("Request failed: {0}")]
    RequestFailed(String),

    #[error("Secure storage error: {0}")]
    SecureStorage(String),
}

pub type AiResult<T> = Result<T, AiClientError>;

impl From<crate::error::AiError> for AiClientError {
    fn from(err: crate::error::AiError) -> Self {
        match err {
            crate::error::AiError::ApiKeyNotSet => AiClientError::ApiKeyNotSet,
            crate::error::AiError::InvalidApiKey => AiClientError::InvalidApiKey,
            crate::error::AiError::RateLimited(msg) => AiClientError::RateLimited(msg),
            crate::error::AiError::ContextTooLong(tokens) => AiClientError::ContextTooLong(tokens),
            crate::error::AiError::ModelNotAvailable(model) => {
                AiClientError::ModelNotAvailable(model)
            }
            crate::error::AiError::NetworkError(msg) => AiClientError::NetworkError(msg),
            crate::error::AiError::Timeout => AiClientError::Timeout,
            crate::error::AiError::InvalidResponse(msg) => AiClientError::InvalidResponse(msg),
            crate::error::AiError::NotEnabled => AiClientError::NotEnabled,
            crate::error::AiError::BaseUrlRequired => AiClientError::BaseUrlRequired,
            crate::error::AiError::ModelRequired => AiClientError::ModelRequired,
            crate::error::AiError::RequestFailed(msg) => AiClientError::RequestFailed(msg),
        }
    }
}

impl From<AiClientError> for crate::error::AppError {
    fn from(err: AiClientError) -> Self {
        crate::error::AppError::Ai(crate::error::AiError::from(err))
    }
}

impl From<AiClientError> for crate::error::AiError {
    fn from(err: AiClientError) -> Self {
        match err {
            AiClientError::ApiKeyNotSet => crate::error::AiError::ApiKeyNotSet,
            AiClientError::InvalidApiKey => crate::error::AiError::InvalidApiKey,
            AiClientError::RateLimited(msg) => crate::error::AiError::RateLimited(msg),
            AiClientError::ContextTooLong(tokens) => crate::error::AiError::ContextTooLong(tokens),
            AiClientError::ModelNotAvailable(model) => {
                crate::error::AiError::ModelNotAvailable(model)
            }
            AiClientError::NetworkError(msg) => crate::error::AiError::NetworkError(msg),
            AiClientError::Timeout => crate::error::AiError::Timeout,
            AiClientError::InvalidResponse(msg) => crate::error::AiError::InvalidResponse(msg),
            AiClientError::NotEnabled => crate::error::AiError::NotEnabled,
            AiClientError::BaseUrlRequired => crate::error::AiError::BaseUrlRequired,
            AiClientError::ModelRequired => crate::error::AiError::ModelRequired,
            AiClientError::RequestFailed(msg) => crate::error::AiError::RequestFailed(msg),
            AiClientError::SecureStorage(msg) => crate::error::AiError::RequestFailed(msg),
        }
    }
}
