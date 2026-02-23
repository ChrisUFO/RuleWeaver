use std::sync::Arc;
use tauri::State;

use crate::database::Database;
use crate::error::{AppError, Result};
use crate::file_storage::skills::{
    delete_skill_from_disk, load_skills_from_disk, save_skill_to_disk,
};
use crate::models::{CreateSkillInput, Skill, UpdateSkillInput};
use crate::templates::skills::{get_bundled_skill_templates, TemplateSkill};

use super::{validate_skill_entry_point, validate_skill_input, validate_skill_schema};

#[tauri::command]
pub fn get_all_skills(db: State<'_, Arc<Database>>) -> Result<Vec<Skill>> {
    // For now, load from disk directly to act as source of truth, but we could also return DB
    if let Ok(skills) = load_skills_from_disk() {
        Ok(skills)
    } else {
        db.get_all_skills()
    }
}

#[tauri::command]
pub fn get_skill_by_id(id: String, db: State<'_, Arc<Database>>) -> Result<Skill> {
    if let Ok(skills) = load_skills_from_disk() {
        if let Some(skill) = skills.into_iter().find(|s| s.id == id) {
            return Ok(skill);
        }
    }
    db.get_skill_by_id(&id)
}

#[tauri::command]
pub fn create_skill(input: CreateSkillInput, db: State<'_, Arc<Database>>) -> Result<Skill> {
    validate_skill_input(&input.name, &input.instructions)?;
    validate_skill_schema(&input.input_schema)?;
    validate_skill_entry_point(&input.entry_point)?;

    // Create in DB first to get ID and defaults
    let created = db.create_skill(input)?;

    // Save to disk
    if let Ok(path) = save_skill_to_disk(&created) {
        // Update DB with the directory path we just resolved
        let update = UpdateSkillInput {
            directory_path: Some(path.to_string_lossy().to_string()),
            ..Default::default()
        };
        db.update_skill(&created.id, update)?;
    }

    db.get_skill_by_id(&created.id)
}

#[tauri::command]
pub fn update_skill(
    id: String,
    input: UpdateSkillInput,
    db: State<'_, Arc<Database>>,
) -> Result<Skill> {
    if let Some(ref name) = input.name {
        if let Some(ref instructions) = input.instructions {
            validate_skill_input(name, instructions)?;
        } else {
            let existing = db.get_skill_by_id(&id)?;
            validate_skill_input(name, &existing.instructions)?;
        }
    } else if let Some(ref instructions) = input.instructions {
        let existing = db.get_skill_by_id(&id)?;
        validate_skill_input(&existing.name, instructions)?;
    }

    if let Some(ref schema) = input.input_schema {
        validate_skill_schema(schema)?;
    }

    if let Some(ref ep) = input.entry_point {
        validate_skill_entry_point(ep)?;
    }

    let updated = db.update_skill(&id, input)?;
    let _ = save_skill_to_disk(&updated);
    Ok(updated)
}

#[tauri::command]
pub fn delete_skill(id: String, db: State<'_, Arc<Database>>) -> Result<()> {
    if let Ok(existing) = db.get_skill_by_id(&id) {
        let _ = delete_skill_from_disk(&existing);
    }
    db.delete_skill(&id)
}

#[tauri::command]
pub fn get_skill_templates() -> Result<Vec<TemplateSkill>> {
    Ok(get_bundled_skill_templates())
}

#[tauri::command]
pub fn install_skill_template(template_id: String, db: State<'_, Arc<Database>>) -> Result<Skill> {
    // 1. Check idempotency: is it already installed?
    if let Ok(existing) = db.get_skill_by_id(&template_id) {
        return Ok(existing);
    }

    // 2. Find template
    let templates = get_bundled_skill_templates();
    let template = templates
        .into_iter()
        .find(|t| t.template_id == template_id)
        .ok_or_else(|| AppError::Validation(format!("Template '{}' not found", template_id)))?;

    // 3. Ensure the metadata uses our specific template ID
    let mut metadata = template.metadata.clone();
    metadata.id = Some(template_id.clone());

    // 4. Create in DB first so it generates default timestamps etc (using our prescribed ID)
    let created = db.create_skill(metadata)?;

    // 5. Save the SKILL.md and skill.json to disk (generates directory for us)
    match save_skill_to_disk(&created) {
        Ok(path) => {
            // Write the custom template files
            for file in template.files {
                let file_path = path.join(&file.filename);
                if let Err(e) = std::fs::write(&file_path, &file.content) {
                    // Rollback DB entry if file write fails
                    let _ = db.delete_skill(&created.id);
                    return Err(AppError::Io(e));
                }
            }

            // 6. Update the DB with the absolute directory path that save_skill_to_disk determined
            let update = UpdateSkillInput {
                directory_path: Some(path.to_string_lossy().to_string()),
                ..Default::default()
            };
            if let Err(e) = db.update_skill(&created.id, update) {
                // Rollback DB entry and disk if update fails
                let _ = delete_skill_from_disk(&created);
                let _ = db.delete_skill(&created.id);
                return Err(e);
            }
        }
        Err(e) => {
            // Rollback DB entry if disk setup fails
            let _ = db.delete_skill(&created.id);
            return Err(e);
        }
    }

    // Return the latest from DB
    db.get_skill_by_id(&template_id)
}
