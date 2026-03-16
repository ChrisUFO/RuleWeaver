use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use super::parse_error::ParseEnumError;

pub const DEFAULT_IMPROVEMENT_PROMPT: &str = r#"# Role
You are an expert technical editor specializing in improving AI assistant rules and instructions.

# Task
Improve the provided rule content while preserving its original intent and technical accuracy.

# Guidelines
1. **Clarity & Conciseness**
   - Remove redundant phrases and wordiness
   - Use direct, imperative language
   - Break complex ideas into digestible points

2. **Structure & Formatting**
   - Add appropriate markdown headings (##, ###) to organize sections
   - Use bullet points or numbered lists for sequences and options
   - Use code blocks (```) for code examples, commands, or file paths
   - Use **bold** for important terms, `code` for technical identifiers

3. **Technical Accuracy**
   - Preserve all technical details, file paths, and commands exactly
   - Keep specific version numbers, URLs, and configuration values
   - Maintain the original scope and constraints

4. **Grammar & Style**
   - Fix spelling and grammar errors
   - Ensure consistent tense and voice
   - Use active voice preferentially

5. **Constraints**
   - Do NOT add new requirements or behaviors not implied by the original
   - Do NOT remove existing requirements unless clearly redundant
   - Do NOT change the fundamental purpose of the rule
   - Keep the same markdown format

# Output Format
Return ONLY the improved markdown content. Do not include explanations, comments, or meta-text about what changed.
"#;

pub const DEFAULT_GENERATION_PROMPT: &str = r#"# Role
You are an expert at creating AI assistant rules and instructions for software development tools.

# Task
Generate a comprehensive, well-structured rule based on the user's description.

# Guidelines
1. **Structure**
   - Start with a clear title/heading
   - Include a brief summary of the rule's purpose
   - Organize instructions into logical sections with headings
   - Use bullet points for lists of options or requirements

2. **Content**
   - Be specific and actionable
   - Include relevant code examples where appropriate
   - Specify constraints and preferences clearly
   - Consider edge cases and common scenarios

3. **Formatting**
   - Use proper markdown syntax
   - Use code blocks for code snippets, commands, and file paths
   - Use **bold** for emphasis on important terms
   - Keep sections concise but complete

4. **Best Practices**
   - Focus on what the AI should do, not what it shouldn't
   - Provide context that helps the AI make good decisions
   - Include relevant technical details (languages, frameworks, patterns)
   - Consider the tool's capabilities and limitations

# Output Format
Return ONLY the rule content in markdown format. Do not include explanations or meta-commentary.
"#;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AiProvider {
    #[default]
    OpenAi,
    Anthropic,
    GoogleAiStudio,
    OpenRouter,
    DeepSeek,
    TogetherAi,
    MinimaxApi,
    MinimaxCodingPlan,
    ZaiGeneralApi,
    ZaiCodingPlan,
    OpenAiCompatible,
}

impl AiProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            AiProvider::OpenAi => "openai",
            AiProvider::Anthropic => "anthropic",
            AiProvider::GoogleAiStudio => "google-ai-studio",
            AiProvider::OpenRouter => "openrouter",
            AiProvider::DeepSeek => "deepseek",
            AiProvider::TogetherAi => "together-ai",
            AiProvider::MinimaxApi => "minimax-api",
            AiProvider::MinimaxCodingPlan => "minimax-coding-plan",
            AiProvider::ZaiGeneralApi => "zai-general-api",
            AiProvider::ZaiCodingPlan => "zai-coding-plan",
            AiProvider::OpenAiCompatible => "openai-compatible",
        }
    }

    pub fn default_base_url(&self) -> &'static str {
        match self {
            AiProvider::OpenAi => "https://api.openai.com/v1",
            AiProvider::Anthropic => "https://api.anthropic.com/v1",
            AiProvider::GoogleAiStudio => "https://generativelanguage.googleapis.com/v1beta/openai",
            AiProvider::OpenRouter => "https://openrouter.ai/api/v1",
            AiProvider::DeepSeek => "https://api.deepseek.com",
            AiProvider::TogetherAi => "https://api.together.xyz/v1",
            AiProvider::MinimaxApi | AiProvider::MinimaxCodingPlan => "https://api.minimax.chat/v1",
            AiProvider::ZaiGeneralApi => "https://api.z.ai/api/paas/v4",
            AiProvider::ZaiCodingPlan => "https://api.z.ai/api/coding/paas/v4",
            AiProvider::OpenAiCompatible => "",
        }
    }

    pub fn default_model(&self) -> &'static str {
        match self {
            AiProvider::OpenAi => "gpt-4.1-mini",
            AiProvider::Anthropic => "claude-sonnet-4-20250514",
            AiProvider::GoogleAiStudio => "gemini-2.5-flash",
            AiProvider::OpenRouter => "openai/gpt-4.1-mini",
            AiProvider::DeepSeek => "deepseek-chat",
            AiProvider::TogetherAi => "meta-llama/Llama-3.3-70B-Instruct-Turbo",
            AiProvider::MinimaxApi | AiProvider::MinimaxCodingPlan => "MiniMax-Text-01",
            AiProvider::ZaiGeneralApi | AiProvider::ZaiCodingPlan => "",
            AiProvider::OpenAiCompatible => "",
        }
    }

    pub fn supports_model_listing(&self) -> bool {
        !matches!(self, AiProvider::Anthropic | AiProvider::OpenAiCompatible)
    }

    pub fn uses_native_anthropic_api(&self) -> bool {
        matches!(self, AiProvider::Anthropic)
    }
}

impl FromStr for AiProvider {
    type Err = ParseEnumError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "openai" => Ok(AiProvider::OpenAi),
            "anthropic" => Ok(AiProvider::Anthropic),
            "google-ai-studio" => Ok(AiProvider::GoogleAiStudio),
            "openrouter" => Ok(AiProvider::OpenRouter),
            "deepseek" => Ok(AiProvider::DeepSeek),
            "together-ai" => Ok(AiProvider::TogetherAi),
            "minimax-api" => Ok(AiProvider::MinimaxApi),
            "minimax-coding-plan" => Ok(AiProvider::MinimaxCodingPlan),
            "zai-general-api" => Ok(AiProvider::ZaiGeneralApi),
            "zai-coding-plan" => Ok(AiProvider::ZaiCodingPlan),
            "openai-compatible" => Ok(AiProvider::OpenAiCompatible),
            _ => Err(ParseEnumError),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiSettings {
    pub provider: AiProvider,
    pub base_url: Option<String>,
    pub model: String,
    pub has_api_key: bool,
    pub improvement_prompt: Option<String>,
    pub generation_prompt: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiSettingsRecord {
    pub provider: String,
    pub base_url: Option<String>,
    pub model: String,
    pub api_key_set: bool,
    pub improvement_prompt: Option<String>,
    pub generation_prompt: Option<String>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for AiSettingsRecord {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            provider: AiProvider::default().as_str().to_string(),
            base_url: None,
            model: AiProvider::default().default_model().to_string(),
            api_key_set: false,
            improvement_prompt: None,
            generation_prompt: None,
            enabled: false,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveAiSettingsInput {
    pub provider: AiProvider,
    pub base_url: Option<String>,
    pub model: String,
    pub api_key: Option<String>,
    pub improvement_prompt: Option<String>,
    pub generation_prompt: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct UpsertAiSettingsInput {
    pub provider: AiProvider,
    pub base_url: Option<String>,
    pub model: String,
    pub api_key_set: bool,
    pub improvement_prompt: Option<String>,
    pub generation_prompt: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImproveRuleInput {
    pub rule_content: String,
    pub rule_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateRuleInput {
    pub description: String,
    pub rule_name: Option<String>,
    pub context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateRuleOutput {
    pub rule_content: String,
    pub suggested_name: Option<String>,
    pub model_used: String,
    pub tokens_used: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImproveRuleOutput {
    pub improved_content: String,
    pub model_used: String,
    pub tokens_used: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestConnectionOutput {
    pub success: bool,
    pub model_available: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    pub id: String,
    pub name: Option<String>,
    pub context_length: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_provider_from_str() {
        assert!(matches!(
            AiProvider::from_str("openai"),
            Ok(AiProvider::OpenAi)
        ));
        assert!(matches!(
            AiProvider::from_str("anthropic"),
            Ok(AiProvider::Anthropic)
        ));
        assert!(matches!(
            AiProvider::from_str("google-ai-studio"),
            Ok(AiProvider::GoogleAiStudio)
        ));
        assert!(matches!(
            AiProvider::from_str("openrouter"),
            Ok(AiProvider::OpenRouter)
        ));
        assert!(matches!(
            AiProvider::from_str("openai-compatible"),
            Ok(AiProvider::OpenAiCompatible)
        ));
        assert!(AiProvider::from_str("invalid").is_err());
    }

    #[test]
    fn test_ai_provider_as_str() {
        assert_eq!(AiProvider::OpenAi.as_str(), "openai");
        assert_eq!(AiProvider::Anthropic.as_str(), "anthropic");
        assert_eq!(AiProvider::GoogleAiStudio.as_str(), "google-ai-studio");
        assert_eq!(AiProvider::OpenAiCompatible.as_str(), "openai-compatible");
    }

    #[test]
    fn test_ai_provider_default_base_url() {
        assert_eq!(
            AiProvider::OpenAi.default_base_url(),
            "https://api.openai.com/v1"
        );
        assert_eq!(
            AiProvider::Anthropic.default_base_url(),
            "https://api.anthropic.com/v1"
        );
        assert_eq!(AiProvider::OpenAiCompatible.default_base_url(), "");
    }

    #[test]
    fn test_ai_provider_uses_native_anthropic_api() {
        assert!(AiProvider::Anthropic.uses_native_anthropic_api());
        assert!(!AiProvider::OpenAi.uses_native_anthropic_api());
        assert!(!AiProvider::OpenRouter.uses_native_anthropic_api());
    }

    #[test]
    fn test_ai_provider_supports_model_listing() {
        assert!(AiProvider::OpenAi.supports_model_listing());
        assert!(!AiProvider::Anthropic.supports_model_listing());
        assert!(!AiProvider::OpenAiCompatible.supports_model_listing());
    }

    #[test]
    fn test_save_ai_settings_input_serialization() {
        let input = SaveAiSettingsInput {
            provider: AiProvider::OpenAi,
            base_url: Some("https://api.openai.com/v1".to_string()),
            model: "gpt-4.1-mini".to_string(),
            api_key: Some("sk-test".to_string()),
            improvement_prompt: None,
            generation_prompt: None,
            enabled: true,
        };

        let json = serde_json::to_string(&input).unwrap();
        let parsed: SaveAiSettingsInput = serde_json::from_str(&json).unwrap();

        assert!(matches!(parsed.provider, AiProvider::OpenAi));
        assert_eq!(parsed.model, "gpt-4.1-mini");
        assert!(parsed.enabled);
    }

    #[test]
    fn test_ai_settings_camel_case_serialization() {
        let settings = AiSettings {
            provider: AiProvider::OpenAi,
            base_url: Some("https://api.openai.com/v1".to_string()),
            model: "gpt-4.1-mini".to_string(),
            has_api_key: true,
            improvement_prompt: Some("Custom prompt".to_string()),
            generation_prompt: None,
            enabled: true,
        };

        let json = serde_json::to_string(&settings).unwrap();

        assert!(json.contains("\"baseUrl\""));
        assert!(json.contains("\"hasApiKey\""));
        assert!(json.contains("\"improvementPrompt\""));
        assert!(json.contains("\"generationPrompt\""));
        assert!(!json.contains("\"base_url\""));
        assert!(!json.contains("\"has_api_key\""));
    }
}
