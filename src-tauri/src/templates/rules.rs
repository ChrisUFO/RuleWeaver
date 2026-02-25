use crate::models::{AdapterType, CreateRuleInput, Scope};
use crate::templates::{
    THEME_ADMIN, THEME_DATA, THEME_ENGINEERING, THEME_MARKETING, THEME_PM, THEME_WRITING,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateRule {
    pub template_id: String,
    pub theme: String,
    pub metadata: CreateRuleInput,
}

pub fn get_bundled_rule_templates() -> Vec<TemplateRule> {
    vec![
        TemplateRule {
            template_id: "tmpl_react_ts_standards".to_string(),
            theme: THEME_ENGINEERING.to_string(),
            metadata: CreateRuleInput {
                id: None,
                name: "React & TypeScript Standards".to_string(),
                description: "Enforce best practices for React and TypeScript development, including functional components, TypeScript for props, and Tailwind CSS preference.".to_string(),
                content: "## Standards\n- Use functional components\n- Use TypeScript for all props\n- Prefer Tailwind CSS".to_string(),
                scope: Scope::Global,
                target_paths: None,
                enabled_adapters: vec![AdapterType::Gemini, AdapterType::Cursor],
                enabled: true,
            },
        },
        TemplateRule {
            template_id: "tmpl_author_persona".to_string(),
            theme: THEME_WRITING.to_string(),
            metadata: CreateRuleInput {
                id: None,
                name: "Author Persona".to_string(),
                description: "Set a specific tone and persona for the AI assistant, focusing on clarity, impact, and a professional editing style.".to_string(),
                content: "## Persona\n- Act as a seasoned business book editor.\n- Focus on clarity and impact.\n- Use active voice.".to_string(),
                scope: Scope::Global,
                target_paths: None,
                enabled_adapters: vec![AdapterType::Gemini],
                enabled: true,
            },
        },
        TemplateRule {
            template_id: "tmpl_pm_copilot".to_string(),
            theme: THEME_PM.to_string(),
            metadata: CreateRuleInput {
                id: None,
                name: "PM Copilot".to_string(),
                description: "Collaborate with a Senior Product Manager persona for strategic planning, documentation, and product discovery.".to_string(),
                content: "# Senior PM Persona\n- **Role**: Act as a Senior Product Manager...\n- **Tone**: Professional, concise...".to_string(),
                scope: Scope::Global,
                target_paths: None,
                enabled_adapters: vec![AdapterType::Gemini],
                enabled: true,
            },
        },
        TemplateRule {
            template_id: "tmpl_marketing_strategist".to_string(),
            theme: THEME_MARKETING.to_string(),
            metadata: CreateRuleInput {
                id: None,
                name: "Marketing Strategist".to_string(),
                description: "Engage an expert Digital Marketing Director for copy generation, strategy brainstorming, and audience targeting.".to_string(),
                content: "# Marketing Copy & Strategy Setup\n- **Role**: Act as an expert Digital Marketing Director...".to_string(),
                scope: Scope::Global,
                target_paths: None,
                enabled_adapters: vec![AdapterType::Gemini],
                enabled: true,
            },
        },
        TemplateRule {
            template_id: "tmpl_data_analyst".to_string(),
            theme: THEME_DATA.to_string(),
            metadata: CreateRuleInput {
                id: None,
                name: "Data Analyst Assistant".to_string(),
                description: "Leverage a Senior Data Scientist persona for data interpretation, query optimization, and statistical insights.".to_string(),
                content: "# Data Analyst Oracle\n- **Role**: Act as a Senior Data Scientist...".to_string(),
                scope: Scope::Global,
                target_paths: None,
                enabled_adapters: vec![AdapterType::Gemini],
                enabled: true,
            },
        },
        TemplateRule {
            template_id: "tmpl_exec_assistant".to_string(),
            theme: THEME_ADMIN.to_string(),
            metadata: CreateRuleInput {
                id: None,
                name: "Executive Assistant".to_string(),
                description: "Enable an efficient Executive Assistant mode for brevity, task prioritization, and professional communication.".to_string(),
                content: "# Executive Assistant Mode\n- **Communication Principle**: Extreme brevity...".to_string(),
                scope: Scope::Global,
                target_paths: None,
                enabled_adapters: vec![AdapterType::Gemini],
                enabled: true,
            },
        },
    ]
}
