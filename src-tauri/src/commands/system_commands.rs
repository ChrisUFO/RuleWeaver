use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use tauri::State;

use crate::database::{get_app_data_path, Database};
use crate::error::Result;
use crate::models::registry::REGISTRY;
use crate::models::{
    AdapterType, CleanupResult, DeleteScopedSecretInput, EffectiveSecret, ExecutionLog,
    InstalledToolInfo, ResolveScopedSecretsInput, ScopedSecret, SecretStorageStatus,
    SyncHistoryEntry, SyncManifestFilter, ToolSyncPreferences, UpsertScopedSecretInput,
    UpsertToolSyncPreferencesInput, WslAdapterConfig, WslConfig, WslDistribution,
};
use crate::secrets;

use super::validate_path;

fn parse_wsl_config_with_recovery(json: &str) -> WslConfig {
    match serde_json::from_str::<WslConfig>(json) {
        Ok(config) => config,
        Err(e) => {
            log::warn!(
                "Failed to deserialize WSL config: {}. Attempting partial recovery...",
                e
            );

            match serde_json::from_str::<serde_json::Value>(json) {
                Ok(value) => {
                    let mut config = WslConfig::default();
                    let mut recovered_fields: Vec<String> = Vec::new();

                    if let Some(enabled) = value.get("enabled").and_then(|v| v.as_bool()) {
                        config.enabled = enabled;
                        recovered_fields.push("enabled".to_string());
                    }

                    if let Some(dist) = value.get("defaultDistribution").and_then(|v| v.as_str()) {
                        config.default_distribution = Some(dist.to_string());
                        recovered_fields.push("defaultDistribution".to_string());
                    }

                    if let Some(adapters) = value.get("adapters").and_then(|v| v.as_object()) {
                        let mut valid_adapters = 0;
                        for (key, adapter_value) in adapters {
                            if let Ok(adapter_type) = AdapterType::from_str(key.as_str()) {
                                if let Ok(adapter_config) = serde_json::from_value::<WslAdapterConfig>(
                                    adapter_value.clone(),
                                ) {
                                    config.adapters.insert(adapter_type, adapter_config);
                                    valid_adapters += 1;
                                }
                            }
                        }
                        if valid_adapters > 0 {
                            recovered_fields.push(format!("{} adapter(s)", valid_adapters));
                        }
                    }

                    if recovered_fields.is_empty() {
                        log::warn!("WSL config recovery: no fields could be recovered");
                    } else {
                        log::info!(
                            "WSL config recovery: recovered [{}]",
                            recovered_fields.join(", ")
                        );
                    }

                    config
                }
                Err(e2) => {
                    log::warn!(
                        "WSL config recovery failed (not valid JSON): {}. Using default config.",
                        e2
                    );
                    WslConfig::default()
                }
            }
        }
    }
}

#[tauri::command]
pub async fn get_execution_history(
    limit: Option<u32>,
    db: State<'_, Arc<Database>>,
) -> Result<Vec<ExecutionLog>> {
    db.get_execution_history(limit.unwrap_or(100)).await
}

#[tauri::command]
pub async fn get_execution_history_filtered(
    command_id: Option<String>,
    failure_class: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
    db: State<'_, Arc<Database>>,
) -> Result<Vec<ExecutionLog>> {
    db.get_execution_history_filtered(
        command_id.as_deref(),
        failure_class.as_deref(),
        limit.unwrap_or(50),
        offset.unwrap_or(0),
    )
    .await
}

#[tauri::command]
pub async fn get_sync_history(
    limit: Option<u32>,
    db: State<'_, Arc<Database>>,
) -> Result<Vec<SyncHistoryEntry>> {
    db.get_sync_history(limit.unwrap_or(50)).await
}

#[tauri::command]
pub async fn read_file_content(path: String) -> Result<String> {
    let validated_path = validate_path(&path)?;
    let content = tokio::task::spawn_blocking(move || {
        fs::read_to_string(validated_path).map_err(crate::error::AppError::Io)
    })
    .await
    .map_err(|e| crate::error::AppError::InvalidInput {
        message: e.to_string(),
    })??;
    Ok(content)
}

#[tauri::command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
pub async fn get_setting(key: String, db: State<'_, Arc<Database>>) -> Result<Option<String>> {
    db.get_setting(&key).await
}

#[tauri::command]
pub async fn set_setting(key: String, value: String, db: State<'_, Arc<Database>>) -> Result<()> {
    db.set_setting(&key, &value).await
}

#[tauri::command]
pub async fn get_all_settings(db: State<'_, Arc<Database>>) -> Result<HashMap<String, String>> {
    db.get_all_settings().await
}

