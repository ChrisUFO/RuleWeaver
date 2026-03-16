use std::time::Duration;
use tokio::time::sleep;

use crate::ai::error::{AiClientError, AiResult};
use crate::ai::models::{AnthropicRequest, ChatCompletionRequest};
use crate::models::{AiProvider, ModelInfo};

pub mod anthropic;
pub mod error;
pub mod models;
pub mod openai_compatible;

const MAX_RETRIES: u32 = 3;
const INITIAL_RETRY_DELAY_MS: u64 = 1000;
const MAX_RETRY_DELAY_MS: u64 = 10000;

fn is_retryable_error(error: &AiClientError) -> bool {
    matches!(
        error,
        AiClientError::RateLimited(_) | AiClientError::NetworkError(_) | AiClientError::Timeout
    )
}

async fn calculate_retry_delay(attempt: u32) -> Duration {
    let delay = INITIAL_RETRY_DELAY_MS * 2u64.pow(attempt);
    Duration::from_millis(delay.min(MAX_RETRY_DELAY_MS))
}

pub struct AiClient {
    provider: AiProvider,
    base_url: Option<String>,
    api_key: String,
    model: String,
}

impl AiClient {
    pub fn new(provider: AiProvider, base_url: Option<&str>, api_key: &str, model: &str) -> Self {
        Self {
            provider,
            base_url: base_url.map(|s| s.to_string()),
            api_key: api_key.to_string(),
            model: model.to_string(),
        }
    }

    pub async fn complete(&self, system_prompt: &str, user_message: &str) -> AiResult<String> {
        let mut last_error = None;

        for attempt in 0..=MAX_RETRIES {
            let result = if self.provider.uses_native_anthropic_api() {
                self.complete_anthropic(system_prompt, user_message).await
            } else {
                self.complete_openai_compatible(system_prompt, user_message)
                    .await
            };

            match result {
                Ok(response) => return Ok(response),
                Err(err) => {
                    if attempt < MAX_RETRIES && is_retryable_error(&err) {
                        let delay = calculate_retry_delay(attempt).await;
                        sleep(delay).await;
                        last_error = Some(err);
                    } else {
                        return Err(err);
                    }
                }
            }
        }

        Err(last_error.unwrap_or(AiClientError::RequestFailed(
            "Max retries exceeded".to_string(),
        )))
    }

    async fn complete_openai_compatible(
        &self,
        system_prompt: &str,
        user_message: &str,
    ) -> AiResult<String> {
        let base_url = self
            .base_url
            .as_deref()
            .unwrap_or_else(|| self.provider.default_base_url());

        if base_url.is_empty() {
            return Err(AiClientError::BaseUrlRequired);
        }

        let client = openai_compatible::OpenAiCompatibleClient::new(
            self.provider,
            base_url,
            &self.api_key,
            &self.model,
        );

        let request = ChatCompletionRequest::new(&self.model, system_prompt, user_message);
        let response = client.complete(request).await?;

        response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or(AiClientError::InvalidResponse(
                "No response choices returned".to_string(),
            ))
    }

    async fn complete_anthropic(
        &self,
        system_prompt: &str,
        user_message: &str,
    ) -> AiResult<String> {
        let client = anthropic::AnthropicClient::new(&self.api_key, &self.model);
        let request = AnthropicRequest::new_with_system(&self.model, system_prompt, user_message);
        let response = client.complete(request).await?;

        response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or(AiClientError::InvalidResponse(
                "No response choices returned".to_string(),
            ))
    }

    pub async fn list_models(&self) -> AiResult<Vec<ModelInfo>> {
        if !self.provider.supports_model_listing() {
            return Ok(get_static_models_for_provider(self.provider));
        }

        let base_url = self
            .base_url
            .as_deref()
            .unwrap_or_else(|| self.provider.default_base_url());

        if base_url.is_empty() {
            return Ok(get_static_models_for_provider(self.provider));
        }

        let client = openai_compatible::OpenAiCompatibleClient::new(
            self.provider,
            base_url,
            &self.api_key,
            &self.model,
        );

        client.list_models().await
    }

    pub async fn test_connection(&self) -> AiResult<bool> {
        if self.provider.uses_native_anthropic_api() {
            let client = anthropic::AnthropicClient::new(&self.api_key, &self.model);
            client.test_connection().await
        } else {
            let base_url = self
                .base_url
                .as_deref()
                .unwrap_or_else(|| self.provider.default_base_url());
            if base_url.is_empty() {
                return Err(AiClientError::BaseUrlRequired);
            }
            let client = openai_compatible::OpenAiCompatibleClient::new(
                self.provider,
                base_url,
                &self.api_key,
                &self.model,
            );
            client.test_connection().await
        }
    }
}

pub fn get_static_models_for_provider(provider: AiProvider) -> Vec<ModelInfo> {
    match provider {
        AiProvider::Anthropic => anthropic::AnthropicClient::list_available_models(),
        _ => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_retryable_error() {
        assert!(is_retryable_error(&AiClientError::RateLimited(
            "test".to_string()
        )));
        assert!(is_retryable_error(&AiClientError::NetworkError(
            "test".to_string()
        )));
        assert!(is_retryable_error(&AiClientError::Timeout));

        assert!(!is_retryable_error(&AiClientError::InvalidApiKey));
        assert!(!is_retryable_error(&AiClientError::ModelNotAvailable(
            "test".to_string()
        )));
        assert!(!is_retryable_error(&AiClientError::InvalidResponse(
            "test".to_string()
        )));
        assert!(!is_retryable_error(&AiClientError::ApiKeyNotSet));
        assert!(!is_retryable_error(&AiClientError::ContextTooLong(100)));
    }

    #[tokio::test]
    async fn test_calculate_retry_delay() {
        assert_eq!(calculate_retry_delay(0).await, Duration::from_millis(1000));
        assert_eq!(calculate_retry_delay(1).await, Duration::from_millis(2000));
        assert_eq!(calculate_retry_delay(2).await, Duration::from_millis(4000));
        assert_eq!(calculate_retry_delay(3).await, Duration::from_millis(8000));
        assert_eq!(calculate_retry_delay(4).await, Duration::from_millis(10000));
        assert_eq!(
            calculate_retry_delay(10).await,
            Duration::from_millis(10000)
        );
    }

    #[test]
    fn test_get_static_models_for_anthropic() {
        let models = get_static_models_for_provider(AiProvider::Anthropic);
        assert!(!models.is_empty());
        assert!(models.iter().any(|m| m.id.contains("claude")));
    }

    #[test]
    fn test_get_static_models_for_other_providers() {
        assert!(get_static_models_for_provider(AiProvider::OpenAi).is_empty());
        assert!(get_static_models_for_provider(AiProvider::DeepSeek).is_empty());
    }

    #[test]
    fn test_ai_client_new() {
        let client = AiClient::new(AiProvider::OpenAi, None, "test-key", "gpt-4");
        assert_eq!(client.model, "gpt-4");
        assert_eq!(client.api_key, "test-key");
    }

    #[test]
    fn test_ai_client_with_custom_base_url() {
        let client = AiClient::new(
            AiProvider::OpenAiCompatible,
            Some("https://custom.api.com"),
            "test-key",
            "custom-model",
        );
        assert_eq!(client.base_url, Some("https://custom.api.com".to_string()));
    }
}
