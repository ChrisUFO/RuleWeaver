use std::sync::Arc;
use tauri::State;

use crate::database::Database;
use crate::error::Result;
use crate::mcp::{McpConnectionInstructions, McpManager, McpStatus};

#[tauri::command]
pub async fn get_mcp_status(mcp: State<'_, McpManager>) -> Result<McpStatus> {
    mcp.status().await
}

#[tauri::command]
pub async fn start_mcp_server(db: State<'_, Arc<Database>>, mcp: State<'_, McpManager>) -> Result<()> {
    mcp.start(&db).await
}

#[tauri::command]
pub async fn stop_mcp_server(mcp: State<'_, McpManager>) -> Result<()> {
    mcp.stop().await
}

#[tauri::command]
pub async fn restart_mcp_server(db: State<'_, Arc<Database>>, mcp: State<'_, McpManager>) -> Result<()> {
    mcp.stop().await?;
    mcp.start(&db).await
}

#[tauri::command]
pub async fn get_mcp_connection_instructions(
    mcp: State<'_, McpManager>,
) -> Result<McpConnectionInstructions> {
    mcp.instructions().await
}

#[tauri::command]
pub async fn get_mcp_logs(limit: Option<u32>, mcp: State<'_, McpManager>) -> Result<Vec<String>> {
    mcp.logs(limit.unwrap_or(50) as usize).await
}