#[tauri::command]
pub async fn list_scoped_secrets(db: State<'_, Arc<Database>>) -> Result<Vec<ScopedSecret>> {
    secrets::list_scoped_secrets(db.inner().as_ref()).await
}

#[tauri::command]
pub fn get_secret_storage_status_cmd() -> Result<SecretStorageStatus> {
    Ok(secrets::get_secret_storage_status())
}

#[tauri::command]
pub async fn upsert_scoped_secret(
    input: UpsertScopedSecretInput,
    db: State<'_, Arc<Database>>,
) -> Result<ScopedSecret> {
    secrets::upsert_scoped_secret(db.inner().as_ref(), input).await
}

#[tauri::command]
pub async fn delete_scoped_secret(
    input: DeleteScopedSecretInput,
    db: State<'_, Arc<Database>>,
) -> Result<()> {
    secrets::delete_scoped_secret(db.inner().as_ref(), input).await
}

#[tauri::command]
pub async fn resolve_scoped_secrets_cmd(
    input: ResolveScopedSecretsInput,
    db: State<'_, Arc<Database>>,
) -> Result<Vec<EffectiveSecret>> {
    secrets::resolve_scoped_secrets(db.inner().as_ref(), input).await
}

#[tauri::command]
pub fn get_app_data_path_cmd(app: tauri::AppHandle) -> Result<String> {
    let path = get_app_data_path(&app)?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn open_in_explorer(path: String) -> Result<()> {
    let validated_path = validate_path(&path)?;

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .args(["/select,", &validated_path.to_string_lossy()])
            .spawn()
            .map_err(crate::error::AppError::Io)?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .args(["-R", &validated_path.to_string_lossy()])
            .spawn()
            .map_err(crate::error::AppError::Io)?;
    }

    #[cfg(target_os = "linux")]
    {
        let parent_dir = validated_path.parent().unwrap_or(std::path::Path::new("/"));
        std::process::Command::new("xdg-open")
            .arg(parent_dir)
            .spawn()
            .map_err(crate::error::AppError::Io)?;
    }

    Ok(())
}

#[tauri::command]
pub fn detect_installed_tools() -> Result<Vec<InstalledToolInfo>> {
    let home = dirs::home_dir().ok_or_else(|| {
        crate::error::AppError::Path("Could not determine home directory".to_string())
    })?;

    let tools = REGISTRY.all();
    let mut results = Vec::new();

    for tool in tools {
        let config_path = get_tool_config_path(&tool.id, &home);
        let is_installed = config_path.as_ref().map(|p| p.exists()).unwrap_or(false);

        results.push(InstalledToolInfo {
            adapter: tool.id,
            name: tool.name.to_string(),
            is_installed,
            config_path: config_path.map(|p| p.to_string_lossy().to_string()),
        });
    }

    Ok(results)
}

fn get_tool_config_path(adapter: &AdapterType, home: &Path) -> Option<PathBuf> {
    let path = match adapter {
        AdapterType::ClaudeCode => home.join(".claude"),
        AdapterType::Cursor => home.join(".cursor"),
        AdapterType::OpenCode => home.join(".config").join("opencode"),
        AdapterType::Cline => home.join("Documents").join("Cline"),
        AdapterType::Gemini => home.join(".gemini"),
        AdapterType::Codex => home.join(".codex"),
        AdapterType::Kilo => home.join(".kilocode"),
        AdapterType::RooCode => home.join(".roo"),
        AdapterType::Augment => home.join(".augment"),
        AdapterType::Antigravity => home.join(".gemini").join("antigravity"),
    };
    Some(path)
}

#[tauri::command]
pub async fn get_all_tool_sync_preferences(
    db: State<'_, Arc<Database>>,
) -> Result<Vec<ToolSyncPreferences>> {
    db.get_all_tool_sync_preferences().await
}

#[tauri::command]
pub async fn get_tool_sync_preferences(
    tool_id: AdapterType,
    db: State<'_, Arc<Database>>,
) -> Result<Option<ToolSyncPreferences>> {
    db.get_tool_sync_preferences(&tool_id).await
}

#[tauri::command]
pub async fn upsert_tool_sync_preferences(
    input: UpsertToolSyncPreferencesInput,
    db: State<'_, Arc<Database>>,
) -> Result<ToolSyncPreferences> {
    db.upsert_tool_sync_preferences(input).await
}

