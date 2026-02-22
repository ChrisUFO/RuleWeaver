use std::sync::Arc;
use tauri::State;

use crate::database::Database;
use crate::error::Result;
use crate::models::{CreateSkillInput, Skill, UpdateSkillInput};

use super::validate_skill_input;

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
    validate_skill_input(&input.name, &input.instructions)?;
    db.create_skill(input)
}

#[tauri::command]
pub fn update_skill(id: String, input: UpdateSkillInput, db: State<'_, Arc<Database>>) -> Result<Skill> {
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

    db.update_skill(&id, input)
}

#[tauri::command]
pub fn delete_skill(id: String, db: State<'_, Arc<Database>>) -> Result<()> {
    db.delete_skill(&id)
}
