mod commands;
mod constants;
mod database;
mod error;
mod execution;
mod file_storage;
mod mcp;
mod models;
mod rule_import;
mod slash_commands;
mod sync;
pub mod templates;

use database::Database;
use file_storage::RuleFileWatcher;
use mcp::McpManager;
use std::sync::Arc;
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Emitter, Manager};

const MINIMIZE_TO_TRAY_KEY: &str = "minimize_to_tray";

pub struct WatcherState(pub RuleFileWatcher);

#[derive(Default)]
pub struct GlobalStatus {
    pub sync_status: parking_lot::Mutex<String>,
    pub mcp_status: parking_lot::Mutex<String>,
    pub menu: parking_lot::Mutex<Option<tauri::menu::Menu<tauri::Wry>>>,
}

impl GlobalStatus {
    pub fn update_tray(&self) {
        let sync = match self.sync_status.try_lock() {
            Some(s) => s.clone(),
            None => return,
        };
        let mcp = match self.mcp_status.try_lock() {
            Some(s) => s.clone(),
            None => return,
        };

        if let Some(menu) = self.menu.lock().as_ref() {
            if let Ok(items) = menu.items() {
                for item in items {
                    if let Some(menu_item) = item.as_menuitem() {
                        if menu_item.id().as_ref() == "status" {
                            let _ = menu_item.set_text(format!("Status: {}", sync));
                        } else if menu_item.id().as_ref() == "mcp_info" {
                            let _ = menu_item.set_text(format!("MCP: {}", mcp));
                        }
                    }
                }
            }
        }
    }