#[tauri::command]
pub async fn cleanup_synced_files(
    filter: SyncManifestFilter,
    db: State<'_, Arc<Database>>,
) -> Result<CleanupResult> {
    const RULEWEAVER_MARKER: &str = "Generated by RuleWeaver";
    let entries = db.list_sync_manifest(filter).await?;

    let mut files_removed = 0;
    let mut files_skipped = 0;
    let mut errors = Vec::new();
    let mut removed_paths = Vec::new();

    for entry in entries {
        let path = PathBuf::from(&entry.path);
        if path.exists() {
            match tokio::fs::read_to_string(&path).await {
                Ok(content) => {
                    if !content.contains(RULEWEAVER_MARKER) {
                        files_skipped += 1;
                        errors.push(format!(
                            "Skipped {}: file does not contain RuleWeaver marker",
                            entry.path
                        ));
                        continue;
                    }
                }
                Err(e) => {
                    files_skipped += 1;
                    errors.push(format!("Failed to read {}: {}", entry.path, e));
                    continue;
                }
            }

            match tokio::fs::remove_file(&path).await {
                Ok(()) => {
                    files_removed += 1;
                    removed_paths.push(entry.path.clone());
                    if let Err(e) = db.delete_sync_manifest_by_path(&entry.path).await {
                        errors.push(format!(
                            "Failed to delete manifest entry for {}: {}",
                            entry.path, e
                        ));
                    }
                }
                Err(e) => {
                    files_skipped += 1;
                    errors.push(format!("Failed to remove {}: {}", entry.path, e));
                }
            }
        } else {
            files_skipped += 1;
            if let Err(e) = db.delete_sync_manifest_by_path(&entry.path).await {
                errors.push(format!(
                    "Failed to delete stale manifest entry for {}: {}",
                    entry.path, e
                ));
            }
        }
    }

    Ok(CleanupResult {
        files_removed,
        files_skipped,
        errors,
        removed_paths,
    })
}

#[tauri::command]
pub async fn get_wsl_config(db: State<'_, Arc<Database>>) -> Result<WslConfig> {
    let config_json = db.get_setting("wsl_config").await?;
    match config_json {
        Some(json) => {
            serde_json::from_str(&json).map_err(|e| crate::error::AppError::InvalidInput {
                message: format!("Failed to parse WSL config: {}", e),
            })
        }
        None => Ok(WslConfig::default()),
    }
}

#[tauri::command]
pub async fn set_wsl_config(config: WslConfig, db: State<'_, Arc<Database>>) -> Result<()> {
    let config_json =
        serde_json::to_string(&config).map_err(|e| crate::error::AppError::InvalidInput {
            message: format!("Failed to serialize WSL config: {}", e),
        })?;
    db.set_setting("wsl_config", &config_json).await
}

#[tauri::command]
pub async fn set_wsl_adapter_config(
    adapter: AdapterType,
    adapter_config: WslAdapterConfig,
    db: State<'_, Arc<Database>>,
) -> Result<WslConfig> {
    let mut config = match db.get_setting("wsl_config").await? {
        Some(json) => parse_wsl_config_with_recovery(&json),
        None => WslConfig::default(),
    };
    config.set_adapter_config(adapter, adapter_config);
    let config_json =
        serde_json::to_string(&config).map_err(|e| crate::error::AppError::InvalidInput {
            message: format!("Failed to serialize WSL config: {}", e),
        })?;
    db.set_setting("wsl_config", &config_json).await?;
    Ok(config)
}

#[tauri::command]
pub async fn set_wsl_enabled(enabled: bool, db: State<'_, Arc<Database>>) -> Result<WslConfig> {
    let mut config = match db.get_setting("wsl_config").await? {
        Some(json) => parse_wsl_config_with_recovery(&json),
        None => WslConfig::default(),
    };
    config.enabled = enabled;
    let config_json =
        serde_json::to_string(&config).map_err(|e| crate::error::AppError::InvalidInput {
            message: format!("Failed to serialize WSL config: {}", e),
        })?;
    db.set_setting("wsl_config", &config_json).await?;
    Ok(config)
}

#[tauri::command]
#[cfg(target_os = "windows")]
pub fn is_wsl_installed() -> bool {
    crate::wsl::WslDetection::is_wsl_installed()
}

#[tauri::command]
#[cfg(not(target_os = "windows"))]
pub fn is_wsl_installed() -> bool {
    false
}

#[tauri::command]
#[cfg(target_os = "windows")]
pub fn list_wsl_distributions() -> Result<Vec<WslDistribution>> {
    if !crate::wsl::WslDetection::is_wsl_installed() {
        return Ok(Vec::new());
    }
    let distros = crate::wsl::WslDetection::list_distributions().map_err(|e| {
        crate::error::AppError::InvalidInput {
            message: e.to_string(),
        }
    })?;
    Ok(distros)
}

#[tauri::command]
#[cfg(not(target_os = "windows"))]
pub fn list_wsl_distributions() -> Result<Vec<WslDistribution>> {
    Ok(Vec::new())
}
