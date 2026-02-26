use crate::error::{AppError, Result};
use crate::models::Scope;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub instructions: String,
    pub scope: Scope,
    #[serde(default)]
    pub input_schema: Vec<SkillParameter>,
    pub enabled: bool,
    pub directory_path: String,
    pub entry_point: String,
    /// Adapters to sync this skill to. Empty = all supported adapters.
    #[serde(default)]
    pub target_adapters: Vec<String>,
    /// Repository roots for local-scope syncing.
    #[serde(default)]
    pub target_paths: Vec<String>,
    #[serde(with = "crate::models::timestamp")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "crate::models::timestamp")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillParameter {
    pub name: String,
    pub description: String,
    #[serde(default = "default_skill_param_type")]
    pub param_type: SkillParameterType,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub default_value: Option<String>,
    #[serde(default)]
    pub enum_values: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SkillParameterType {
    String,
    Number,
    Boolean,
    Enum,
    Array,
    Object,
}

fn default_skill_param_type() -> SkillParameterType {
    SkillParameterType::String
}

impl Skill {
    pub fn validate_payload(
        &self,
        args_map: &serde_json::Map<String, serde_json::Value>,
    ) -> Result<Vec<(String, String)>> {
        let mut skill_envs: Vec<(String, String)> = Vec::new();
        let mut missing_required: Vec<String> = Vec::new();

        for param in &self.input_schema {
            let mut raw_value = args_map
                .get(&param.name)
                .map(|v| {
                    if let Some(s) = v.as_str() {
                        s.to_string()
                    } else {
                        v.to_string()
                    }
                })
                .unwrap_or_default();

            if raw_value.is_empty() {
                if let Some(ref default) = param.default_value {
                    raw_value = default.clone();
                }
            }

            if raw_value.is_empty() && param.required {
                missing_required.push(param.name.clone());
                continue;
            }

            if raw_value.is_empty() {
                continue;
            }

            if matches!(param.param_type, SkillParameterType::Enum) {
                if let Some(ref options) = param.enum_values {
                    if !options.contains(&raw_value) {
                        return Err(AppError::Validation(format!(
                            "Invalid value '{}' for parameter '{}'. Must be one of: {}",
                            raw_value,
                            param.name,
                            options.join(", ")
                        )));
                    }
                }
            }

            let env_name = format!(
                "{}{}",
                crate::constants::skills::SKILL_PARAM_PREFIX,
                param.name.replace('-', "_").to_uppercase()
            );
            skill_envs.push((env_name, raw_value));
        }

        if !missing_required.is_empty() {
            return Err(AppError::Validation(format!(
                "Missing required parameters: {}",
                missing_required.join(", ")
            )));
        }

        Ok(skill_envs)
    }
}

pub fn validate_skill_input(name: &str, instructions: &str) -> Result<()> {
    if name.trim().is_empty() {
        return Err(AppError::Validation(
            "Skill name cannot be empty".to_string(),
        ));
    }
    if name.len() > crate::constants::limits::MAX_SKILL_NAME_LENGTH {
        return Err(AppError::Validation(format!(
            "Skill name too long (max {} characters)",
            crate::constants::limits::MAX_SKILL_NAME_LENGTH
        )));
    }
    if instructions.len() > crate::constants::limits::MAX_SKILL_INSTRUCTIONS_LENGTH {
        return Err(AppError::Validation(format!(
            "Skill instructions too large (max {} characters)",
            crate::constants::limits::MAX_SKILL_INSTRUCTIONS_LENGTH
        )));
    }
    Ok(())
}

pub fn validate_skill_schema(schema: &[SkillParameter]) -> Result<()> {
    let mut names = std::collections::HashSet::new();

    for param in schema {
        let trimmed_name = param.name.trim();
        if trimmed_name.is_empty() {
            return Err(AppError::Validation(
                "Parameter name cannot be empty".to_string(),
            ));
        }

        if !trimmed_name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            return Err(AppError::Validation(format!(
                "Parameter name '{}' can only contain alphanumeric characters and underscores",
                trimmed_name
            )));
        }

        if !names.insert(trimmed_name.to_lowercase()) {
            return Err(AppError::Validation(format!(
                "Duplicate parameter name: {}",
                trimmed_name
            )));
        }

        if matches!(param.param_type, SkillParameterType::Enum) {
            let options = param.enum_values.as_ref();
            if options.is_none() || options.unwrap().is_empty() {
                return Err(AppError::Validation(format!(
                    "Parameter '{}' is type Enum but has no enum_values defined",
                    trimmed_name
                )));
            }
            let options_vec = options.unwrap();

            if let Some(default_val) = &param.default_value {
                if !options_vec.contains(default_val) {
                    return Err(AppError::Validation(format!(
                        "Default value '{}' for Enum parameter '{}' must be one of the enum_values",
                        default_val, trimmed_name
                    )));
                }
            }
        }
    }

    Ok(())
}

/// Validate that all adapter IDs in `target_adapters` are known and support skills.
pub fn validate_skill_target_adapters(target_adapters: &[String]) -> Result<()> {
    use crate::models::registry::{ArtifactType, REGISTRY};
    use crate::models::{AdapterType, Scope};
    for adapter_str in target_adapters {
        let adapter = AdapterType::from_str(adapter_str).ok_or_else(|| {
            AppError::Validation(format!("Unknown adapter: '{}'", adapter_str))
        })?;
        REGISTRY
            .validate_support(&adapter, &Scope::Global, ArtifactType::Skill)
            .map_err(|_| {
                AppError::Validation(format!(
                    "Adapter '{}' does not support skills",
                    adapter_str
                ))
            })?;
    }
    Ok(())
}

pub fn validate_skill_entry_point(entry_point: &str) -> Result<()> {
    let trimmed = entry_point.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation(
            "Entry point cannot be empty".to_string(),
        ));
    }
    if trimmed.contains("..")
        || trimmed.starts_with('/')
        || trimmed.starts_with('\\')
        || trimmed.contains(':')
    {
        return Err(AppError::Validation(
            "Entry point must be a relative path without directory traversal".to_string(),
        ));
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CreateSkillInput {
    pub id: Option<String>,
    pub name: String,
    pub description: String,
    pub instructions: String,
    pub scope: Scope,
    #[serde(default)]
    pub input_schema: Vec<SkillParameter>,
    pub directory_path: String,
    pub entry_point: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Adapters to sync this skill to. Empty = all supported adapters.
    #[serde(default)]
    pub target_adapters: Vec<String>,
    /// Repository roots for local-scope syncing.
    #[serde(default)]
    pub target_paths: Vec<String>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSkillInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub instructions: Option<String>,
    pub scope: Option<Scope>,
    pub input_schema: Option<Vec<SkillParameter>>,
    pub directory_path: Option<String>,
    pub entry_point: Option<String>,
    pub enabled: Option<bool>,
    /// Adapters to sync this skill to. None = no change; Some([]) = all adapters.
    pub target_adapters: Option<Vec<String>>,
    /// Repository roots for local-scope syncing.
    pub target_paths: Option<Vec<String>>,
}