    pub fn update_mcp_status(&self, status: &str) {
        {
            *self.mcp_status.lock() = status.to_string();
        }
        self.update_tray();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    log::info!("RuleWeaver application initializing");

    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Initialize database asynchronously blocking the setup
            let db = tauri::async_runtime::block_on(async {
                let db = Arc::new(Database::new(app.handle()).await?);

                // Sync skills to database on startup
                // Note: sync_skills_to_db likely needs to be async or internal calls do.
                // Assuming sync_skills_to_db handles sync logic, we might need to update it.
                // For now, if it uses DB it must be awaited if we made DB async.
                // Checking previous code: sync_skills calls sync_skills_to_db(&db).await.
                // So sync_skills_to_db must be async.
                if let Err(e) = crate::file_storage::skills::sync_skills_to_db(&db).await {
                    log::error!("Failed to sync skills on startup: {}", e);
                    let _ = app.emit("startup-sync-error", e.to_string());
                }

                // Migrate legacy configurations
                if let Err(e) = crate::sync::check_and_migrate_legacy_paths() {
                    log::error!("Failed to migrate legacy paths: {}", e);
                }

                // First-run bootstrap import from existing AI tool files
                let bootstrap_done = db
                    .get_setting("ai_tool_import_bootstrap_done")
                    .await
                    .ok()
                    .flatten()
                    .map(|v| v == "true")
                    .unwrap_or(false);

                if !bootstrap_done {
                    let mut mark_bootstrap_done = false;
                    let options = crate::models::ImportExecutionOptions {
                        conflict_mode: crate::models::ImportConflictMode::Rename,
                        ..Default::default()
                    };
                    let max_size = crate::rule_import::resolve_max_size(&options);
                    match crate::rule_import::scan_ai_tool_candidates(&db, max_size).await {
                        Ok(scan) => {
                            if scan.candidates.is_empty() {
                                mark_bootstrap_done = true;
                            } else {
                                match crate::rule_import::execute_import(&db, scan, options).await {
                                    Ok(import_result) => {
                                        mark_bootstrap_done = true;
                                        log::info!(
                                            "Bootstrap import complete: {} imported, {} skipped, {} conflicts",
                                            import_result.imported.len(),
                                            import_result.skipped.len(),
                                            import_result.conflicts.len()
                                        );

                                        if import_result.imported.len()
                                            + import_result.skipped.len()
                                            + import_result.conflicts.len()
                                            > 0
                                        {
                                            use tauri_plugin_notification::NotificationExt;
                                            app.notification()
                                                .builder()
                                                .title("Existing Rules Imported")
                                                .body(format!(
                                                    "Imported {} rule(s), skipped {}, conflicts {}",
                                                    import_result.imported.len(),
                                                    import_result.skipped.len(),
                                                    import_result.conflicts.len()
                                                ))
                                                .show()
                                                .ok();
                                        }
                                    }
                                    Err(e) => {
                                        log::error!("Bootstrap import failed: {}", e);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            log::error!("Bootstrap import scan failed: {}", e);
                        }
                    }
                    if mark_bootstrap_done {
                        if let Err(e) = db.set_setting("ai_tool_import_bootstrap_done", "true").await {
                            log::error!("Failed to persist bootstrap import flag: {}", e);
                        }
                    }
                }
                
                Ok::<_, Box<dyn std::error::Error>>(db)
            })?;

            let watcher = RuleFileWatcher::new();
            let mcp_manager = McpManager::new(crate::constants::DEFAULT_MCP_PORT);

            // Need to block on getting settings for initial setup
            let (auto_start_mcp, minimize_to_tray, storage_mode) = tauri::async_runtime::block_on(async {
                let auto = db
                    .get_setting("mcp_auto_start")
                    .await
                    .ok()
                    .flatten()
                    .map(|v| v == "true")
                    .unwrap_or(false);
                
                let min = db.get_setting(MINIMIZE_TO_TRAY_KEY).await.ok().flatten();
                if min.is_none() {
                    db.set_setting(MINIMIZE_TO_TRAY_KEY, "true").await?;
                }
                
                let storage = db.get_storage_mode().await?;
                Ok::<_, crate::error::AppError>((auto, min, storage))
            })?;

            if auto_start_mcp {
                let mcp_for_setup = mcp_manager.clone();
                let db_for_setup = Arc::clone(&db);
                tauri::async_runtime::spawn(async move {
                    let _ = mcp_for_setup.start(&db_for_setup).await;
                });
            }

            // Start file watcher if in file storage mode
            if storage_mode == "file" {
                let app_handle = app.handle().clone();
                let db_clone = Arc::clone(&db);
                let watcher_clone = watcher.clone();

                tauri::async_runtime::spawn(async move {
                    if let Err(e) = setup_watcher(app_handle, db_clone, watcher_clone).await {
                        log::error!("Failed to setup file watcher: {}", e);
                    }
                });
            }

            let status_label = MenuItemBuilder::with_id("status", "Status: Idle")
                .enabled(false)
                .build(app)?;
            let quick_sync = MenuItemBuilder::with_id("sync", "Quick Sync").build(app)?;
            let mcp_info = MenuItemBuilder::with_id("mcp_info", "MCP: Disconnected")
                .enabled(false)
                .build(app)?;

            let show = MenuItemBuilder::with_id("show", "Show RuleWeaver").build(app)?;
            let hide = MenuItemBuilder::with_id("hide", "Hide to Tray").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "Quit RuleWeaver").build(app)?;
            let tray_menu = MenuBuilder::new(app)
                .item(&status_label)
                .item(&mcp_info)
                .separator()
                .item(&quick_sync)
                .separator()
                .item(&show)
                .item(&hide)
                .separator()
                .item(&quit)
                .build()?;

            let global_status = GlobalStatus::default();
            {
                *global_status.menu.lock() = Some(tray_menu.clone());
            }

            let app_handle = app.handle().clone();
            TrayIconBuilder::with_id("main")
                .menu(&tray_menu)
                .on_menu_event(move |app, event| match event.id().as_ref() {
                    "sync" => {
                        let app_handle = app.clone();
                        tauri::async_runtime::spawn(async move {
                            if let (Some(db), Some(status)) = (
                                app_handle.try_state::<Arc<Database>>(),
                                app_handle.try_state::<GlobalStatus>(),
                            ) {
                                {
                                    *status.sync_status.lock() = "Syncing...".to_string();
                                    status.update_tray();
                                }

                                // Perform sync asynchronously
                                let result = async {
                                    let engine = crate::sync::SyncEngine::new(db);
                                    let rules = db.get_all_rules().await?;
                                    Ok::<_, crate::error::AppError>(engine.sync_all(rules))
                                }.await;

                                {
                                    *status.sync_status.lock() = "Idle".to_string();
                                    status.update_tray();
                                }

                                match result {
                                    Ok(sync_result) => {
                                        use tauri_plugin_notification::NotificationExt;
                                        if sync_result.success {
                                            app_handle
                                                .notification()
                                                .builder()
                                                .title("Sync Complete")
                                                .body(format!(
                                                    "Successfully synced {} files.",
                                                    sync_result.files_written.len()
                                                ))
                                                .show()
                                                .ok();
                                        } else {
                                            app_handle
                                                .notification()
                                                .builder()
                                                .title("Sync Failed")
                                                .body("Errors occurred during quick sync.")
                                                .show()
                                                .ok();
                                        }
                                    }
                                    Err(e) => {
                                        log::error!("Sync task failed: {}", e);
                                    }
                                }
                            }
                        });
                    }

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
                        if let Some(watcher_state) = app.try_state::<WatcherState>() {
                            let _ = watcher_state.0.stop();
                        }
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
                        
                        // Need a way to check setting synchronously or fire async check
                        // Since on_window_event is sync, we'll spawn a check.
                        // However, preventing close is immediate.
                        // Standard pattern: Prevent close, check setting, if minimize -> hide, else -> exit.
                        
                        let app = app_for_events.clone();
                        tauri::async_runtime::spawn(async move {
                            let should_minimize = if let Some(db) = app.try_state::<Arc<Database>>() {
                                db.get_setting(MINIMIZE_TO_TRAY_KEY)
                                    .await
                                    .ok()
                                    .flatten()
                                    .map(|v| v == "true")
                                    .unwrap_or(true)
                            } else {
                                true
                            };

                            if should_minimize {
                                if let Some(main) = app.get_webview_window("main") {
                                    let _ = main.hide();
                                }
                            } else {
                                if let Some(mcp) = app.try_state::<McpManager>() {
                                    let mcp_clone = mcp.inner().clone();
                                    let _ = mcp_clone.stop().await;
                                }
                                app.exit(0);
                            }
                        });
                    }
                });
            }

            app.manage(Arc::clone(&db));
            app.manage(mcp_manager);
            app.manage(WatcherState(watcher));
            app.manage(global_status);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_all_rules,
            commands::get_rule_by_id,
            commands::create_rule,
            commands::update_rule,
            commands::delete_rule,
            commands::bulk_delete_rules,
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
            commands::scan_ai_tool_import_candidates,
            commands::import_ai_tool_rules,
            commands::scan_rule_file_import,
            commands::import_rule_from_file,
            commands::scan_rule_directory_import,
            commands::import_rules_from_directory,
            commands::scan_rule_url_import,
            commands::import_rule_from_url,
            commands::scan_rule_clipboard_import,
            commands::import_rule_from_clipboard,
            commands::get_rule_import_history,
            commands::export_configuration,
            commands::import_configuration,
            commands::preview_import,
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
            commands::sync_skills,
            commands::get_mcp_status,
            commands::start_mcp_server,
            commands::stop_mcp_server,
            commands::restart_mcp_server,
            commands::get_mcp_connection_instructions,
            commands::get_mcp_logs,
            commands::get_execution_history,
            slash_commands::commands::sync_slash_command,
            slash_commands::commands::sync_all_slash_commands,
            slash_commands::commands::get_slash_command_status,
            slash_commands::commands::cleanup_slash_commands,
            slash_commands::commands::remove_slash_command_files,
            slash_commands::commands::get_slash_command_adapters,
            slash_commands::commands::test_slash_command_generation,
            slash_commands::commands::get_slash_command_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

pub fn run_mcp_cli(port: u16, token: Option<String>) -> std::result::Result<(), String> {
    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    
    rt.block_on(async {
        let db = Arc::new(Database::new_for_cli().await.map_err(|e| e.to_string())?);
        let manager = McpManager::new(port);
        
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

async fn setup_watcher(
    app: tauri::AppHandle,
    db: Arc<Database>,
    watcher: RuleFileWatcher,
) -> crate::error::Result<()> {
    let global_dir = crate::file_storage::get_global_rules_dir()?;
    if !global_dir.exists() {
        std::fs::create_dir_all(&global_dir)?;
    }

    let app_handle_for_callback = app.clone();
    let db_for_callback = Arc::clone(&db);

    let callback = Box::new(move |event: crate::file_storage::FileChangeEvent| {
        let app = app_handle_for_callback.clone();
        let db = Arc::clone(&db_for_callback);

        match event {
            crate::file_storage::FileChangeEvent::Created(path)
            | crate::file_storage::FileChangeEvent::Modified(path) => {
                log::info!("File watcher detected change in: {}", path.display());

                tauri::async_runtime::spawn(async move {
                    if let Err(e) = handle_external_rule_change(&app, &db, path).await {
                        log::error!("Failed to handle external rule change: {}", e);
                        use tauri_plugin_notification::NotificationExt;
                        app.notification()
                            .builder()
                            .title("Sync Error")
                            .body("Failed to process file changes")
                            .show()
                            .ok();
                    }
                });
            }
            crate::file_storage::FileChangeEvent::Deleted(path) => {
                log::info!("File watcher detected deletion: {}", path.display());
                let _ = app.emit("rule-file-deleted", path.to_string_lossy());
            }
        }
    });

    watcher.start(&global_dir, callback.clone())?;

    if let Ok(local_roots) = crate::commands::get_local_rule_roots(&db).await {
        for root in local_roots {
            let local_rules_dir = crate::file_storage::get_local_rules_dir(&root);
            if local_rules_dir.exists() {
                if let Err(e) = watcher.start(&local_rules_dir, callback.clone()) {
                    log::error!(
                        "Failed to watch local dir {}: {}",
                        local_rules_dir.display(),
                        e
                    );
                }
            }
        }
    }

    Ok(())
}

async fn handle_external_rule_change(
    app: &tauri::AppHandle,
    db: &Database,
    path: std::path::PathBuf,
) -> crate::error::Result<()> {
    use tauri_plugin_notification::NotificationExt;

    let status = app.try_state::<GlobalStatus>();

    // Canonicalize input path for reliable comparison
    let canonical_path = std::fs::canonicalize(&path)?;
    let path_str = canonical_path.to_string_lossy().to_string();

    // 1. Load the rule from disk
    let rule_from_disk = crate::file_storage::load_rule_from_file(&canonical_path)?;

    // 2. Check if it exists in DB
    let existing_rule = db.get_rule_by_id(&rule_from_disk.id).await.ok();

    if let Some(_existing) = existing_rule {
        // Compute what we *would* write to disk for this rule if we synced from DB
        // to see if the external change actually introduced a difference.
        let engine = crate::sync::SyncEngine::new(db);

        // Check for conflicts using hashes
        let rules = db.get_all_rules().await?;
        let preview = engine.preview(rules);

        let conflict = preview.conflicts.iter().find(|c| {
            if let Ok(c_path) = std::fs::canonicalize(std::path::Path::new(&c.file_path)) {
                c_path == canonical_path
            } else {
                false
            }
        });

        if let Some(_c) = conflict {
            // There is a real difference between what's in DB and what's on disk.
            log::info!(
                "External change conflict detected for rule: {}",
                rule_from_disk.name
            );

            app.notification()
                .builder()
                .title("Sync Conflict Detected")
                .body(format!(
                    "External changes to '{}' conflict with local database. Click to resolve.",
                    rule_from_disk.name
                ))
                .show()
                .ok();

            let _ = app.emit("rule-conflict", path_str);
            return Ok(());
        } else {
            // No conflict found by SyncEngine. This means the file on disk matches RuleWeaver's last known state.
            // This event was likely triggered by RuleWeaver itself during a sync.
            // We return early to avoid an infinite loop of Sync -> Watcher -> Sync.
            log::debug!(
                "File watcher ignore: No conflict for {}",
                rule_from_disk.name
            );
            return Ok(());
        }
    } else {
        // New rule created externally - this is always an update
        log::info!("New rule detected externally: {}", rule_from_disk.name);
        db.create_rule(crate::models::CreateRuleInput {
            name: rule_from_disk.name.clone(),
            content: rule_from_disk.content.clone(),
            scope: rule_from_disk.scope,
            target_paths: rule_from_disk.target_paths.clone(),
            enabled_adapters: rule_from_disk.enabled_adapters.clone(),
            enabled: rule_from_disk.enabled,
        }).await?;
    }

    // If we reached here, we need to sync other adapters/files affected by this rule
    if let Some(ref s) = status {
        *s.sync_status.lock() = "Syncing...".to_string();
        s.update_tray();
    }

    let engine = crate::sync::SyncEngine::new(db);
    let sync_result = engine.sync_rule(rule_from_disk.clone());

    if let Some(ref s) = status {
        *s.sync_status.lock() = "Idle".to_string();
        s.update_tray();
    }

    if sync_result.success {
        app.notification()
            .builder()
            .title("Rule Auto-Synced")
            .body(format!(
                "External changes to '{}' have been applied.",
                rule_from_disk.name
            ))
            .show()
            .ok();
        let _ = app.emit("sync-complete", sync_result);
    } else {
        app.notification()
            .builder()
            .title("Sync Warning")
            .body(format!(
                "Detected changes in '{}' but sync had issues.",
                rule_from_disk.name
            ))
            .show()
            .ok();
        let _ = app.emit("sync-error", sync_result);
    }

    Ok(())
}
