use crate::models::{AdapterType, CreateRuleInput, Scope};
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
            theme: "Engineering".to_string(),
            metadata: CreateRuleInput {
                name: "React & TypeScript Standards".to_string(),
                content: "## Standards\n- Use functional components\n- Use TypeScript for all props\n- Prefer Tailwind CSS".to_string(),
                scope: Scope::Global,
                target_paths: None,
                enabled_adapters: vec![AdapterType::Gemini, AdapterType::Cursor],
                enabled: true,
            },
        },
        TemplateRule {
            template_id: "tmpl_author_persona".to_string(),
            theme: "Writing".to_string(),
            metadata: CreateRuleInput {
                name: "Author Persona".to_string(),
                content: "## Persona\n- Act as a seasoned business book editor.\n- Focus on clarity and impact.\n- Use active voice.".to_string(),
                scope: Scope::Global,
                target_paths: None,
                enabled_adapters: vec![AdapterType::Gemini],
                enabled: true,
            },
        },
        TemplateRule {
            template_id: "tmpl_pm_copilot".to_string(),
            theme: "Project Management".to_string(),
            metadata: CreateRuleInput {
                name: "PM Copilot".to_string(),
                content: "# Senior PM Persona\n- **Role**: Act as a Senior Product Manager...\n- **Tone**: Professional, concise...".to_string(),
                scope: Scope::Global,
                target_paths: None,
                enabled_adapters: vec![AdapterType::Gemini],
                enabled: true,
            },
        },
        TemplateRule {
            template_id: "tmpl_marketing_strategist".to_string(),
            theme: "Marketing".to_string(),
            metadata: CreateRuleInput {
                name: "Marketing Strategist".to_string(),
                content: "# Marketing Copy & Strategy Setup\n- **Role**: Act as an expert Digital Marketing Director...".to_string(),
                scope: Scope::Global,
                target_paths: None,
                enabled_adapters: vec![AdapterType::Gemini],
                enabled: true,
            },
        },
        TemplateRule {
            template_id: "tmpl_data_analyst".to_string(),
            theme: "Data Analysis".to_string(),
            metadata: CreateRuleInput {
                name: "Data Analyst Assistant".to_string(),
                content: "# Data Analyst Oracle\n- **Role**: Act as a Senior Data Scientist...".to_string(),
                scope: Scope::Global,
                target_paths: None,
                enabled_adapters: vec![AdapterType::Gemini],
                enabled: true,
            },
        },
        TemplateRule {
            template_id: "tmpl_exec_assistant".to_string(),
            theme: "Admin".to_string(),
            metadata: CreateRuleInput {
                name: "Executive Assistant".to_string(),
                content: "# Executive Assistant Mode\n- **Communication Principle**: Extreme brevity...".to_string(),
                scope: Scope::Global,
                target_paths: None,
                enabled_adapters: vec![AdapterType::Gemini],
                enabled: true,
            },
        },
    ]
}
