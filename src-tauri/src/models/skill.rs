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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SkillParameterType {
    String,
    Number,
    Boolean,
}

fn default_skill_param_type() -> SkillParameterType {
    SkillParameterType::String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSkillInput {
    pub name: String,
    pub description: String,
    pub instructions: String,
    #[serde(default)]
    pub input_schema: Vec<SkillParameter>,
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
    pub enabled: Option<bool>,
}
