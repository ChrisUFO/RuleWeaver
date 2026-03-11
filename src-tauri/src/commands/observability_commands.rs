use std::path::PathBuf;
use std::sync::Arc;

use tauri::State;

use crate::database::Database;
use crate::error::Result;
use crate::models::{ObservabilityEvent, ObservabilityEventFilter};

#[tauri::command]
pub async fn list_observability_events(
    filter: Option<ObservabilityEventFilter>,
    db: State<'_, Arc<Database>>,
) -> Result<Vec<ObservabilityEvent>> {
    db.list_observability_events(&filter.unwrap_or_default())
        .await
}

#[tauri::command]
pub async fn export_observability_events(
    path: String,
    filter: Option<ObservabilityEventFilter>,
    selected_ids: Option<Vec<String>>,
    db: State<'_, Arc<Database>>,
) -> Result<usize> {
    crate::observability::export_events(
        db.inner().as_ref(),
        &PathBuf::from(path),
        selected_ids.as_deref(),
        &filter.unwrap_or_default(),
    )
    .await
}
