use crate::ai::error::{AiClientError, AiResult};
use crate::ai::models::{AnthropicRequest, ChatCompletionRequest};
use crate::models::{AiProvider, ModelInfo};

pub mod anthropic;
pub mod error;
pub mod models;
pub mod openai_compatible;

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
        if self.provider.uses_native_anthropic_api() {
            self.complete_anthropic(system_prompt, user_message).await
        } else {
            self.complete_openai_compatible(system_prompt, user_message)
                .await
        }
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
