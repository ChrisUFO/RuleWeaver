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
pub async fn start_mcp_server(
    app: tauri::AppHandle,
    db: State<'_, Arc<Database>>,
    mcp: State<'_, McpManager>,
    status: State<'_, crate::GlobalStatus>,
) -> Result<()> {
    mcp.set_app_handle(app).await;
    match mcp.start(&db).await {
        Ok(_) => {
            status.update_mcp_status(&format!("Running (Port {})", mcp.port()));
            Ok(())
        }
        Err(e) => {
            status.update_mcp_status("Error Starting");
            Err(e)
        }
    }
}

#[tauri::command]
pub async fn stop_mcp_server(
    mcp: State<'_, McpManager>,
    status: State<'_, crate::GlobalStatus>,
) -> Result<()> {
    match mcp.stop().await {
        Ok(_) => {
            status.update_mcp_status("Stopped");
            Ok(())
        }
        Err(e) => {
            status.update_mcp_status("Error Stopping");
            Err(e)
        }
    }
}

#[tauri::command]
pub async fn restart_mcp_server(
    app: tauri::AppHandle,
    db: State<'_, Arc<Database>>,
    mcp: State<'_, McpManager>,
    status: State<'_, crate::GlobalStatus>,
) -> Result<()> {
    let _ = mcp.stop().await;
    mcp.set_app_handle(app).await;
    match mcp.start(&db).await {
        Ok(_) => {
            status.update_mcp_status(&format!("Running (Port {})", mcp.port()));
            Ok(())
        }
        Err(e) => {
            status.update_mcp_status("Error Restarting");
            Err(e)
        }
    }
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
