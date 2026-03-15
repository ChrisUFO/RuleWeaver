use serde::{Deserialize, Serialize};

use crate::models::AdapterType;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolSyncPreferences {
    pub tool_id: AdapterType,
    pub sync_rules: bool,
    pub sync_commands: bool,
    pub sync_skills: bool,
}

impl Default for ToolSyncPreferences {
    fn default() -> Self {
        Self {
            tool_id: AdapterType::Gemini,
            sync_rules: true,
            sync_commands: true,
            sync_skills: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertToolSyncPreferencesInput {
    pub tool_id: AdapterType,
    pub sync_rules: Option<bool>,
    pub sync_commands: Option<bool>,
    pub sync_skills: Option<bool>,
}
