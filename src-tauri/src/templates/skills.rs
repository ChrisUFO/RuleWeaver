use crate::models::{CreateSkillInput, Scope, SkillParameter, SkillParameterType};
use crate::templates::{THEME_ENGINEERING, THEME_PM, THEME_WRITING};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateFile {
    pub filename: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateSkill {
    pub template_id: String,
    pub theme: String,
    pub metadata: CreateSkillInput,
    pub files: Vec<TemplateFile>,
}

pub fn get_bundled_skill_templates() -> Vec<TemplateSkill> {
    vec![
        TemplateSkill {
            template_id: "tmpl_code_review".to_string(),
            theme: THEME_ENGINEERING.to_string(),
            metadata: CreateSkillInput {
                id: None,
                name: "Basic Code Reviewer".to_string(),
                description: "A Python skill that reviews code files for simple anti-patterns."
                    .to_string(),
                // Display instructions with placeholders for input
                instructions: include_str!("code_review_instructions.md").to_string(),
                input_schema: vec![SkillParameter {
                    name: "target_file".to_string(),
                    description: "The target file to review".to_string(),
                    param_type: SkillParameterType::String,
                    required: true,
                    default_value: None,
                    enum_values: None,
                }],
                directory_path: "".to_string(),
                entry_point: "python review.py".to_string(),
                scope: Scope::Global,
                enabled: true,
                ..Default::default()
            },
            files: vec![TemplateFile {
                filename: "review.py".to_string(),
                content: include_str!("review.py").to_string(),
            }],
        },
        TemplateSkill {
            template_id: "book-writing-assistant".to_string(),
            theme: THEME_WRITING.to_string(),
            metadata: CreateSkillInput {
                id: None,
                name: "Book Writing Assistant".to_string(),
                description:
                    "Structured workflow for authors to plan chapters and brainstorm plot beats."
                        .to_string(),
                instructions: include_str!("write_instructions.md").to_string(),
                scope: Scope::Global,
                input_schema: vec![],
                directory_path: "".to_string(),
                entry_point: "python write.py".to_string(),
                enabled: true,
                ..Default::default()
            },
            files: vec![TemplateFile {
                filename: "write.py".to_string(),
                content: include_str!("write.py").to_string(),
            }],
        },
        TemplateSkill {
            template_id: "project-planner".to_string(),
            theme: THEME_PM.to_string(),
            metadata: CreateSkillInput {
                id: None,
                name: "Project Planner".to_string(),
                description: "Generates project plans and Mermaid Gantt charts from feature lists."
                    .to_string(),
                instructions: include_str!("plan_instructions.md").to_string(),
                scope: Scope::Global,
                input_schema: vec![],
                directory_path: "".to_string(),
                entry_point: "python plan.py".to_string(),
                enabled: true,
                ..Default::default()
            },
            files: vec![TemplateFile {
                filename: "plan.py".to_string(),
                content: include_str!("plan.py").to_string(),
            }],
        },
        TemplateSkill {
            template_id: "tmpl_system_info".to_string(),
            theme: THEME_PM.to_string(),
            metadata: CreateSkillInput {
                id: None,
                name: "System Information".to_string(),
                description: "A Powershell script to dump basic system information.".to_string(),
                instructions: include_str!("sysinfo_instructions.md").to_string(),
                input_schema: vec![],
                directory_path: "".to_string(),
                entry_point: "pwsh -File ./sysinfo.ps1".to_string(),
                scope: Scope::Global,
                enabled: true,
                ..Default::default()
            },
            files: vec![TemplateFile {
                filename: "sysinfo.ps1".to_string(),
                content: include_str!("sysinfo.ps1").to_string(),
            }],
        },
    ]
}
