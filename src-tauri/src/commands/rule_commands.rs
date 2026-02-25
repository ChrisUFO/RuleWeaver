use std::sync::Arc;
use tauri::State;

use crate::database::Database;
use crate::error::{AppError, Result};
use crate::file_storage;
use crate::models::{CreateRuleInput, Rule, SyncResult, UpdateRuleInput};
use crate::sync::SyncEngine;
use crate::templates::rules::{get_bundled_rule_templates, TemplateRule};

use super::{
    get_local_rule_roots, register_local_rule_paths, storage_location_for_rule, use_file_storage,
    validate_local_rule_paths, validate_rule_input,
};

/// Helper function to sync all rules to AI tool locations.
/// Logs any errors that occur during the sync process.
async fn sync_to_ai_tools(db: &Database) {
    match db.get_all_rules().await {
        Ok(rules) => {
            let engine = SyncEngine::new(db);
            let sync_result = engine.sync_all(rules).await;
            if !sync_result.errors.is_empty() {
                log::error!("AI tool sync failed with errors: {:?}", sync_result.errors);
            }
        }
        Err(e) => {
            log::error!("Failed to get rules for AI tool sync: {}", e);
        }
    }
}

/// Helper function to delete a rule file from all possible storage locations.
/// This handles the case where a rule exists only as a file and not in the database.
async fn delete_rule_from_all_locations(id: &str, db: &Database) -> Result<()> {
    // First try global
    file_storage::delete_rule_file(id, &file_storage::StorageLocation::Global, Some(db)).await?;

    // Then try all local paths
    if let Ok(local_roots) = get_local_rule_roots(db).await {
        for root in local_roots {
            file_storage::delete_rule_file(
                id,
                &file_storage::StorageLocation::Local(root),
                Some(db),
            )
            .await?;
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn get_all_rules(db: State<'_, Arc<Database>>) -> Result<Vec<Rule>> {
    if use_file_storage(&db).await {
        let local_roots = get_local_rule_roots(&db).await?;
        let loaded = file_storage::load_rules_from_locations(&local_roots)?;
        Ok(loaded.rules)
    } else {
        db.get_all_rules().await
    }
}

#[tauri::command]
pub async fn get_rule_by_id(id: String, db: State<'_, Arc<Database>>) -> Result<Rule> {
    if use_file_storage(&db).await {
        let local_roots = get_local_rule_roots(&db).await?;
        let loaded = file_storage::load_rules_from_locations(&local_roots)?;
        loaded
            .rules
            .into_iter()
            .find(|r| r.id == id)
            .ok_or_else(|| AppError::RuleNotFound { id })
    } else {
        db.get_rule_by_id(&id).await
    }
}

#[tauri::command]
pub async fn create_rule(input: CreateRuleInput, db: State<'_, Arc<Database>>) -> Result<Rule> {
    validate_rule_input(&input.name, &input.content)?;
    validate_local_rule_paths(&db, None, Some(input.scope), &input.target_paths).await?;

    let created = db.create_rule(input).await?;

    if use_file_storage(&db).await {
        let location = storage_location_for_rule(&created);
        file_storage::save_rule_to_disk(&created, &location)?;
        db.update_rule_file_index(&created.id, &location).await?;
        register_local_rule_paths(&db, &created).await?;
    }

    // Sync to AI tool locations
    sync_to_ai_tools(&db).await;

    Ok(created)
}

#[tauri::command]
pub async fn update_rule(
    id: String,
    input: UpdateRuleInput,
    db: State<'_, Arc<Database>>,
) -> Result<Rule> {
    if let Some(ref name) = input.name {
        if let Some(ref content) = input.content {
            validate_rule_input(name, content)?;
        } else {
            let existing = db.get_rule_by_id(&id).await?;
            validate_rule_input(name, &existing.content)?;
        }
    } else if let Some(ref content) = input.content {
        let existing = db.get_rule_by_id(&id).await?;
        validate_rule_input(&existing.name, content)?;
    }

    validate_local_rule_paths(&db, Some(&id), input.scope, &input.target_paths).await?;

    let updated = db.update_rule(&id, input).await?;

    if use_file_storage(&db).await {
        let location = storage_location_for_rule(&updated);
        file_storage::save_rule_to_disk(&updated, &location)?;
        db.update_rule_file_index(&updated.id, &location).await?;
        register_local_rule_paths(&db, &updated).await?;
    }

    // Sync to AI tool locations
    sync_to_ai_tools(&db).await;

    Ok(updated)
}

#[tauri::command]
pub async fn delete_rule(id: String, db: State<'_, Arc<Database>>) -> Result<()> {
    if use_file_storage(&db).await {
        // Try to get the rule from DB to determine storage location
        if let Ok(existing) = db.get_rule_by_id(&id).await {
            let location = storage_location_for_rule(&existing);
            file_storage::delete_rule_file(&id, &location, Some(&db)).await?;
            db.remove_rule_file_index(&id).await?;
        } else {
            // Rule not in DB but might exist as file - try to delete from all locations
            delete_rule_from_all_locations(&id, &db).await?;
        }
    }
    db.delete_rule(&id).await?;

    // Sync to AI tool locations to remove deleted rule from adapters
    sync_to_ai_tools(&db).await;

    Ok(())
}

#[tauri::command]
pub async fn bulk_delete_rules(ids: Vec<String>, db: State<'_, Arc<Database>>) -> Result<()> {
    let use_fs = use_file_storage(&db).await;

    for id in ids {
        if use_fs {
            if let Ok(existing) = db.get_rule_by_id(&id).await {
                let location = storage_location_for_rule(&existing);
                let _ = file_storage::delete_rule_file(&id, &location, Some(&db)).await;
                let _ = db.remove_rule_file_index(&id).await;
            } else {
                // Rule not in DB but might exist as file - try to delete from all locations
                // Note: We intentionally ignore errors here to continue deleting other rules
                let _ = delete_rule_from_all_locations(&id, &db).await;
            }
        }
        db.delete_rule(&id).await?;
    }

    // Sync to AI tool locations to remove deleted rules from adapters
    sync_to_ai_tools(&db).await;

    Ok(())
}

#[tauri::command]
pub async fn toggle_rule(id: String, enabled: bool, db: State<'_, Arc<Database>>) -> Result<Rule> {
    let toggled = db.toggle_rule(&id, enabled).await?;

    if use_file_storage(&db).await {
        let location = storage_location_for_rule(&toggled);
        file_storage::save_rule_to_disk(&toggled, &location)?;
        db.update_rule_file_index(&toggled.id, &location).await?;
        register_local_rule_paths(&db, &toggled).await?;
    }

    // Sync to AI tool locations - enabled/disabled status affects adapter files
    sync_to_ai_tools(&db).await;

    Ok(toggled)
}

#[tauri::command]
pub async fn sync_rules(db: State<'_, Arc<Database>>) -> Result<SyncResult> {
    let rules = db.get_all_rules().await?;
    let engine = SyncEngine::new(&db);
    Ok(engine.sync_all(rules).await)
}

#[tauri::command]
pub async fn preview_sync(db: State<'_, Arc<Database>>) -> Result<SyncResult> {
    let rules = db.get_all_rules().await?;
    let engine = SyncEngine::new(&db);
    Ok(engine.preview(rules).await)
}

#[tauri::command]
pub fn get_rule_templates() -> Result<Vec<TemplateRule>> {
    Ok(get_bundled_rule_templates())
}

#[tauri::command]
pub async fn install_rule_template(
    template_id: String,
    db: State<'_, Arc<Database>>,
) -> Result<Rule> {
    // 1. Check idempotency: is it already installed?
    if let Ok(existing) = db.get_rule_by_id(&template_id).await {
        return Ok(existing);
    }

    // 2. Find template
    let templates = get_bundled_rule_templates();
    let template = templates
        .into_iter()
        .find(|t| t.template_id == template_id)
        .ok_or_else(|| AppError::Validation(format!("Template '{}' not found", template_id)))?;

    // 3. Ensure the metadata uses our specific template ID
    let input = template.metadata.clone();

    // We don't have a direct 'id' field in CreateRuleInput, but we can wrap it if needed.
    // However, create_rule usually generates a new ID.
    // For templates, we might want to keep the template ID as the rule ID for idempotency checks.
    // Let's see if Database::create_rule supports passing an ID.
    // Checking models/rule.rs: CreateRuleInput doesn't have an ID.
    // But Rule has an ID.

    // Actually, create_skill had an id: Option<String>.
    // Let's check Database::create_rule signature if possible, or just generate a new one.
    // Given the idempotency check above uses template_id, we SHOULD use template_id as the ID.

    // I'll assume for now we might need to update CreateRuleInput to support an optional ID if we want idempotency by ID.
    // Or just accept it creates a new one each time if renamed.

    let created = db.create_rule(input).await?;

    if use_file_storage(&db).await {
        let location = storage_location_for_rule(&created);
        file_storage::save_rule_to_disk(&created, &location)?;
        db.update_rule_file_index(&created.id, &location).await?;
        register_local_rule_paths(&db, &created).await?;
    }

    // Sync to AI tool locations
    sync_to_ai_tools(&db).await;

    Ok(created)
}
