use crate::models::{CreateSkillInput, SkillParameter, SkillParameterType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateFile {
    pub filename: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateSkill {
    pub template_id: String,
    pub metadata: CreateSkillInput,
    pub files: Vec<TemplateFile>,
}

pub fn get_bundled_skill_templates() -> Vec<TemplateSkill> {
    vec![
        TemplateSkill {
            template_id: "tmpl_code_review".to_string(),
            metadata: CreateSkillInput {
                id: None,
                name: "Basic Code Reviewer".to_string(),
                description: "A Python skill that reviews code files for simple anti-patterns.".to_string(),
                // Display instructions with placeholders for input
                instructions: "Review the target code file for security flaws.\n\n```bash\npython review.py\n```".to_string(),
                input_schema: vec![
                    SkillParameter {
                        name: "target_file".to_string(),
                        description: "The target file to review".to_string(),
                        param_type: SkillParameterType::String,
                        required: true,
                        default_value: None,
                        enum_values: None,
                    }
                ],
                directory_path: "".to_string(),
                entry_point: "python review.py".to_string(),
                enabled: true,
            },
            files: vec![
                TemplateFile {
                    filename: "review.py".to_string(),
                    content: include_str!("review.py").to_string(),
                }
            ],
        },
        TemplateSkill {
            template_id: "tmpl_system_info".to_string(),
            metadata: CreateSkillInput {
                id: None,
                name: "System Information".to_string(),
                description: "A Powershell script to dump basic system information.".to_string(),
                instructions: "Dumps system information.\n\n```powershell\n.\\sysinfo.ps1\n```".to_string(),
                input_schema: vec![],
                directory_path: "".to_string(),
                entry_point: "pwsh -File ./sysinfo.ps1".to_string(),
                enabled: true,
            },
            files: vec![
                TemplateFile {
                    filename: "sysinfo.ps1".to_string(),
                    content: include_str!("sysinfo.ps1").to_string(),
                }
            ],
        }
    ]
}
