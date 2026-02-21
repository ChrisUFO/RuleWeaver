mod commands;
mod database;
mod error;
mod models;
mod sync;

use database::Database;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let db = Database::new(app.handle())?;
            app.manage(db);
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
