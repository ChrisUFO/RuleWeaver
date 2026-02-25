use crate::models::CreateCommandInput;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateCommand {
    pub template_id: String,
    pub theme: String,
    pub metadata: CreateCommandInput,
}

pub fn get_bundled_command_templates() -> Vec<TemplateCommand> {
    vec![
        TemplateCommand {
            template_id: "tmpl_refactor_clean_code".to_string(),
            theme: "Engineering".to_string(),
            metadata: CreateCommandInput {
                id: None,
                name: "Refactor Clean Code".to_string(),
                description: "Refactors code for readability and maintainability.".to_string(),
                script: "echo \"Refactoring selected code...\"".to_string(),
                arguments: vec![],
                expose_via_mcp: true,
                is_placeholder: true,
                generate_slash_commands: false,
                slash_command_adapters: vec![],
                target_paths: vec![],
            },
        },
        TemplateCommand {
            template_id: "tmpl_generate_prd".to_string(),
            theme: "Project Management".to_string(),
            metadata: CreateCommandInput {
                id: None,
                name: "Generate PRD Outline".to_string(),
                description: "Drafts a PRD outline from notes.".to_string(),
                script: "echo \"Generating PRD...\"".to_string(),
                arguments: vec![],
                expose_via_mcp: true,
                is_placeholder: true,
                generate_slash_commands: false,
                slash_command_adapters: vec![],
                target_paths: vec![],
            },
        },
        TemplateCommand {
            template_id: "tmpl_user_story_map".to_string(),
            theme: "Project Management".to_string(),
            metadata: CreateCommandInput {
                id: None,
                name: "User Story Map".to_string(),
                description: "Converts feature ideas into structured user stories.".to_string(),
                script: "echo \"Generating user story map...\"".to_string(),
                arguments: vec![],
                expose_via_mcp: true,
                generate_slash_commands: false,
                slash_command_adapters: vec![],
                target_paths: vec![],
                is_placeholder: true,
            },
        },
        TemplateCommand {
            template_id: "tmpl_brainstorm_chapter".to_string(),
            theme: "Writing".to_string(),
            metadata: CreateCommandInput {
                id: None,
                name: "Brainstorm Chapter Beats".to_string(),
                description: "Generates a beat sheet for a specific chapter scenario.".to_string(),
                script: "echo \"Brainstorming beats...\"".to_string(),
                arguments: vec![],
                expose_via_mcp: true,
                is_placeholder: true,
                generate_slash_commands: false,
                slash_command_adapters: vec![],
                target_paths: vec![],
            },
        },
        TemplateCommand {
            template_id: "tmpl_repurpose_content".to_string(),
            theme: "Marketing".to_string(),
            metadata: CreateCommandInput {
                id: None,
                name: "Repurpose Content".to_string(),
                description: "Transforms long-form into social posts.".to_string(),
                script: "echo \"Repurposing content...\"".to_string(),
                arguments: vec![],
                expose_via_mcp: true,
                is_placeholder: true,
                generate_slash_commands: false,
                slash_command_adapters: vec![],
                target_paths: vec![],
            },
        },
        TemplateCommand {
            template_id: "tmpl_summarize_dataset".to_string(),
            theme: "Data Analysis".to_string(),
            metadata: CreateCommandInput {
                id: None,
                name: "Summarize Dataset".to_string(),
                description: "Executive summary of raw data.".to_string(),
                script: "echo \"Summarizing data...\"".to_string(),
                arguments: vec![],
                expose_via_mcp: true,
                is_placeholder: true,
                generate_slash_commands: false,
                slash_command_adapters: vec![],
                target_paths: vec![],
            },
        },
        TemplateCommand {
            template_id: "tmpl_extract_actions".to_string(),
            theme: "Admin".to_string(),
            metadata: CreateCommandInput {
                id: None,
                name: "Extract Action Items".to_string(),
                description: "Pulls actions from meeting transcripts.".to_string(),
                script: "echo \"Extracting actions...\"".to_string(),
                arguments: vec![],
                expose_via_mcp: true,
                is_placeholder: true,
                generate_slash_commands: false,
                slash_command_adapters: vec![],
                target_paths: vec![],
            },
        },
    ]
}
