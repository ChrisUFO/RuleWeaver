use crate::models::{Command, Rule, Skill};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfiguration {
    pub version: String,
    pub exported_at: DateTime<Utc>,
    pub rules: Vec<Rule>,
    pub commands: Vec<Command>,
    pub skills: Vec<Skill>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ImportMode {
    Overwrite,
    Skip,
}

impl ExportConfiguration {
    pub fn new(rules: Vec<Rule>, commands: Vec<Command>, skills: Vec<Skill>) -> Self {
        Self {
            version: "1.0".to_string(),
            exported_at: Utc::now(),
            rules,
            commands,
            skills,
        }
    }
}
