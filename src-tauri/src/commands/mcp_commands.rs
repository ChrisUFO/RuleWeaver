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
    db: State<'_, Arc<Database>>,
    mcp: State<'_, McpManager>,
    status: State<'_, crate::GlobalStatus>,
) -> Result<()> {
    mcp.start(&db).await?;
    {
        *status.mcp_status.lock() = format!("Running (Port {})", mcp.port());
        status.update_tray();
    }
    Ok(())
}

#[tauri::command]
pub async fn stop_mcp_server(
    mcp: State<'_, McpManager>,
    status: State<'_, crate::GlobalStatus>,
) -> Result<()> {
    mcp.stop().await?;
    {
        *status.mcp_status.lock() = "Stopped".to_string();
        status.update_tray();
    }
    Ok(())
}

#[tauri::command]
pub async fn restart_mcp_server(
    db: State<'_, Arc<Database>>,
    mcp: State<'_, McpManager>,
    status: State<'_, crate::GlobalStatus>,
) -> Result<()> {
    mcp.stop().await?;
    mcp.start(&db).await?;
    {
        *status.mcp_status.lock() = format!("Running (Port {})", mcp.port());
        status.update_tray();
    }
    Ok(())
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
