use crate::error::Result;
use crate::models::registry::{ToolEntry, REGISTRY};

#[tauri::command]
pub fn get_tool_registry() -> Result<Vec<ToolEntry>> {
    Ok(REGISTRY.all().into_iter().cloned().collect())
}
