mod commands;
mod constants;
mod database;
mod error;
mod execution;
mod file_storage;
mod mcp;
mod models;
mod sync;
pub mod templates;

use database::Database;
use mcp::McpManager;
use std::sync::Arc;
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::Manager;

const MINIMIZE_TO_TRAY_KEY: &str = "minimize_to_tray";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    log::info!("RuleWeaver application initializing");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let db = Arc::new(Database::new(app.handle())?);

            // Sync skills to database on startup
            if let Err(e) = crate::file_storage::skills::sync_skills_to_db(&db) {
                log::error!("Failed to sync skills on startup: {}", e);
            }

            let mcp_manager = McpManager::new(crate::constants::DEFAULT_MCP_PORT);

            let auto_start_mcp = db
                .get_setting("mcp_auto_start")
                .ok()
                .flatten()
                .map(|v| v == "true")
                .unwrap_or(false);

            if auto_start_mcp {
                let mcp_for_setup = mcp_manager.clone();
                let db_for_setup = Arc::clone(&db);
                tauri::async_runtime::spawn(async move {
                    let _ = mcp_for_setup.start(&db_for_setup).await;
                });
            }

            if db.get_setting(MINIMIZE_TO_TRAY_KEY)?.is_none() {
                db.set_setting(MINIMIZE_TO_TRAY_KEY, "true")?;
            }

            let show = MenuItemBuilder::with_id("show", "Show RuleWeaver").build(app)?;
            let hide = MenuItemBuilder::with_id("hide", "Hide to Tray").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "Quit RuleWeaver").build(app)?;
            let tray_menu = MenuBuilder::new(app)
                .item(&show)
                .item(&hide)
                .separator()
                .item(&quit)
                .build()?;

            let app_handle = app.handle().clone();
            TrayIconBuilder::new()
                .menu(&tray_menu)
                .on_menu_event(move |app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                    "hide" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.hide();
                        }
                    }
                    "quit" => {
                        if let Some(mcp) = app.try_state::<McpManager>() {
                            let mcp_clone = mcp.inner().clone();
                            tauri::async_runtime::spawn(async move {
                                let _ = mcp_clone.stop().await;
                            });
                        }
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(move |tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(true) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.unminimize();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            if let Some(window) = app_handle.get_webview_window("main") {
                let app_for_events = app_handle.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let minimize_to_tray = app_for_events
                            .try_state::<Arc<Database>>()
                            .and_then(|db| db.get_setting(MINIMIZE_TO_TRAY_KEY).ok().flatten())
                            .map(|v| v == "true")
                            .unwrap_or(true);

                        if minimize_to_tray {
                            if let Some(main) = app_for_events.get_webview_window("main") {
                                let _ = main.hide();
                            }
                        } else {
                            if let Some(mcp) = app_for_events.try_state::<McpManager>() {
                                let mcp_clone = mcp.inner().clone();
                                tauri::async_runtime::spawn(async move {
                                    let _ = mcp_clone.stop().await;
                                });
                            }
                            app_for_events.exit(0);
                        }
                    }
                });
            }

            app.manage(Arc::clone(&db));
            app.manage(mcp_manager);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_all_rules,
            commands::get_rule_by_id,
            commands::create_rule,
            commands::update_rule,
            commands::delete_rule,
            commands::toggle_rule,
            commands::sync_rules,
            commands::preview_sync,
            commands::get_sync_history,
            commands::get_app_data_path_cmd,
            commands::open_in_explorer,
            commands::read_file_content,
            commands::resolve_conflict,
            commands::get_app_version,
            commands::get_setting,
            commands::set_setting,
            commands::get_all_settings,
            commands::migrate_to_file_storage,
            commands::rollback_file_migration,
            commands::verify_file_migration,
            commands::get_file_migration_progress,
            commands::get_storage_info,
            commands::get_storage_mode,
            commands::get_all_commands,
            commands::get_command_by_id,
            commands::create_command,
            commands::update_command,
            commands::delete_command,
            commands::test_command,
            commands::sync_commands,
            commands::get_all_skills,
            commands::get_skill_by_id,
            commands::create_skill,
            commands::update_skill,
            commands::delete_skill,
            commands::get_skill_templates,
            commands::install_skill_template,
            commands::get_mcp_status,
            commands::start_mcp_server,
            commands::stop_mcp_server,
            commands::restart_mcp_server,
            commands::get_mcp_connection_instructions,
            commands::get_mcp_logs,
            commands::get_execution_history,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

pub fn run_mcp_cli(port: u16, token: Option<String>) -> std::result::Result<(), String> {
    let db = Arc::new(Database::new_for_cli().map_err(|e| e.to_string())?);
    let manager = McpManager::new(port);

    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    rt.block_on(async {
        if let Some(t) = token {
            manager.set_api_token(t).await;
        }

        manager.start(&db).await.map_err(|e| e.to_string())?;
        manager
            .wait_until_stopped()
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    })
}
