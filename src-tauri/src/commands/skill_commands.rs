use std::sync::Arc;
use tauri::State;

use crate::database::Database;
use crate::error::{AppError, Result};
use crate::file_storage::skills::{delete_skill_from_disk, save_skill_to_disk};
use crate::models::{CreateSkillInput, Skill, UpdateSkillInput};
use crate::templates::skills::{get_bundled_skill_templates, TemplateSkill};

#[tauri::command]
pub fn get_all_skills(db: State<'_, Arc<Database>>) -> Result<Vec<Skill>> {
    db.get_all_skills()
}

#[tauri::command]
pub fn get_skill_by_id(id: String, db: State<'_, Arc<Database>>) -> Result<Skill> {
    db.get_skill_by_id(&id)
}

#[tauri::command]
pub fn create_skill(input: CreateSkillInput, db: State<'_, Arc<Database>>) -> Result<Skill> {
    crate::models::validate_skill_input(&input.name, &input.instructions)?;
    crate::models::validate_skill_schema(&input.input_schema)?;
    crate::models::validate_skill_entry_point(&input.entry_point)?;

    // Create in DB first
    let created = db.create_skill(input)?;

    // Atomic Cleanup Guard for DB
    struct SkillCreationGuard<'a> {
        db: &'a Database,
        skill_id: String,
        defused: bool,
    }

    impl<'a> Drop for SkillCreationGuard<'a> {
        fn drop(&mut self) {
            if !self.defused {
                let _ = self.db.delete_skill(&self.skill_id);
            }
        }
    }

    let mut guard = SkillCreationGuard {
        db: &db,
        skill_id: created.id.clone(),
        defused: false,
    };

    // Save to disk
    let path = match save_skill_to_disk(&created) {
        Ok(p) => p,
        Err(e) => return Err(e), // Guard will drop and delete from DB
    };

    // Update DB with the directory path
    let update = UpdateSkillInput {
        directory_path: Some(path.to_string_lossy().to_string()),
        ..Default::default()
    };

    if let Err(e) = db.update_skill(&created.id, update) {
        // Attempt to cleanup disk if DB update fails
        let _ = std::fs::remove_dir_all(&path);
        return Err(e);
    }

    guard.defused = true;
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
            crate::models::validate_skill_input(name, instructions)?;
        } else {
            let existing = db.get_skill_by_id(&id)?;
            crate::models::validate_skill_input(name, &existing.instructions)?;
        }
    } else if let Some(ref instructions) = input.instructions {
        let existing = db.get_skill_by_id(&id)?;
        crate::models::validate_skill_input(&existing.name, instructions)?;
    }

    if let Some(ref schema) = input.input_schema {
        crate::models::validate_skill_schema(schema)?;
    }

    if let Some(ref ep) = input.entry_point {
        crate::models::validate_skill_entry_point(ep)?;
    }

    let updated = db.update_skill(&id, input)?;
    save_skill_to_disk(&updated)?;
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

    // Atomic Cleanup Guard
    struct SkillInstallationGuard<'a> {
        db: &'a Database,
        skill_id: String,
        directory_path: Option<std::path::PathBuf>,
        defused: bool,
    }

    impl<'a> Drop for SkillInstallationGuard<'a> {
        fn drop(&mut self) {
            if !self.defused {
                // Rollback DB
                let _ = self.db.delete_skill(&self.skill_id);
                // Rollback Disk
                if let Some(ref path) = self.directory_path {
                    if path.exists() {
                        let _ = std::fs::remove_dir_all(path);
                    }
                }
            }
        }
    }

    let mut guard = SkillInstallationGuard {
        db: &db,
        skill_id: created.id.clone(),
        directory_path: None,
        defused: false,
    };

    // 5. Save the SKILL.md and skill.json to disk (generates directory for us)
    // Propagate disk write errors
    let path = save_skill_to_disk(&created)?;
    guard.directory_path = Some(path.clone());

    // Write the custom template files
    for file in template.files {
        // Security: Validate template filenames to prevent path traversal
        if file.filename.contains("..")
            || file.filename.contains('/')
            || file.filename.contains('\\')
        {
            return Err(AppError::Validation(format!(
                "Invalid template filename: {}",
                file.filename
            )));
        }
        let file_path = path.join(&file.filename);
        std::fs::write(&file_path, &file.content).map_err(AppError::Io)?;
    }

    // 6. Update the DB with the absolute directory path that save_skill_to_disk determined
    let update = UpdateSkillInput {
        directory_path: Some(path.to_string_lossy().to_string()),
        ..Default::default()
    };
    db.update_skill(&created.id, update)?;

    // Success! Defuse the guard
    guard.defused = true;

    // Return the latest from DB
    db.get_skill_by_id(&template_id)
}

#[tauri::command]
pub fn sync_skills(db: State<'_, Arc<Database>>) -> Result<u32> {
    crate::file_storage::skills::sync_skills_to_db(&db)
}
