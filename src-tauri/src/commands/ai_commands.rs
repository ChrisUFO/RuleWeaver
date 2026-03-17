use std::str::FromStr;
use std::sync::Arc;
use tauri::State;

use crate::ai::AiClient;
use crate::database::Database;
use crate::error::Result;
use crate::models::{
    AiProvider, AiSettings, GenerateRuleInput, GenerateRuleOutput, ImproveRuleInput,
    ImproveRuleOutput, ModelInfo, SaveAiSettingsInput, TestConnectionOutput, UpsertAiSettingsInput,
    DEFAULT_GENERATION_PROMPT, DEFAULT_IMPROVEMENT_PROMPT,
};
use crate::secure_storage::SecretStorage;

#[tauri::command]
pub async fn get_ai_settings(db: State<'_, Arc<Database>>) -> Result<AiSettings> {
    let record = db.get_ai_settings().await?;

    let provider = AiProvider::from_str(&record.provider).unwrap_or_default();

    Ok(AiSettings {
        provider,
        base_url: record.base_url,
        model: record.model,
        has_api_key: record.api_key_set,
        improvement_prompt: record.improvement_prompt,
        generation_prompt: record.generation_prompt,
        enabled: record.enabled,
    })
}

#[tauri::command]
pub async fn save_ai_settings(
    db: State<'_, Arc<Database>>,
    input: SaveAiSettingsInput,
) -> Result<AiSettings> {
    let has_api_key = input.api_key.is_some() && !input.api_key.as_ref().unwrap().is_empty();

    if let Some(ref api_key) = input.api_key {
        if !api_key.is_empty() {
            let key_id = format!("{}-api-key", input.provider.as_str());
            let storage = SecretStorage::global();
            storage.set_secret(&key_id, api_key).await?;
        }
    }

    let base_url = if input.provider == AiProvider::OpenAiCompatible {
        input.base_url.clone()
    } else {
        let default = input.provider.default_base_url().to_string();
        if input.base_url.as_deref() == Some(&default) || input.base_url.is_none() {
            None
        } else {
            input.base_url
        }
    };

    let upsert_input = UpsertAiSettingsInput {
        provider: input.provider,
        base_url,
        model: input.model,
        api_key_set: has_api_key,
        improvement_prompt: input.improvement_prompt,
        generation_prompt: input.generation_prompt,
        enabled: input.enabled,
    };

    db.upsert_ai_settings(&upsert_input).await?;

    get_ai_settings(db).await
}

#[tauri::command]
pub async fn test_ai_connection(db: State<'_, Arc<Database>>) -> Result<TestConnectionOutput> {
    let settings = db.get_ai_settings().await?;

    if !settings.enabled {
        return Ok(TestConnectionOutput {
            success: false,
            model_available: false,
            error: Some("AI feature is not enabled".to_string()),
        });
    }

    let provider = match AiProvider::from_str(&settings.provider) {
        Ok(p) => p,
        Err(_) => {
            return Ok(TestConnectionOutput {
                success: false,
                model_available: false,
                error: Some(format!("Invalid provider: {}", settings.provider)),
            });
        }
    };

    let api_key = match get_api_key_for_provider(&settings.provider).await? {
        Some(key) => key,
        None => {
            return Ok(TestConnectionOutput {
                success: false,
                model_available: false,
                error: Some("API key not configured".to_string()),
            });
        }
    };

    let client = AiClient::new(
        provider,
        settings.base_url.as_deref(),
        &api_key,
        &settings.model,
    );

    match client.test_connection().await {
        Ok(true) => Ok(TestConnectionOutput {
            success: true,
            model_available: true,
            error: None,
        }),
        Ok(false) => Ok(TestConnectionOutput {
            success: false,
            model_available: false,
            error: Some("Connection test failed".to_string()),
        }),
        Err(e) => Ok(TestConnectionOutput {
            success: false,
            model_available: false,
            error: Some(e.to_string()),
        }),
    }
}

