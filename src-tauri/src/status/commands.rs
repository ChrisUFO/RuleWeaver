//! Tauri commands for the unified artifact status system.

use std::sync::Arc;
use tauri::State;

use crate::database::Database;
use crate::error::Result;
use crate::status::{ArtifactStatusEntry, RepairResult, StatusEngine, StatusFilter, StatusSummary};

#[tauri::command]
pub async fn get_artifact_status(
    db: State<'_, Arc<Database>>,
    filter: Option<StatusFilter>,
) -> Result<Vec<ArtifactStatusEntry>> {
    let engine = StatusEngine::new(db.inner().clone())?;
    let filter = filter.unwrap_or_default();
    engine.compute_status(&filter).await
}

#[tauri::command]
pub async fn get_artifact_status_summary(
    db: State<'_, Arc<Database>>,
    filter: Option<StatusFilter>,
) -> Result<StatusSummary> {
    let engine = StatusEngine::new(db.inner().clone())?;
    let filter = filter.unwrap_or_default();
    engine.get_summary(&filter).await
}

#[tauri::command]
pub async fn repair_artifact(
    db: State<'_, Arc<Database>>,
    entry_id: String,
) -> Result<RepairResult> {
    let engine = StatusEngine::new(db.inner().clone())?;
    engine.repair_artifact(&entry_id).await
}

#[tauri::command]
pub async fn repair_all_artifacts(
    db: State<'_, Arc<Database>>,
    filter: Option<StatusFilter>,
) -> Result<Vec<RepairResult>> {
    let engine = StatusEngine::new(db.inner().clone())?;
    let filter = filter.unwrap_or_default();
    engine.repair_all_artifacts(&filter).await
}

#[tauri::command]
pub async fn refresh_artifact_status(
    db: State<'_, Arc<Database>>,
    filter: Option<StatusFilter>,
) -> Result<Vec<ArtifactStatusEntry>> {
    let engine = StatusEngine::new(db.inner().clone())?;
    let filter = filter.unwrap_or_default();
    engine.compute_status(&filter).await
}
