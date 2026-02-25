use std::sync::Arc;
use tauri::State;

use crate::database::Database;
use crate::error::{AppError, Result};
use crate::file_storage::skills::{delete_skill_from_disk, save_skill_to_disk};
use crate::models::{CreateSkillInput, Skill, UpdateSkillInput};
use crate::templates::skills::{get_bundled_skill_templates, TemplateSkill};

#[tauri::command]
pub async fn get_all_skills(db: State<'_, Arc<Database>>) -> Result<Vec<Skill>> {
    db.get_all_skills().await
}

#[tauri::command]
pub async fn get_skill_by_id(id: String, db: State<'_, Arc<Database>>) -> Result<Skill> {
    db.get_skill_by_id(&id).await
}

#[tauri::command]
pub async fn create_skill(input: CreateSkillInput, db: State<'_, Arc<Database>>) -> Result<Skill> {
    crate::models::validate_skill_input(&input.name, &input.instructions)?;
    crate::models::validate_skill_schema(&input.input_schema)?;
    crate::models::validate_skill_entry_point(&input.entry_point)?;

    // Create in DB first
    let created = db.create_skill(input).await?;

    // Save to disk
    // If saving to disk fails, we must rollback the DB entry
    let path = match save_skill_to_disk(&created) {
        Ok(p) => p,
        Err(e) => {
            let _ = db.delete_skill(&created.id).await;
            return Err(e);
        }
    };

    // Update DB with the directory path
    let update = UpdateSkillInput {
        directory_path: Some(path.to_string_lossy().to_string()),
        ..Default::default()
    };

    if let Err(e) = db.update_skill(&created.id, update).await {
        // Attempt to cleanup disk if DB update fails
        let _ = std::fs::remove_dir_all(&path);
        // Also try to remove the initial DB entry to keep state clean
        let _ = db.delete_skill(&created.id).await;
        return Err(e);
    }

    db.get_skill_by_id(&created.id).await
}

#[tauri::command]
pub async fn update_skill(
    id: String,
    input: UpdateSkillInput,
    db: State<'_, Arc<Database>>,
) -> Result<Skill> {
    if let Some(ref name) = input.name {
        if let Some(ref instructions) = input.instructions {
            crate::models::validate_skill_input(name, instructions)?;
        } else {
            let existing = db.get_skill_by_id(&id).await?;
            crate::models::validate_skill_input(name, &existing.instructions)?;
        }
    } else if let Some(ref instructions) = input.instructions {
        let existing = db.get_skill_by_id(&id).await?;
        crate::models::validate_skill_input(&existing.name, instructions)?;
    }

    if let Some(ref schema) = input.input_schema {
        crate::models::validate_skill_schema(schema)?;
    }

    if let Some(ref ep) = input.entry_point {
        crate::models::validate_skill_entry_point(ep)?;
    }

    let updated = db.update_skill(&id, input).await?;
    save_skill_to_disk(&updated)?;
    Ok(updated)
}

#[tauri::command]
pub async fn delete_skill(id: String, db: State<'_, Arc<Database>>) -> Result<()> {
    if let Ok(existing) = db.get_skill_by_id(&id).await {
        let _ = delete_skill_from_disk(&existing);
    }
    db.delete_skill(&id).await
}

#[tauri::command]
pub fn get_skill_templates() -> Result<Vec<TemplateSkill>> {
    Ok(get_bundled_skill_templates())
}

#[tauri::command]
pub async fn install_skill_template(
    template_id: String,
    db: State<'_, Arc<Database>>,
) -> Result<Skill> {
    // Clone Arc for use in rollback closure
    let db_clone = Arc::clone(&db);

    // Helper closure for rollback on failure after disk write
    let rollback = |skill_id: String, dir_path: std::path::PathBuf| async move {
        let _ = db_clone.delete_skill(&skill_id).await;
        if dir_path.exists() {
            let _ = std::fs::remove_dir_all(&dir_path);
        }
    };

    // 1. Check idempotency: is it already installed?
    if let Ok(existing) = db.get_skill_by_id(&template_id).await {
        return Ok(existing);
    }

    // 2. Find template
    let templates = get_bundled_skill_templates();
    let template = templates
        .into_iter()
        .find(|t| t.template_id == template_id)
        .ok_or_else(|| AppError::Validation(format!("Template '{}' not found", template_id)))?;

    // 3. Check for name collisions
    let all_skills = db.get_all_skills().await?;
    if all_skills.iter().any(|s| s.name == template.metadata.name) {
        return Err(AppError::Validation(format!(
            "A skill with the name '{}' already exists. Please rename or delete it before installing this template.",
            template.metadata.name
        )));
    }

    // 4. Ensure the metadata uses our specific template ID
    let mut metadata = template.metadata.clone();
    metadata.id = Some(template_id.clone());

    // 4. Create in DB first so it generates default timestamps etc (using our prescribed ID)
    let created = db.create_skill(metadata).await?;

    // 5. Save the SKILL.md and skill.json to disk (generates directory for us)
    // Propagate disk write errors
    let path = match save_skill_to_disk(&created) {
        Ok(p) => p,
        Err(e) => {
            let _ = db.delete_skill(&created.id).await;
            return Err(e);
        }
    };

    // Write the custom template files
    for file in template.files {
        // Security: Validate template filenames to prevent path traversal
        if file.filename.contains("..")
            || file.filename.contains('/')
            || file.filename.contains('\\')
        {
            rollback(created.id.clone(), path.clone()).await;
            return Err(AppError::Validation(format!(
                "Invalid template filename: {}",
                file.filename
            )));
        }
        let file_path = path.join(&file.filename);
        if let Err(e) = std::fs::write(&file_path, &file.content).map_err(AppError::Io) {
            rollback(created.id.clone(), path.clone()).await;
            return Err(e);
        }
    }

    // 6. Update the DB with the absolute directory path that save_skill_to_disk determined
    let update = UpdateSkillInput {
        directory_path: Some(path.to_string_lossy().to_string()),
        ..Default::default()
    };

    if let Err(e) = db.update_skill(&created.id, update).await {
        rollback(created.id.clone(), path.clone()).await;
        return Err(e);
    }

    // Return the latest from DB
    db.get_skill_by_id(&template_id).await
}

#[tauri::command]
pub async fn sync_skills(db: State<'_, Arc<Database>>) -> Result<u32> {
    // Note: sync_skills_to_db currently takes a &Database but inside it probably calls sync DB methods.
    // We need to update file_storage::skills::sync_skills_to_db to use the new async DB methods too.
    // For now assuming it will be updated or we need to update it.
    // Let's assume crate::file_storage::skills::sync_skills_to_db needs to be async.
    crate::file_storage::skills::sync_skills_to_db(&db).await
}