#[tauri::command]
pub async fn list_ai_models(db: State<'_, Arc<Database>>) -> Result<Vec<ModelInfo>> {
    let settings = db.get_ai_settings().await?;

    if !settings.enabled {
        return Ok(vec![]);
    }

    let provider = match AiProvider::from_str(&settings.provider) {
        Ok(p) => p,
        Err(_) => return Ok(vec![]),
    };

    let api_key = match get_api_key_for_provider(&settings.provider).await? {
        Some(key) => key,
        None => return Ok(vec![]),
    };

    let client = AiClient::new(
        provider,
        settings.base_url.as_deref(),
        &api_key,
        &settings.model,
    );

    match client.list_models().await {
        Ok(models) => Ok(models),
        Err(_) => Ok(vec![]),
    }
}

#[tauri::command]
pub async fn improve_rule_with_ai(
    db: State<'_, Arc<Database>>,
    input: ImproveRuleInput,
) -> Result<ImproveRuleOutput> {
    log::info!(
        "improve_rule_with_ai called - content_length: {}",
        input.rule_content.len()
    );

    let settings = db.get_ai_settings().await?;
    log::debug!(
        "improve_rule_with_ai: provider={}, model={:?}, enabled={}",
        settings.provider,
        settings.model,
        settings.enabled
    );

    if !settings.enabled {
        log::warn!("improve_rule_with_ai: AI feature is not enabled");
        return Err(crate::error::AppError::Ai(
            crate::error::AiError::NotEnabled,
        ));
    }

    let provider = match AiProvider::from_str(&settings.provider) {
        Ok(p) => {
            log::debug!("improve_rule_with_ai: Parsed provider: {:?}", p);
            p
        }
        Err(_) => {
            let error_message = format!("Invalid provider: {}", settings.provider);
            log::error!("improve_rule_with_ai: {}", error_message);
            return Err(crate::error::AppError::Ai(
                crate::error::AiError::RequestFailed(error_message),
            ));
        }
    };

    let api_key = match get_api_key_for_provider(&settings.provider).await? {
        Some(key) => {
            log::debug!("improve_rule_with_ai: Found API key, length: {}", key.len());
            key
        }
        None => {
            log::warn!(
                "improve_rule_with_ai: API key not configured for provider: {}",
                settings.provider
            );
            return Err(crate::error::AppError::Ai(
                crate::error::AiError::ApiKeyNotSet,
            ));
        }
    };

    let prompt = settings
        .improvement_prompt
        .unwrap_or_else(|| DEFAULT_IMPROVEMENT_PROMPT.to_string());

    log::debug!(
        "improve_rule_with_ai: Using prompt, length: {}",
        prompt.len()
    );

    let user_message = if let Some(ref name) = input.rule_name {
        format!(
            "Improve this rule named '{}' by applying the guidelines.\n\n# Rule Content\n\n{}",
            name, input.rule_content
        )
    } else {
        format!(
            "Improve the following rule by applying the guidelines.\n\n# Rule Content\n\n{}",
            input.rule_content
        )
    };

    log::debug!(
        "improve_rule_with_ai: Calling AI client - model: {}, base_url: {:?}",
        settings.model,
        settings.base_url
    );

    let client = AiClient::new(
        provider,
        settings.base_url.as_deref(),
        &api_key,
        &settings.model,
    );

    match client.complete(&prompt, &user_message).await {
        Ok(improved_content) => {
            log::info!(
                "improve_rule_with_ai: Successfully improved rule - model: {}, content_length: {}",
                settings.model,
                improved_content.len()
            );
            Ok(ImproveRuleOutput {
                improved_content,
                model_used: settings.model,
                tokens_used: None,
            })
        }
        Err(e) => {
            log::error!("improve_rule_with_ai: AI client error: {:?}", e);
            Err(crate::error::AppError::Ai(e.into()))
        }
    }
}

