use std::sync::Arc;
use tauri::State;

use crate::database::{Database, ReconciliationLogEntry};
use crate::error::Result;
use crate::reconciliation::{ReconcilePlan, ReconcileResult, ReconciliationEngine};

#[tauri::command]
pub async fn reconcile_all(db: State<'_, Arc<Database>>, dry_run: bool) -> Result<ReconcileResult> {
    let engine = ReconciliationEngine::new(db.inner().clone())?;
    engine.reconcile(dry_run).await
}

#[tauri::command]
pub async fn reconcile_preview(db: State<'_, Arc<Database>>) -> Result<ReconcilePlan> {
    let engine = ReconciliationEngine::new(db.inner().clone())?;
    let desired = engine.compute_desired_state().await?;
    let actual = engine.scan_actual_state().await?;
    Ok(engine.plan(&desired, &actual))
}

#[tauri::command]
pub async fn get_reconciliation_logs(
    db: State<'_, Arc<Database>>,
    limit: i64,
) -> Result<Vec<ReconciliationLogEntry>> {
    db.get_reconciliation_logs(limit).await
}

#[tauri::command]
pub async fn clear_reconciliation_logs(db: State<'_, Arc<Database>>) -> Result<()> {
    db.clear_reconciliation_logs().await
}
