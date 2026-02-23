use crate::error::{AppError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub instructions: String,
    #[serde(default)]
    pub input_schema: Vec<SkillParameter>,
    pub enabled: bool,
    pub directory_path: String,
    pub entry_point: String,
    #[serde(with = "crate::models::timestamp")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "crate::models::timestamp")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
            let raw_value = args_map
                .get(&param.name)
                .map(|v| {
                    if let Some(s) = v.as_str() {
                        s.to_string()
                    } else {
                        v.to_string()
                    }
                })
                .or_else(|| param.default_value.clone())
                .unwrap_or_default();

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
                            "Parameter '{}' must be one of: {}",
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

    pub fn validate_metadata(&self) -> Result<()> {
        validate_skill_input(&self.name, &self.instructions)?;
        validate_skill_schema(&self.input_schema)?;
        validate_skill_entry_point(&self.entry_point)?;
        if self.directory_path.trim().is_empty() {
            return Err(AppError::Validation(
                "directory_path cannot be empty".to_string(),
            ));
        }
        Ok(())
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSkillInput {
    pub id: Option<String>,
    pub name: String,
    pub description: String,
    pub instructions: String,
    #[serde(default)]
    pub input_schema: Vec<SkillParameter>,
    pub directory_path: String,
    pub entry_point: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateSkillInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub instructions: Option<String>,
    pub input_schema: Option<Vec<SkillParameter>>,
    pub directory_path: Option<String>,
    pub entry_point: Option<String>,
    pub enabled: Option<bool>,
}
