use std::path::PathBuf;
use std::sync::Arc;

use tauri::State;

use crate::database::Database;
use crate::error::Result;
use crate::models::{
    ImportExecutionOptions, ImportExecutionResult, ImportHistoryEntry, ImportScanResult,
};
use crate::rule_import;

#[tauri::command]
pub fn scan_ai_tool_import_candidates(
    options: Option<ImportExecutionOptions>,
    db: State<'_, Arc<Database>>,
) -> Result<ImportScanResult> {
    let opts = options.unwrap_or_default();
    let max_size = rule_import::resolve_max_size(&opts);
    rule_import::scan_ai_tool_candidates(&db, max_size)
}

#[tauri::command]
pub fn import_ai_tool_rules(
    options: Option<ImportExecutionOptions>,
    db: State<'_, Arc<Database>>,
) -> Result<ImportExecutionResult> {
    let opts = options.unwrap_or_default();
    let max_size = rule_import::resolve_max_size(&opts);
    let scan = rule_import::scan_ai_tool_candidates(&db, max_size)?;
    rule_import::execute_import(&db, scan, opts)
}

#[tauri::command]
pub fn import_rule_from_file(
    path: String,
    options: Option<ImportExecutionOptions>,
    db: State<'_, Arc<Database>>,
) -> Result<ImportExecutionResult> {
    let opts = options.unwrap_or_default();
    let max_size = rule_import::resolve_max_size(&opts);
    let scan = rule_import::scan_file_to_candidates(&PathBuf::from(path), max_size);
    rule_import::execute_import(&db, scan, opts)
}

#[tauri::command]
pub fn scan_rule_file_import(
    path: String,
    options: Option<ImportExecutionOptions>,
) -> ImportScanResult {
    let opts = options.unwrap_or_default();
    let max_size = rule_import::resolve_max_size(&opts);
    rule_import::scan_file_to_candidates(&PathBuf::from(path), max_size)
}

#[tauri::command]
pub fn import_rules_from_directory(
    path: String,
    options: Option<ImportExecutionOptions>,
    db: State<'_, Arc<Database>>,
) -> Result<ImportExecutionResult> {
    let opts = options.unwrap_or_default();
    let max_size = rule_import::resolve_max_size(&opts);
    let scan = rule_import::scan_directory_to_candidates(&PathBuf::from(path), max_size);
    rule_import::execute_import(&db, scan, opts)
}

#[tauri::command]
pub fn scan_rule_directory_import(
    path: String,
    options: Option<ImportExecutionOptions>,
) -> ImportScanResult {
    let opts = options.unwrap_or_default();
    let max_size = rule_import::resolve_max_size(&opts);
    rule_import::scan_directory_to_candidates(&PathBuf::from(path), max_size)
}

#[tauri::command]
pub async fn import_rule_from_url(
    url: String,
    options: Option<ImportExecutionOptions>,
    db: State<'_, Arc<Database>>,
) -> Result<ImportExecutionResult> {
    let opts = options.unwrap_or_default();
    let max_size = rule_import::resolve_max_size(&opts);
    let scan = rule_import::scan_url_to_candidates(&url, max_size).await?;
    rule_import::execute_import(&db, scan, opts)
}

#[tauri::command]
pub async fn scan_rule_url_import(
    url: String,
    options: Option<ImportExecutionOptions>,
) -> Result<ImportScanResult> {
    let opts = options.unwrap_or_default();
    let max_size = rule_import::resolve_max_size(&opts);
    rule_import::scan_url_to_candidates(&url, max_size).await
}

#[tauri::command]
pub fn import_rule_from_clipboard(
    content: String,
    name: Option<String>,
    options: Option<ImportExecutionOptions>,
    db: State<'_, Arc<Database>>,
) -> Result<ImportExecutionResult> {
    let opts = options.unwrap_or_default();
    let max_size = rule_import::resolve_max_size(&opts);
    let scan = rule_import::scan_clipboard_to_candidates(&content, name.as_deref(), max_size)?;
    rule_import::execute_import(&db, scan, opts)
}

#[tauri::command]
pub fn scan_rule_clipboard_import(
    content: String,
    name: Option<String>,
    options: Option<ImportExecutionOptions>,
) -> Result<ImportScanResult> {
    let opts = options.unwrap_or_default();
    let max_size = rule_import::resolve_max_size(&opts);
    rule_import::scan_clipboard_to_candidates(&content, name.as_deref(), max_size)
}

#[tauri::command]
pub fn get_rule_import_history(db: State<'_, Arc<Database>>) -> Vec<ImportHistoryEntry> {
    rule_import::read_import_history(&db)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_rule_clipboard_import_enforces_max_size() {
        let result = scan_rule_clipboard_import(
            "123456".to_string(),
            Some("clip".to_string()),
            Some(ImportExecutionOptions {
                max_file_size_bytes: Some(5),
                ..Default::default()
            }),
        );
        assert!(result.is_err());
    }

    #[test]
    fn scan_rule_file_import_returns_errors_for_missing_file() {
        let result = scan_rule_file_import(
            "C:/definitely/not/found.md".to_string(),
            Some(ImportExecutionOptions::default()),
        );
        assert!(!result.errors.is_empty());
    }
}