#[tauri::command]
pub async fn generate_rule_with_ai(
    db: State<'_, Arc<Database>>,
    input: GenerateRuleInput,
) -> Result<GenerateRuleOutput> {
    log::info!(
        "generate_rule_with_ai called - description_length: {}",
        input.description.len()
    );

    let settings = db.get_ai_settings().await?;
    log::debug!(
        "generate_rule_with_ai: provider={}, model={:?}, enabled={}",
        settings.provider,
        settings.model,
        settings.enabled
    );

    if !settings.enabled {
        log::warn!("generate_rule_with_ai: AI feature is not enabled");
        return Err(crate::error::AppError::Ai(
            crate::error::AiError::NotEnabled,
        ));
    }

    let provider = match AiProvider::from_str(&settings.provider) {
        Ok(p) => {
            log::debug!("generate_rule_with_ai: Parsed provider: {:?}", p);
            p
        }
        Err(_) => {
            let error_message = format!("Invalid provider: {}", settings.provider);
            log::error!("generate_rule_with_ai: {}", error_message);
            return Err(crate::error::AppError::Ai(
                crate::error::AiError::RequestFailed(error_message),
            ));
        }
    };

    let api_key = match get_api_key_for_provider(&settings.provider).await? {
        Some(key) => {
            log::debug!(
                "generate_rule_with_ai: Found API key, length: {}",
                key.len()
            );
            key
        }
        None => {
            log::warn!(
                "generate_rule_with_ai: API key not configured for provider: {}",
                settings.provider
            );
            return Err(crate::error::AppError::Ai(
                crate::error::AiError::ApiKeyNotSet,
            ));
        }
    };

    let prompt = settings
        .generation_prompt
        .unwrap_or_else(|| DEFAULT_GENERATION_PROMPT.to_string());

    log::debug!(
        "generate_rule_with_ai: Using prompt, length: {}",
        prompt.len()
    );

    let user_message = if let Some(ref name) = input.rule_name {
        if let Some(ref context) = input.context {
            format!(
                "Create a rule named '{}' with the following description and context.\n\n# Description\n\n{}\n\n# Context\n\n{}",
                name, input.description, context
            )
        } else {
            format!(
                "Create a rule named '{}' with the following description.\n\n# Description\n\n{}",
                name, input.description
            )
        }
    } else if let Some(ref context) = input.context {
        format!(
            "Create a rule with the following description and context.\n\n# Description\n\n{}\n\n# Context\n\n{}",
            input.description, context
        )
    } else {
        format!(
            "Create a rule with the following description.\n\n# Description\n\n{}",
            input.description
        )
    };

    log::debug!(
        "generate_rule_with_ai: Calling AI client - model: {}, base_url: {:?}",
        settings.model,
        settings.base_url
    );

    let client = AiClient::new(
        provider,
        settings.base_url.as_deref(),
        &api_key,
        &settings.model,
    );

    match client.complete(&prompt, &user_message).await {
        Ok(rule_content) => {
            log::info!("generate_rule_with_ai: Successfully generated rule - model: {}, content_length: {}",
                settings.model, rule_content.len());
            let suggested_name = input
                .rule_name
                .or_else(|| extract_title_from_content(&rule_content));

            Ok(GenerateRuleOutput {
                rule_content,
                suggested_name,
                model_used: settings.model,
                tokens_used: None,
            })
        }
        Err(e) => {
            log::error!("generate_rule_with_ai: AI client error: {:?}", e);
            Err(crate::error::AppError::Ai(e.into()))
        }
    }
}

#[tauri::command]
pub async fn get_default_ai_prompts() -> Result<(String, String)> {
    Ok((
        DEFAULT_IMPROVEMENT_PROMPT.to_string(),
        DEFAULT_GENERATION_PROMPT.to_string(),
    ))
}

async fn get_api_key_for_provider(provider: &str) -> Result<Option<String>> {
    let key_id = format!("{}-api-key", provider);
    let storage = SecretStorage::global();
    storage.get_secret(&key_id).await
}

fn extract_title_from_content(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(stripped) = trimmed.strip_prefix("# ") {
            return Some(stripped.to_string());
        }
    }
    None
}
