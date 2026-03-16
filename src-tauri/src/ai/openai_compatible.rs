use reqwest::Client;
use serde_json::Value;

use crate::ai::error::{AiClientError, AiResult};
use crate::ai::models::{ChatCompletionRequest, ChatCompletionResponse, OpenAiModelsResponse};
use crate::models::{AiProvider, ModelInfo};

const REQUEST_TIMEOUT_SECS: u64 = 120;

pub struct OpenAiCompatibleClient {
    client: Client,
    base_url: String,
    api_key: String,
    model: String,
}

impl OpenAiCompatibleClient {
    pub fn new(provider: AiProvider, base_url: &str, api_key: &str, model: &str) -> Self {
        let base_url = if base_url.is_empty() {
            provider.default_base_url().to_string()
        } else {
            base_url.trim_end_matches('/').to_string()
        };

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client,
            base_url,
            api_key: api_key.to_string(),
            model: model.to_string(),
        }
    }

    pub async fn complete(
        &self,
        request: ChatCompletionRequest,
    ) -> AiResult<ChatCompletionResponse> {
        let url = format!("{}/chat/completions", self.base_url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    AiClientError::Timeout
                } else if e.is_connect() {
                    AiClientError::NetworkError("Failed to connect to API".to_string())
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

        serde_json::from_str(&response_text)
            .map_err(|e| AiClientError::InvalidResponse(format!("Failed to parse response: {}", e)))
    }

    pub async fn list_models(&self) -> AiResult<Vec<ModelInfo>> {
        let url = format!("{}/models", self.base_url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    AiClientError::Timeout
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

        let models_response: OpenAiModelsResponse =
            serde_json::from_str(&response_text).map_err(|e| {
                AiClientError::InvalidResponse(format!("Failed to parse models response: {}", e))
            })?;

        let models: Vec<ModelInfo> = models_response
            .data
            .into_iter()
            .filter(|m| {
                !m.id.contains(":") && !m.id.starts_with("whisper") && !m.id.starts_with("tts")
            })
            .map(|m| ModelInfo {
                id: m.id,
                name: m.name,
                context_length: m.context_length,
            })
            .collect();

        Ok(models)
    }

    pub async fn test_connection(&self) -> AiResult<bool> {
        let url = format!("{}/models", self.base_url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    AiClientError::Timeout
                } else {
                    AiClientError::RequestFailed(e.to_string())
                }
            })?;

        Ok(response.status().is_success())
    }

    #[allow(dead_code)]
    pub async fn test_model(&self) -> AiResult<bool> {
        let request = ChatCompletionRequest {
            model: self.model.clone(),
            messages: vec![crate::ai::models::ChatMessage {
                role: "user".to_string(),
                content: "Say 'test'".to_string(),
            }],
            max_tokens: Some(10),
            temperature: Some(0.1),
        };

        let result = self.complete(request).await?;
        Ok(!result.choices.is_empty())
    }

    fn parse_error(&self, response_text: &str, status: u16) -> AiClientError {
        if let Ok(error_response) = serde_json::from_str::<Value>(response_text) {
            if let Some(error) = error_response.get("error") {
                let error_type = error.get("type").and_then(|t| t.as_str()).unwrap_or("");
                let message = error.get("message").and_then(|m| m.as_str()).unwrap_or("");

                match error_type {
                    "invalid_api_key" | "authentication_error" => {
                        return AiClientError::InvalidApiKey;
                    }
                    "rate_limit_exceeded" | "rate_limit_error" => {
                        return AiClientError::RateLimited(message.to_string());
                    }
                    "context_length_exceeded" => {
                        return AiClientError::ContextTooLong(0);
                    }
                    "model_not_found" | "invalid_request_error" if message.contains("model") => {
                        return AiClientError::ModelNotAvailable(self.model.clone());
                    }
                    _ => {}
                }

                if !message.is_empty() {
                    return AiClientError::RequestFailed(message.to_string());
                }
            }

            if let Some(message) = error_response.get("message").and_then(|m| m.as_str()) {
                if message.contains("API key") || message.contains("Unauthorized") {
                    return AiClientError::InvalidApiKey;
                }
                if message.contains("rate limit") {
                    return AiClientError::RateLimited(message.to_string());
                }
                return AiClientError::RequestFailed(message.to_string());
            }
        }

        match status {
            401 => AiClientError::InvalidApiKey,
            429 => AiClientError::RateLimited("Rate limit exceeded".to_string()),
            _ => AiClientError::RequestFailed(format!("HTTP {} error: {}", status, response_text)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_client() -> OpenAiCompatibleClient {
        OpenAiCompatibleClient::new(
            AiProvider::OpenAi,
            "https://api.openai.com/v1",
            "test-key",
            "gpt-4",
        )
    }

    #[test]
    fn test_parse_error_invalid_api_key() {
        let client = create_test_client();
        let response = r#"{"error": {"type": "invalid_api_key", "message": "Invalid API key"}}"#;
        let error = client.parse_error(response, 401);
        assert!(matches!(error, AiClientError::InvalidApiKey));
    }

    #[test]
    fn test_parse_error_authentication_error() {
        let client = create_test_client();
        let response = r#"{"error": {"type": "authentication_error", "message": "Unauthorized"}}"#;
        let error = client.parse_error(response, 401);
        assert!(matches!(error, AiClientError::InvalidApiKey));
    }

    #[test]
    fn test_parse_error_rate_limited() {
        let client = create_test_client();
        let response =
            r#"{"error": {"type": "rate_limit_exceeded", "message": "Too many requests"}}"#;
        let error = client.parse_error(response, 429);
        assert!(matches!(error, AiClientError::RateLimited(_)));
    }

    #[test]
    fn test_parse_error_context_too_long() {
        let client = create_test_client();
        let response =
            r#"{"error": {"type": "context_length_exceeded", "message": "Context too long"}}"#;
        let error = client.parse_error(response, 400);
        assert!(matches!(error, AiClientError::ContextTooLong(_)));
    }

    #[test]
    fn test_parse_error_model_not_found() {
        let client = create_test_client();
        let response =
            r#"{"error": {"type": "model_not_found", "message": "The model does not exist"}}"#;
        let error = client.parse_error(response, 404);
        assert!(matches!(error, AiClientError::ModelNotAvailable(_)));
    }

    #[test]
    fn test_parse_error_http_401() {
        let client = create_test_client();
        let response = r#"{}"#;
        let error = client.parse_error(response, 401);
        assert!(matches!(error, AiClientError::InvalidApiKey));
    }

    #[test]
    fn test_parse_error_http_429() {
        let client = create_test_client();
        let response = r#"{}"#;
        let error = client.parse_error(response, 429);
        assert!(matches!(error, AiClientError::RateLimited(_)));
    }

    #[test]
    fn test_parse_error_message_contains_api_key() {
        let client = create_test_client();
        let response = r#"{"message": "Invalid API key provided"}"#;
        let error = client.parse_error(response, 400);
        assert!(matches!(error, AiClientError::InvalidApiKey));
    }

    #[test]
    fn test_parse_error_message_contains_rate_limit() {
        let client = create_test_client();
        let response = r#"{"message": "You exceeded the rate limit"}"#;
        let error = client.parse_error(response, 400);
        assert!(matches!(error, AiClientError::RateLimited(_)));
    }

    #[test]
    fn test_client_new_with_empty_base_url() {
        let client = OpenAiCompatibleClient::new(AiProvider::OpenAi, "", "test-key", "gpt-4");
        assert_eq!(client.base_url, "https://api.openai.com/v1");
    }

    #[test]
    fn test_client_new_with_trailing_slash() {
        let client = OpenAiCompatibleClient::new(
            AiProvider::OpenAiCompatible,
            "https://custom.api.com/v1/",
            "test-key",
            "model",
        );
        assert_eq!(client.base_url, "https://custom.api.com/v1");
    }
}
