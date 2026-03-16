use reqwest::Client;
use serde_json::Value;

use crate::ai::error::{AiClientError, AiResult};
use crate::ai::models::{AnthropicRequest, AnthropicResponse, ChatCompletionResponse};
use crate::models::ModelInfo;

const REQUEST_TIMEOUT_SECS: u64 = 120;
const ANTHROPIC_VERSION: &str = "2023-06-01";

pub struct AnthropicClient {
    client: Client,
    api_key: String,
    model: String,
}

impl AnthropicClient {
    pub fn new(api_key: &str, model: &str) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client,
            api_key: api_key.to_string(),
            model: model.to_string(),
        }
    }

    pub async fn complete(&self, request: AnthropicRequest) -> AiResult<ChatCompletionResponse> {
        let url = "https://api.anthropic.com/v1/messages";

        let response = self
            .client
            .post(url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    AiClientError::Timeout
                } else if e.is_connect() {
                    AiClientError::NetworkError("Failed to connect to Anthropic API".to_string())
                } else {
                    AiClientError::RequestFailed(e.to_string())
                }
            })?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());

        if !status.is_success() {
            return Err(self.parse_error(&response_text, status.as_u16()));
        }

        let anthropic_response: AnthropicResponse =
            serde_json::from_str(&response_text).map_err(|e| {
                AiClientError::InvalidResponse(format!("Failed to parse Anthropic response: {}", e))
            })?;

        Ok(ChatCompletionResponse::from(anthropic_response))
    }

    pub async fn test_connection(&self) -> AiResult<bool> {
        let request =
            AnthropicRequest::new(&self.model, "You are a helpful assistant.", "Say 'test'");

        let result = self.complete(request).await?;
        Ok(!result.choices.is_empty())
    }

    pub fn list_available_models() -> Vec<ModelInfo> {
        vec![
            ModelInfo {
                id: "claude-opus-4-20250514".to_string(),
                name: Some("Claude Opus 4".to_string()),
                context_length: Some(200000),
            },
            ModelInfo {
                id: "claude-sonnet-4-20250514".to_string(),
                name: Some("Claude Sonnet 4".to_string()),
                context_length: Some(200000),
            },
            ModelInfo {
                id: "claude-3-7-sonnet-20250219".to_string(),
                name: Some("Claude 3.7 Sonnet".to_string()),
                context_length: Some(200000),
            },
            ModelInfo {
                id: "claude-3-5-sonnet-20241022".to_string(),
                name: Some("Claude 3.5 Sonnet".to_string()),
                context_length: Some(200000),
            },
            ModelInfo {
                id: "claude-3-5-haiku-20241022".to_string(),
                name: Some("Claude 3.5 Haiku".to_string()),
                context_length: Some(200000),
            },
            ModelInfo {
                id: "claude-3-opus-20240229".to_string(),
                name: Some("Claude 3 Opus".to_string()),
                context_length: Some(200000),
            },
        ]
    }

    fn parse_error(&self, response_text: &str, status: u16) -> AiClientError {
        if let Ok(error_response) = serde_json::from_str::<Value>(response_text) {
            if let Some(error) = error_response.get("error") {
                let error_type = error.get("type").and_then(|t| t.as_str()).unwrap_or("");
                let message = error.get("message").and_then(|m| m.as_str()).unwrap_or("");

                match error_type {
                    "authentication_error" | "invalid_request_error"
                        if message.contains("API key") =>
                    {
                        return AiClientError::InvalidApiKey;
                    }
                    "rate_limit_error" => {
                        return AiClientError::RateLimited(message.to_string());
                    }
                    "invalid_request_error" if message.contains("model") => {
                        return AiClientError::ModelNotAvailable(self.model.clone());
                    }
                    "invalid_request_error"
                        if message.contains("context") || message.contains("token") =>
                    {
                        return AiClientError::ContextTooLong(0);
                    }
                    _ => {}
                }

                if !message.is_empty() {
                    return AiClientError::RequestFailed(message.to_string());
                }
            }
        }

        match status {
            401 => AiClientError::InvalidApiKey,
            429 => AiClientError::RateLimited("Rate limit exceeded".to_string()),
            _ => AiClientError::RequestFailed(format!("HTTP {} error: {}", status, response_text)),
        }
    }
}
