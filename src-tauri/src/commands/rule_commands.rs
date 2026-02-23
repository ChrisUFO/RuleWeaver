use std::sync::Arc;
use tauri::State;

use crate::database::Database;
use crate::error::{AppError, Result};
use crate::file_storage;
use crate::models::{CreateRuleInput, Rule, SyncResult, UpdateRuleInput};
use crate::sync::SyncEngine;

use super::{
    get_local_rule_roots, register_local_rule_paths, storage_location_for_rule, use_file_storage,
    validate_local_rule_paths, validate_rule_input,
};

#[tauri::command]
pub fn get_all_rules(db: State<'_, Arc<Database>>) -> Result<Vec<Rule>> {
    if use_file_storage(&db) {
        let local_roots = get_local_rule_roots(&db)?;
        let loaded = file_storage::load_rules_from_locations(&local_roots)?;
        Ok(loaded.rules)
    } else {
        db.get_all_rules()
    }
}

#[tauri::command]
pub fn get_rule_by_id(id: String, db: State<'_, Arc<Database>>) -> Result<Rule> {
    if use_file_storage(&db) {
        let local_roots = get_local_rule_roots(&db)?;
        let loaded = file_storage::load_rules_from_locations(&local_roots)?;
        loaded
            .rules
            .into_iter()
            .find(|r| r.id == id)
            .ok_or_else(|| AppError::RuleNotFound { id })
    } else {
        db.get_rule_by_id(&id)
    }
}

#[tauri::command]
pub fn create_rule(input: CreateRuleInput, db: State<'_, Arc<Database>>) -> Result<Rule> {
    validate_rule_input(&input.name, &input.content)?;
    validate_local_rule_paths(&db, None, Some(input.scope), &input.target_paths)?;

    let created = db.create_rule(input)?;

    if use_file_storage(&db) {
        let location = storage_location_for_rule(&created);
        file_storage::save_rule_to_disk(&created, &location)?;
        db.update_rule_file_index(&created.id, &location)?;
        register_local_rule_paths(&db, &created)?;
    }

    Ok(created)
}

#[tauri::command]
pub fn update_rule(
    id: String,
    input: UpdateRuleInput,
    db: State<'_, Arc<Database>>,
) -> Result<Rule> {
    if let Some(ref name) = input.name {
        if let Some(ref content) = input.content {
            validate_rule_input(name, content)?;
        } else {
            let existing = db.get_rule_by_id(&id)?;
            validate_rule_input(name, &existing.content)?;
        }
    } else if let Some(ref content) = input.content {
        let existing = db.get_rule_by_id(&id)?;
        validate_rule_input(&existing.name, content)?;
    }

    validate_local_rule_paths(&db, Some(&id), input.scope, &input.target_paths)?;

    let updated = db.update_rule(&id, input)?;

    if use_file_storage(&db) {
        let location = storage_location_for_rule(&updated);
        file_storage::save_rule_to_disk(&updated, &location)?;
        db.update_rule_file_index(&updated.id, &location)?;
        register_local_rule_paths(&db, &updated)?;
    }

    Ok(updated)
}

#[tauri::command]
pub fn delete_rule(id: String, db: State<'_, Arc<Database>>) -> Result<()> {
    if use_file_storage(&db) {
        if let Ok(existing) = db.get_rule_by_id(&id) {
            let location = storage_location_for_rule(&existing);
            file_storage::delete_rule_file(&id, &location, Some(&db))?;
            db.remove_rule_file_index(&id)?;
        }
    }
    db.delete_rule(&id)
}

#[tauri::command]
pub fn toggle_rule(id: String, enabled: bool, db: State<'_, Arc<Database>>) -> Result<Rule> {
    let toggled = db.toggle_rule(&id, enabled)?;

    if use_file_storage(&db) {
        let location = storage_location_for_rule(&toggled);
        file_storage::save_rule_to_disk(&toggled, &location)?;
        db.update_rule_file_index(&toggled.id, &location)?;
        register_local_rule_paths(&db, &toggled)?;
    }

    Ok(toggled)
}

#[tauri::command]
pub fn sync_rules(db: State<'_, Arc<Database>>) -> Result<SyncResult> {
    let rules = db.get_all_rules()?;
    let engine = SyncEngine::new(&db);
    Ok(engine.sync_all(rules))
}

#[tauri::command]
pub fn preview_sync(db: State<'_, Arc<Database>>) -> Result<SyncResult> {
    let rules = db.get_all_rules()?;
    let engine = SyncEngine::new(&db);
    Ok(engine.preview(rules))
}
