use std::path::PathBuf;
use std::str::FromStr;
use tokio::sync::Mutex;

use chrono::{DateTime, TimeZone, Utc};
use rusqlite::{params, params_from_iter, Connection, OptionalExtension};
use tauri::Manager;

use crate::error::{AppError, Result};
use crate::file_storage::StorageLocation;
use crate::models::{
    AdapterType, Command, CommandArgument, CreateCommandInput, CreateRuleInput, CreateSkillInput,
    CreateSyncManifestInput, DeleteScopedSecretInput, ExecutionLog, ObservabilityEvent,
    ObservabilityEventFilter, ObservabilityEventStatus, ObservabilityEventType, ReconcileOperation,
    ReconcileResultType, Rule, Scope, ScopedSecret, SecretScope, Skill, SyncHistoryEntry,
    SyncManifestEntry, SyncManifestFilter, ToolSyncPreferences, UpdateCommandInput,
    UpdateRuleInput, UpdateSkillInput, UpsertScopedSecretInput, UpsertToolSyncPreferencesInput,
};

fn parse_timestamp_or_now(timestamp: i64) -> DateTime<Utc> {
    chrono::Utc
        .timestamp_opt(timestamp, 0)
        .single()
        .unwrap_or_else(chrono::Utc::now)
}

fn parse_observability_event_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ObservabilityEvent> {
    let timestamp: i64 = row.get("timestamp")?;
    let event_type_raw: String = row.get("event_type")?;
    let status_raw: String = row.get("status")?;
    Ok(ObservabilityEvent {
        id: row.get("id")?,
        timestamp: parse_timestamp_or_now(timestamp),
        event_type: event_type_raw.parse().map_err(|error: AppError| {
            rusqlite::Error::FromSqlConversionFailure(
                2,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })?,
        status: status_raw.parse().map_err(|error: AppError| {
            rusqlite::Error::FromSqlConversionFailure(
                3,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })?,
        source: row.get("source")?,
        entity_kind: row.get("entity_kind")?,
        entity_id: row.get("entity_id")?,
        entity_name: row.get("entity_name")?,
        workspace_path: row.get("workspace_path")?,
        summary: row.get("summary")?,
        metadata: row.get("metadata")?,
        stdout_excerpt: row.get("stdout_excerpt")?,
        stderr_excerpt: row.get("stderr_excerpt")?,
        duration_ms: row
            .get::<_, Option<i64>>("duration_ms")?
            .map(|value| value as u64),
        exit_code: row.get("exit_code")?,
        failure_class: row.get("failure_class")?,
        attempt_number: row
            .get::<_, Option<i64>>("attempt_number")?
            .map(|value| value as u8),
        is_redacted: row.get::<_, i32>("is_redacted")? != 0,
    })
}

pub struct Database(Mutex<Connection>, String);

impl std::fmt::Debug for Database {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Database").finish_non_exhaustive()
    }
}

pub struct ExecutionLogInput<'a> {
    pub command_id: &'a str,
    pub command_name: &'a str,
    pub arguments_json: &'a str,
    pub stdout: &'a str,
    pub stderr: &'a str,
    pub exit_code: i32,
    pub duration_ms: u64,
    pub triggered_by: &'a str,
    pub failure_class: Option<&'a str>,
    pub adapter_context: Option<&'a str>,
    pub workspace_path: Option<&'a str>,
    pub is_redacted: bool,
    pub attempt_number: u8,
}

pub struct ObservabilityEventInput<'a> {
    pub event_type: ObservabilityEventType,
    pub status: ObservabilityEventStatus,
    pub source: &'a str,
    pub entity_kind: Option<&'a str>,
    pub entity_id: Option<&'a str>,
    pub entity_name: Option<&'a str>,
    pub workspace_path: Option<&'a str>,
    pub summary: &'a str,
    pub metadata: Option<&'a str>,
    pub stdout_excerpt: Option<&'a str>,
    pub stderr_excerpt: Option<&'a str>,
    pub duration_ms: Option<u64>,
    pub exit_code: Option<i32>,
    pub failure_class: Option<&'a str>,
    pub attempt_number: Option<u8>,
    pub is_redacted: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReconciliationLogEntry {
    pub id: String,
    #[serde(with = "crate::models::timestamp")]
    pub timestamp: DateTime<Utc>,
    pub operation: ReconcileOperation,
    pub artifact_type: Option<String>,
    pub adapter: Option<AdapterType>,
    pub scope: Option<Scope>,
    pub path: String,
    pub result: ReconcileResultType,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ScopedSecretRecord {
    pub id: String,
    pub key: String,
    pub value: String,
    pub scope: SecretScope,
    pub workspace_path: Option<String>,
    pub artifact_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Database {
    async fn new_with_db_path(db_path: PathBuf) -> Result<Self> {
        let secret_namespace = db_path.to_string_lossy().to_string();
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Connection::open is blocking, so we wrap it in spawn_blocking
        let conn = tokio::task::spawn_blocking(move || -> Result<Connection> {
            let mut conn = Connection::open(&db_path)?;
            run_migrations(&mut conn)?;
            Ok(conn)
        })
        .await
        .map_err(|e| AppError::Database(rusqlite::Error::ToSqlConversionFailure(Box::new(e))))??;

        Ok(Self(Mutex::new(conn), secret_namespace))
    }

    pub async fn new(app_handle: &tauri::AppHandle) -> Result<Self> {
        let app_data_dir = app_handle
            .path()
            .app_data_dir()
            .map_err(|e| AppError::Path(e.to_string()))?;
        let db_path = app_data_dir.join("ruleweaver.db");
        Self::new_with_db_path(db_path).await
    }

    pub async fn new_for_cli() -> Result<Self> {
        let app_data_dir = default_app_data_dir()?;
        let db_path = app_data_dir.join("ruleweaver.db");
        Self::new_with_db_path(db_path).await
    }

    #[cfg(any(test, feature = "test-helpers"))]
    pub async fn new_in_memory() -> Result<Self> {
        crate::secure_storage::reset_test_store();
        let secret_namespace = format!("memory:{}", uuid::Uuid::new_v4());
        let conn = tokio::task::spawn_blocking(move || -> Result<Connection> {
            let mut conn = Connection::open_in_memory()?;
            run_migrations(&mut conn)?;
            Ok(conn)
        })
        .await
        .map_err(|e| AppError::Database(rusqlite::Error::ToSqlConversionFailure(Box::new(e))))??;

        Ok(Self(Mutex::new(conn), secret_namespace))
    }

    pub fn secret_namespace(&self) -> &str {
        &self.1
    }

    /// Re-establishes the database connection and runs migrations.
    /// Useful for recovering from disk disconnections or handling external database modifications.
    #[allow(dead_code)]
    pub async fn reconnect(&self) -> Result<()> {
        let db_path = {
            let conn = self.0.lock().await;
            let path: String = conn.query_row("PRAGMA database_list", [], |row| row.get(2))?;
            PathBuf::from(path)
        };

        let new_conn = tokio::task::spawn_blocking(move || -> Result<Connection> {
            let mut conn = Connection::open(&db_path)?;
            run_migrations(&mut conn)?;
            Ok(conn)
        })
        .await
        .map_err(|e| AppError::Database(rusqlite::Error::ToSqlConversionFailure(Box::new(e))))??;

        let mut guard = self.0.lock().await;
        *guard = new_conn;
        Ok(())
    }

    pub async fn get_all_rules(&self) -> Result<Vec<Rule>> {
        let conn = self.0.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, content, scope, target_paths, enabled_adapters, enabled, created_at, updated_at 
             FROM rules 
             ORDER BY updated_at DESC"
        )?;

        let rules = stmt
            .query_map([], |row| {
                let id: String = row.get("id")?;
                let name: String = row.get("name")?;
                let description: String = row.get("description")?;
                let content: String = row.get("content")?;
                let scope_str: String = row.get("scope")?;
                let target_paths_json: Option<String> = row.get("target_paths")?;
                let enabled_adapters_json: String = row.get("enabled_adapters")?;
                let enabled: bool = row.get("enabled")?;
                let created_at: i64 = row.get("created_at")?;
                let updated_at: i64 = row.get("updated_at")?;

                let scope = Scope::from_str(&scope_str).map_err(|_| {
                    rusqlite::Error::FromSqlConversionFailure(
                        4,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("Invalid scope for rule {}: {}", id, scope_str),
                        )),
                    )
                })?;

                let target_paths: Option<Vec<String>> = match target_paths_json {
                    Some(j) => Some(serde_json::from_str(&j).map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            5,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })?),
                    None => None,
                };

                let enabled_adapters: Vec<AdapterType> =
                    serde_json::from_str(&enabled_adapters_json).map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            6,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })?;

                Ok(Rule {
                    id,
                    name,
                    description,
                    content,
                    scope,
                    target_paths,
                    enabled_adapters,
                    enabled,
                    created_at: parse_timestamp_or_now(created_at),
                    updated_at: parse_timestamp_or_now(updated_at),
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(rules)
    }

    pub async fn get_rule_by_id(&self, id: &str) -> Result<Rule> {
        let conn = self.0.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, content, scope, target_paths, enabled_adapters, enabled, created_at, updated_at 
             FROM rules 
             WHERE id = ?"
        )?;

        let rule = stmt
            .query_row(params![id], |row| {
                let id: String = row.get("id")?;
                let name: String = row.get("name")?;
                let description: String = row.get("description")?;
                let content: String = row.get("content")?;
                let scope_str: String = row.get("scope")?;
                let target_paths_json: Option<String> = row.get("target_paths")?;
                let enabled_adapters_json: String = row.get("enabled_adapters")?;
                let enabled: bool = row.get("enabled")?;
                let created_at: i64 = row.get("created_at")?;
                let updated_at: i64 = row.get("updated_at")?;

                let scope = Scope::from_str(&scope_str).map_err(|_| {
                    rusqlite::Error::FromSqlConversionFailure(
                        4,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("Invalid scope for rule {}: {}", id, scope_str),
                        )),
                    )
                })?;

                let target_paths: Option<Vec<String>> = match target_paths_json {
                    Some(j) => Some(serde_json::from_str(&j).map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            5,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })?),
                    None => None,
                };

                let enabled_adapters: Vec<AdapterType> =
                    serde_json::from_str(&enabled_adapters_json).map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            6,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })?;

                Ok(Rule {
                    id,
                    name,
                    description,
                    content,
                    scope,
                    target_paths,
                    enabled_adapters,
                    enabled,
                    created_at: parse_timestamp_or_now(created_at),
                    updated_at: parse_timestamp_or_now(updated_at),
                })
            })
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    AppError::RuleNotFound { id: id.to_string() }
                }
                _ => AppError::Database(e),
            })?;

        Ok(rule)
    }

    pub async fn create_rule(&self, input: CreateRuleInput) -> Result<Rule> {
        let conn = self.0.lock().await;
        let now = chrono::Utc::now().timestamp();
        let id = input.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        let target_paths_json = input
            .target_paths
            .as_ref()
            .map(|p| serde_json::to_string(p).unwrap_or_default());

        let enabled_adapters_json = serde_json::to_string(&input.enabled_adapters)?;

        conn.execute(
            "INSERT INTO rules (id, name, description, content, scope, target_paths, enabled_adapters, enabled, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                id,
                input.name,
                input.description,
                input.content,
                input.scope.as_str(),
                target_paths_json,
                enabled_adapters_json,
                input.enabled,
                now,
                now
            ],
        )?;

        drop(conn);
        self.get_rule_by_id(&id).await
    }

    pub async fn update_rule(&self, id: &str, input: UpdateRuleInput) -> Result<Rule> {
        let existing = self.get_rule_by_id(id).await?;
        let conn = self.0.lock().await;

        let name = input.name.unwrap_or(existing.name);
        let description = input.description.unwrap_or(existing.description);
        let content = input.content.unwrap_or(existing.content);
        let scope = input.scope.unwrap_or(existing.scope);
        let target_paths = input.target_paths.or(existing.target_paths);
        let enabled_adapters = input.enabled_adapters.unwrap_or(existing.enabled_adapters);
        let enabled = input.enabled.unwrap_or(existing.enabled);
        let now = chrono::Utc::now().timestamp();

        let target_paths_json = target_paths
            .as_ref()
            .map(|p| serde_json::to_string(p).unwrap_or_default());

        let enabled_adapters_json = serde_json::to_string(&enabled_adapters)?;

        conn.execute(
            "UPDATE rules SET name = ?, description = ?, content = ?, scope = ?, target_paths = ?, enabled_adapters = ?, enabled = ?, updated_at = ?
             WHERE id = ?",
            params![
                name,
                description,
                content,
                scope.as_str(),
                target_paths_json,
                enabled_adapters_json,
                enabled,
                now,
                id
            ],
        )?;

        drop(conn);
        self.get_rule_by_id(id).await
    }

    pub async fn delete_rule(&self, id: &str) -> Result<()> {
        let conn = self.0.lock().await;
        conn.execute("DELETE FROM rules WHERE id = ?", params![id])?;
        Ok(())
    }

    pub async fn toggle_rule(&self, id: &str, enabled: bool) -> Result<Rule> {
        let conn = self.0.lock().await;
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "UPDATE rules SET enabled = ?, updated_at = ? WHERE id = ?",
            params![enabled, now, id],
        )?;

        drop(conn);
        self.get_rule_by_id(id).await
    }

    pub async fn get_all_commands(&self) -> Result<Vec<Command>> {
        let conn = self.0.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, script, arguments, is_placeholder, generate_slash_commands, slash_command_adapters, target_paths, created_at, updated_at, timeout_ms, max_retries, base_path
             FROM commands
             ORDER BY updated_at DESC",
        )?;

        let commands = stmt
            .query_map([], |row| {
                let id: String = row.get("id")?;
                let name: String = row.get("name")?;
                let description: String = row.get("description")?;
                let script: String = row.get("script")?;
                let arguments_json: String = row.get("arguments")?;
                let is_placeholder: bool = row.get("is_placeholder")?;
                let generate_slash_commands: bool = row.get("generate_slash_commands")?;
                let slash_adapters_json: String = row.get("slash_command_adapters")?;
                let target_paths_json: String = row.get("target_paths")?;
                let created_at: i64 = row.get("created_at")?;
                let updated_at: i64 = row.get("updated_at")?;
                let timeout_ms: Option<i64> = row.get("timeout_ms")?;
                let max_retries: Option<i32> = row.get("max_retries")?;
                let base_path: Option<String> = row.get("base_path")?;

                let arguments: Vec<CommandArgument> = serde_json::from_str(&arguments_json)
                    .map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            4,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })?;

                let slash_command_adapters: Vec<String> =
                    serde_json::from_str(&slash_adapters_json).map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            7,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })?;

                let target_paths: Vec<String> =
                    serde_json::from_str(&target_paths_json).map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            8,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })?;

                Ok(Command {
                    id,
                    name,
                    description,
                    script,
                    arguments,
                    is_placeholder,
                    generate_slash_commands,
                    slash_command_adapters,
                    target_paths,
                    base_path,
                    timeout_ms: timeout_ms.map(|t| t as u64),
                    max_retries: max_retries.map(|r| r as u8),
                    created_at: parse_timestamp_or_now(created_at),
                    updated_at: parse_timestamp_or_now(updated_at),
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(commands)
    }

    pub async fn get_command_by_id(&self, id: &str) -> Result<Command> {
        let conn = self.0.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, script, arguments, is_placeholder, generate_slash_commands, slash_command_adapters, target_paths, created_at, updated_at, timeout_ms, max_retries, base_path
             FROM commands
             WHERE id = ?",
        )?;

        let command = stmt
            .query_row(params![id], |row| {
                let id: String = row.get("id")?;
                let name: String = row.get("name")?;
                let description: String = row.get("description")?;
                let script: String = row.get("script")?;
                let arguments_json: String = row.get("arguments")?;
                let is_placeholder: bool = row.get("is_placeholder")?;
                let generate_slash_commands: bool = row.get("generate_slash_commands")?;
                let slash_adapters_json: String = row.get("slash_command_adapters")?;
                let target_paths_json: String = row.get("target_paths")?;
                let created_at: i64 = row.get("created_at")?;
                let updated_at: i64 = row.get("updated_at")?;
                let timeout_ms: Option<i64> = row.get("timeout_ms")?;
                let max_retries: Option<i32> = row.get("max_retries")?;
                let base_path: Option<String> = row.get("base_path")?;

                let arguments: Vec<CommandArgument> = serde_json::from_str(&arguments_json)
                    .map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            4,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })?;

                let slash_command_adapters: Vec<String> =
                    serde_json::from_str(&slash_adapters_json).map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            7,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })?;

                let target_paths: Vec<String> =
                    serde_json::from_str(&target_paths_json).map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            8,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })?;

                Ok(Command {
                    id,
                    name,
                    description,
                    script,
                    arguments,
                    is_placeholder,
                    generate_slash_commands,
                    slash_command_adapters,
                    target_paths,
                    base_path,
                    timeout_ms: timeout_ms.map(|t| t as u64),
                    max_retries: max_retries.map(|r| r as u8),
                    created_at: parse_timestamp_or_now(created_at),
                    updated_at: parse_timestamp_or_now(updated_at),
                })
            })
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    AppError::CommandNotFound { id: id.to_string() }
                }
                _ => AppError::Database(e),
            })?;

        Ok(command)
    }

    pub async fn create_command(&self, input: CreateCommandInput) -> Result<Command> {
        let conn = self.0.lock().await;
        let now = chrono::Utc::now().timestamp();
        let id = input.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let arguments_json = serde_json::to_string(&input.arguments)?;
        let slash_adapters_json = serde_json::to_string(&input.slash_command_adapters)?;
        let target_paths_json = serde_json::to_string(&input.target_paths)?;

        conn.execute(
            "INSERT INTO commands (id, name, description, script, arguments, is_placeholder, generate_slash_commands, slash_command_adapters, target_paths, created_at, updated_at, timeout_ms, max_retries, base_path)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                id,
                input.name,
                input.description,
                input.script,
                arguments_json,
                input.is_placeholder,
                input.generate_slash_commands,
                slash_adapters_json,
                target_paths_json,
                now,
                now,
                input.timeout_ms.map(|t| t as i64),
                input.max_retries.map(|r| r as i32),
                input.base_path
            ],
        )?;

        drop(conn);
        self.get_command_by_id(&id).await
    }

    pub async fn update_command(&self, id: &str, input: UpdateCommandInput) -> Result<Command> {
        let existing = self.get_command_by_id(id).await?;
        let conn = self.0.lock().await;

        let name = input.name.unwrap_or(existing.name);
        let description = input.description.unwrap_or(existing.description);
        let script = input.script.unwrap_or(existing.script);
        let arguments = input.arguments.unwrap_or(existing.arguments);
        let is_placeholder = input.is_placeholder.unwrap_or(existing.is_placeholder);
        let generate_slash_commands = input
            .generate_slash_commands
            .unwrap_or(existing.generate_slash_commands);
        let slash_command_adapters = input
            .slash_command_adapters
            .unwrap_or(existing.slash_command_adapters);
        let target_paths = input.target_paths.unwrap_or(existing.target_paths);
        let base_path = input.base_path.or(existing.base_path);
        let timeout_ms = input.timeout_ms.or(existing.timeout_ms);
        let max_retries = input.max_retries.or(existing.max_retries);
        let now = chrono::Utc::now().timestamp();
        let arguments_json = serde_json::to_string(&arguments)?;
        let slash_adapters_json = serde_json::to_string(&slash_command_adapters)?;
        let target_paths_json = serde_json::to_string(&target_paths)?;

        conn.execute(
            "UPDATE commands SET name = ?, description = ?, script = ?, arguments = ?, is_placeholder = ?, generate_slash_commands = ?, slash_command_adapters = ?, target_paths = ?, updated_at = ?, timeout_ms = ?, max_retries = ?, base_path = ?
             WHERE id = ?",
            params![
                name,
                description,
                script,
                arguments_json,
                is_placeholder,
                generate_slash_commands,
                slash_adapters_json,
                target_paths_json,
                now,
                timeout_ms.map(|t| t as i64),
                max_retries.map(|r| r as i32),
                base_path,
                id
            ],
        )?;

        drop(conn);
        self.get_command_by_id(id).await
    }

    pub async fn delete_command(&self, id: &str) -> Result<()> {
        let conn = self.0.lock().await;
        conn.execute("DELETE FROM commands WHERE id = ?", params![id])?;
        Ok(())
    }

    pub async fn get_all_skills(&self) -> Result<Vec<Skill>> {
        let conn = self.0.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, instructions, input_schema, enabled, created_at, updated_at, directory_path, entry_point, scope, target_adapters, target_paths, base_path
             FROM skills
             ORDER BY updated_at DESC",
        )?;

        let skills = stmt
            .query_map([], |row| {
                Ok(Skill {
                    id: row.get("id")?,
                    name: row.get("name")?,
                    description: row.get("description")?,
                    instructions: row.get("instructions")?,
                    input_schema: {
                        let raw: String = row.get("input_schema")?;
                        serde_json::from_str(&raw).map_err(|e| {
                            rusqlite::Error::FromSqlConversionFailure(
                                4,
                                rusqlite::types::Type::Text,
                                Box::new(e),
                            )
                        })?
                    },
                    enabled: row.get("enabled")?,
                    created_at: parse_timestamp_or_now(row.get("created_at")?),
                    updated_at: parse_timestamp_or_now(row.get("updated_at")?),
                    directory_path: row.get("directory_path")?,
                    entry_point: row.get("entry_point")?,
                    scope: Scope::from_str(&row.get::<_, String>("scope")?).map_err(|_| {
                        rusqlite::Error::FromSqlConversionFailure(
                            10,
                            rusqlite::types::Type::Text,
                            Box::new(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                "Invalid skill scope",
                            )),
                        )
                    })?,
                    target_adapters: {
                        let raw: String = row.get("target_adapters")?;
                        serde_json::from_str(&raw).unwrap_or_else(|e| {
                            log::warn!("Failed to parse skill JSON: {}. Falling back to empty.", e);
                            Vec::new()
                        })
                    },
                    target_paths: {
                        let raw: String = row.get("target_paths")?;
                        serde_json::from_str(&raw).unwrap_or_else(|e| {
                            log::warn!("Failed to parse skill JSON: {}. Falling back to empty.", e);
                            Vec::new()
                        })
                    },
                    base_path: row.get("base_path")?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(skills)
    }

    pub async fn get_skill_by_id(&self, id: &str) -> Result<Skill> {
        let conn = self.0.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, instructions, input_schema, enabled, created_at, updated_at, directory_path, entry_point, scope, target_adapters, target_paths, base_path
             FROM skills WHERE id = ?",
        )?;

        let skill = stmt
            .query_row(params![id], |row| {
                Ok(Skill {
                    id: row.get("id")?,
                    name: row.get("name")?,
                    description: row.get("description")?,
                    instructions: row.get("instructions")?,
                    input_schema: {
                        let raw: String = row.get("input_schema")?;
                        serde_json::from_str(&raw).map_err(|e| {
                            rusqlite::Error::FromSqlConversionFailure(
                                4,
                                rusqlite::types::Type::Text,
                                Box::new(e),
                            )
                        })?
                    },
                    enabled: row.get("enabled")?,
                    created_at: parse_timestamp_or_now(row.get("created_at")?),
                    updated_at: parse_timestamp_or_now(row.get("updated_at")?),
                    directory_path: row.get("directory_path")?,
                    entry_point: row.get("entry_point")?,
                    scope: Scope::from_str(&row.get::<_, String>("scope")?).map_err(|_| {
                        rusqlite::Error::FromSqlConversionFailure(
                            10,
                            rusqlite::types::Type::Text,
                            Box::new(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                format!("Invalid scope for skill: {}", id),
                            )),
                        )
                    })?,
                    target_adapters: {
                        let raw: String = row.get("target_adapters")?;
                        serde_json::from_str(&raw).unwrap_or_else(|e| {
                            log::warn!("Failed to parse skill JSON: {}. Falling back to empty.", e);
                            Vec::new()
                        })
                    },
                    target_paths: {
                        let raw: String = row.get("target_paths")?;
                        serde_json::from_str(&raw).unwrap_or_else(|e| {
                            log::warn!("Failed to parse skill JSON: {}. Falling back to empty.", e);
                            Vec::new()
                        })
                    },
                    base_path: row.get("base_path")?,
                })
            })
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    AppError::SkillNotFound { id: id.to_string() }
                }
                _ => AppError::Database(e),
            })?;

        Ok(skill)
    }

    pub async fn create_skill(&self, input: CreateSkillInput) -> Result<Skill> {
        let conn = self.0.lock().await;
        let now = chrono::Utc::now().timestamp();
        let id = input.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let input_schema_json = serde_json::to_string(&input.input_schema)?;
        let target_adapters_json = serde_json::to_string(&input.target_adapters)?;
        let target_paths_json = serde_json::to_string(&input.target_paths)?;

        conn.execute(
            "INSERT INTO skills (id, name, description, instructions, input_schema, enabled, directory_path, entry_point, scope, target_adapters, target_paths, created_at, updated_at, base_path)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                &id,
                &input.name,
                &input.description,
                &input.instructions,
                &input_schema_json,
                &input.enabled,
                &input.directory_path,
                &input.entry_point,
                &input.scope.as_str(),
                &target_adapters_json,
                &target_paths_json,
                &now,
                &now,
                &input.base_path
            ],
        )?;

        drop(conn);
        self.get_skill_by_id(&id).await
    }

    pub async fn update_skill(&self, id: &str, input: UpdateSkillInput) -> Result<Skill> {
        let existing = self.get_skill_by_id(id).await?;
        let conn = self.0.lock().await;

        let name = input.name.unwrap_or(existing.name);
        let description = input.description.unwrap_or(existing.description);
        let instructions = input.instructions.unwrap_or(existing.instructions);
        let input_schema = input.input_schema.unwrap_or(existing.input_schema);
        let enabled = input.enabled.unwrap_or(existing.enabled);
        let directory_path = input.directory_path.unwrap_or(existing.directory_path);
        let entry_point = input.entry_point.unwrap_or(existing.entry_point);
        let scope = input.scope.unwrap_or(existing.scope);
        let target_adapters = input.target_adapters.unwrap_or(existing.target_adapters);
        let target_paths = input.target_paths.unwrap_or(existing.target_paths);
        let base_path = input.base_path.or(existing.base_path);
        let now = chrono::Utc::now().timestamp();
        let input_schema_json = serde_json::to_string(&input_schema)?;
        let target_adapters_json = serde_json::to_string(&target_adapters)?;
        let target_paths_json = serde_json::to_string(&target_paths)?;

        conn.execute(
            "UPDATE skills SET name = ?, description = ?, instructions = ?, input_schema = ?, enabled = ?, directory_path = ?, entry_point = ?, scope = ?, target_adapters = ?, target_paths = ?, updated_at = ?, base_path = ? WHERE id = ?",
            params![
                &name,
                &description,
                &instructions,
                &input_schema_json,
                &enabled,
                &directory_path,
                &entry_point,
                &scope.as_str(),
                &target_adapters_json,
                &target_paths_json,
                &now,
                &base_path,
                &id
            ],
        )?;

        drop(conn);
        self.get_skill_by_id(id).await
    }

    pub async fn delete_skill(&self, id: &str) -> Result<()> {
        let conn = self.0.lock().await;
        conn.execute("DELETE FROM skills WHERE id = ?", params![id])?;
        Ok(())
    }

    pub async fn rule_exists_with_name(&self, name: &str) -> Result<bool> {
        let conn = self.0.lock().await;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM rules WHERE name = ? COLLATE NOCASE",
            params![name],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub async fn command_exists_with_name(&self, name: &str) -> Result<bool> {
        let conn = self.0.lock().await;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM commands WHERE name = ? COLLATE NOCASE",
            params![name],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub async fn skill_exists_with_name(&self, name: &str) -> Result<bool> {
        let conn = self.0.lock().await;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM skills WHERE name = ? COLLATE NOCASE",
            params![name],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub async fn add_execution_log(&self, input: &ExecutionLogInput<'_>) -> Result<()> {
        let conn = self.0.lock().await;
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT INTO execution_logs (id, command_id, command_name, arguments, stdout, stderr, exit_code, duration_ms, executed_at, triggered_by, failure_class, adapter_context, is_redacted, attempt_number)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                id,
                input.command_id,
                input.command_name,
                input.arguments_json,
                input.stdout,
                input.stderr,
                input.exit_code,
                input.duration_ms as i64,
                now,
                input.triggered_by,
                input.failure_class,
                input.adapter_context,
                input.is_redacted as i32,
                input.attempt_number as i32
            ],
        )?;

        Ok(())
    }

    pub async fn add_observability_event(
        &self,
        input: &ObservabilityEventInput<'_>,
    ) -> Result<String> {
        let id = uuid::Uuid::new_v4().to_string();
        let timestamp = chrono::Utc::now();

        // Defense-in-depth: second-pass redaction on all text fields
        let (summary, summary_redacted) = crate::redaction::redact(input.summary);
        let (metadata, metadata_redacted) = match input.metadata {
            Some(m) => {
                let (r, red) = crate::redaction::redact(m);
                (Some(r), red)
            }
            None => (None, false),
        };
        let (stdout, stdout_redacted) = match input.stdout_excerpt {
            Some(s) => {
                let (r, red) = crate::redaction::redact(s);
                (Some(r), red)
            }
            None => (None, false),
        };
        let (stderr, stderr_redacted) = match input.stderr_excerpt {
            Some(s) => {
                let (r, red) = crate::redaction::redact(s);
                (Some(r), red)
            }
            None => (None, false),
        };

        let is_redacted = input.is_redacted
            || summary_redacted
            || metadata_redacted
            || stdout_redacted
            || stderr_redacted;

        let conn = self.0.lock().await; // Re-added this line as it was missing in the provided snippet but needed for `conn.execute`
        conn.execute(
            "INSERT INTO observability_events (
                id, timestamp, event_type, status, source, entity_kind, entity_id, entity_name,
                workspace_path, summary, metadata, stdout_excerpt, stderr_excerpt, duration_ms,
                exit_code, failure_class, attempt_number, is_redacted
             ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                id,
                timestamp.timestamp(), // Use timestamp.timestamp() for i64
                input.event_type.as_str(),
                input.status.as_str(),
                input.source,
                input.entity_kind,
                input.entity_id,
                input.entity_name,
                input.workspace_path,
                summary,  // Use redacted summary
                metadata, // Use redacted metadata
                stdout,   // Use redacted stdout
                stderr,   // Use redacted stderr
                input.duration_ms.map(|value| value as i64),
                input.exit_code,
                input.failure_class,
                input.attempt_number.map(|value| value as i64),
                is_redacted as i32,
            ],
        )?;

        Ok(id)
    }

    pub async fn list_observability_events(
        &self,
        filter: &ObservabilityEventFilter,
    ) -> Result<Vec<ObservabilityEvent>> {
        let conn = self.0.lock().await;

        let mut where_clauses = Vec::new();
        let mut sql_params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(event_type) = &filter.event_type {
            where_clauses.push("event_type = ?".to_string());
            sql_params.push(Box::new(event_type.as_str().to_string()));
        }

        if let Some(status) = &filter.status {
            where_clauses.push("status = ?".to_string());
            sql_params.push(Box::new(status.as_str().to_string()));
        }

        if let Some(source) = filter
            .source
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        {
            where_clauses.push("LOWER(source) LIKE ?".to_string());
            sql_params.push(Box::new(format!("%{}%", source.trim().to_lowercase())));
        }

        if let Some(entity_name) = filter
            .entity_name
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        {
            where_clauses.push("LOWER(COALESCE(entity_name, '')) LIKE ?".to_string());
            sql_params.push(Box::new(format!("%{}%", entity_name.trim().to_lowercase())));
        }

        if let Some(workspace_path) = filter
            .workspace_path
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        {
            where_clauses.push("LOWER(COALESCE(workspace_path, '')) LIKE ?".to_string());
            sql_params.push(Box::new(format!(
                "%{}%",
                workspace_path.trim().to_lowercase()
            )));
        }

        if let Some(query) = filter
            .query
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        {
            let pattern = format!("%{}%", query.trim().to_lowercase());
            where_clauses.push(
                "(LOWER(summary) LIKE ? OR LOWER(source) LIKE ? OR LOWER(COALESCE(entity_name, '')) LIKE ? OR LOWER(COALESCE(workspace_path, '')) LIKE ? OR LOWER(COALESCE(metadata, '')) LIKE ? OR LOWER(COALESCE(stdout_excerpt, '')) LIKE ? OR LOWER(COALESCE(stderr_excerpt, '')) LIKE ?)"
                    .to_string(),
            );
            for _ in 0..7 {
                sql_params.push(Box::new(pattern.clone()));
            }
        }

        if let Some(from_timestamp) = filter.from_timestamp {
            where_clauses.push("timestamp >= ?".to_string());
            sql_params.push(Box::new(from_timestamp.timestamp()));
        }

        if let Some(to_timestamp) = filter.to_timestamp {
            where_clauses.push("timestamp <= ?".to_string());
            sql_params.push(Box::new(to_timestamp.timestamp()));
        }

        let mut sql = "SELECT id, timestamp, event_type, status, source, entity_kind, entity_id, entity_name, workspace_path, summary, metadata, stdout_excerpt, stderr_excerpt, duration_ms, exit_code, failure_class, attempt_number, is_redacted FROM observability_events".to_string();
        if !where_clauses.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&where_clauses.join(" AND "));
        }
        sql.push_str(" ORDER BY timestamp DESC, id DESC LIMIT ?");
        sql_params.push(Box::new(filter.limit.unwrap_or(250).min(1000) as i64));

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt
            .query_map(
                params_from_iter(sql_params.iter()),
                parse_observability_event_row,
            )?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub async fn get_observability_events_by_ids(
        &self,
        ids: &[String],
    ) -> Result<Vec<ObservabilityEvent>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let conn = self.0.lock().await;
        let placeholders = std::iter::repeat_n("?", ids.len())
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!(
            "SELECT id, timestamp, event_type, status, source, entity_kind, entity_id, entity_name, workspace_path, summary, metadata, stdout_excerpt, stderr_excerpt, duration_ms, exit_code, failure_class, attempt_number, is_redacted FROM observability_events WHERE id IN ({placeholders}) ORDER BY timestamp DESC, id DESC"
        );

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt
            .query_map(params_from_iter(ids.iter()), parse_observability_event_row)?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub async fn get_execution_history(&self, limit: u32) -> Result<Vec<ExecutionLog>> {
        let conn = self.0.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, command_id, command_name, arguments, stdout, stderr, exit_code, duration_ms, executed_at, triggered_by, failure_class, adapter_context, is_redacted, attempt_number
             FROM execution_logs
             ORDER BY executed_at DESC
             LIMIT ?",
        )?;

        let rows = stmt
            .query_map(params![limit], |row| {
                let timestamp: i64 = row.get("executed_at")?;
                Ok(ExecutionLog {
                    id: row.get("id")?,
                    command_id: row.get("command_id")?,
                    command_name: row.get("command_name")?,
                    arguments: row.get("arguments")?,
                    stdout: row.get("stdout")?,
                    stderr: row.get("stderr")?,
                    exit_code: row.get("exit_code")?,
                    duration_ms: row.get::<_, i64>("duration_ms")? as u64,
                    executed_at: parse_timestamp_or_now(timestamp),
                    triggered_by: row.get("triggered_by")?,
                    failure_class: row.get("failure_class")?,
                    adapter_context: row.get("adapter_context")?,
                    is_redacted: row.get::<_, i32>("is_redacted")? != 0,
                    attempt_number: row.get::<_, i32>("attempt_number")? as u8,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(rows)
    }

    pub async fn get_execution_history_filtered(
        &self,
        command_id: Option<&str>,
        failure_class: Option<&str>,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<ExecutionLog>> {
        let conn = self.0.lock().await;

        let (sql, params) = {
            let mut where_clauses = Vec::new();
            let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

            if let Some(cid) = command_id {
                where_clauses.push("command_id = ?");
                params.push(Box::new(cid.to_string()));
            }

            if let Some(fc) = failure_class {
                where_clauses.push("failure_class = ?");
                params.push(Box::new(fc.to_string()));
            }

            let mut sql = "SELECT id, command_id, command_name, arguments, stdout, stderr, exit_code, duration_ms, executed_at, triggered_by, failure_class, adapter_context, is_redacted, attempt_number FROM execution_logs".to_string();

            if !where_clauses.is_empty() {
                sql.push_str(&format!(" WHERE {}", where_clauses.join(" AND ")));
            }

            sql.push_str(" ORDER BY executed_at DESC LIMIT ? OFFSET ?");
            params.push(Box::new(limit as i64));
            params.push(Box::new(offset as i64));
            (sql, params)
        };

        let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let mut stmt = conn.prepare(&sql)?;

        let rows = stmt
            .query_map(params_refs.as_slice(), |row| {
                let timestamp: i64 = row.get("executed_at")?;
                Ok(ExecutionLog {
                    id: row.get("id")?,
                    command_id: row.get("command_id")?,
                    command_name: row.get("command_name")?,
                    arguments: row.get("arguments")?,
                    stdout: row.get("stdout")?,
                    stderr: row.get("stderr")?,
                    exit_code: row.get("exit_code")?,
                    duration_ms: row.get::<_, i64>("duration_ms")? as u64,
                    executed_at: parse_timestamp_or_now(timestamp),
                    triggered_by: row.get("triggered_by")?,
                    failure_class: row.get("failure_class")?,
                    adapter_context: row.get("adapter_context")?,
                    is_redacted: row.get::<_, i32>("is_redacted")? != 0,
                    attempt_number: row.get::<_, i32>("attempt_number")? as u8,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(rows)
    }

    pub async fn get_file_hash(&self, file_path: &str) -> Result<Option<String>> {
        let conn = self.0.lock().await;
        let result: Option<String> = conn
            .query_row(
                "SELECT content_hash FROM sync_history WHERE file_path = ?",
                params![file_path],
                |row| row.get(0),
            )
            .optional()?;

        Ok(result)
    }

    pub async fn set_file_hash(&self, file_path: &str, hash: &str) -> Result<()> {
        let conn = self.0.lock().await;
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT OR REPLACE INTO sync_history (file_path, content_hash, last_sync_at)
             VALUES (?, ?, ?)",
            params![file_path, hash, now],
        )?;

        Ok(())
    }

    pub async fn add_sync_log(
        &self,
        files_written: u32,
        status: &str,
        triggered_by: &str,
    ) -> Result<()> {
        let conn = self.0.lock().await;
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT INTO sync_logs (id, timestamp, files_written, status, triggered_by)
             VALUES (?, ?, ?, ?, ?)",
            params![id, now, files_written, status, triggered_by],
        )?;

        Ok(())
    }

    pub async fn get_sync_history(&self, limit: u32) -> Result<Vec<SyncHistoryEntry>> {
        let conn = self.0.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, files_written, status, triggered_by 
             FROM sync_logs 
             ORDER BY timestamp DESC 
             LIMIT ?",
        )?;

        let entries = stmt
            .query_map(params![limit], |row| {
                let id: String = row.get("id")?;
                let timestamp: i64 = row.get("timestamp")?;
                let files_written: u32 = row.get("files_written")?;
                let status: String = row.get("status")?;
                let triggered_by: String = row.get("triggered_by")?;

                Ok(SyncHistoryEntry {
                    id,
                    timestamp: parse_timestamp_or_now(timestamp),
                    files_written,
                    status,
                    triggered_by,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(entries)
    }

    pub async fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let conn = self.0.lock().await;
        let result: Option<String> = conn
            .query_row(
                "SELECT value FROM settings WHERE key = ?",
                params![key],
                |row| row.get(0),
            )
            .optional()?;

        Ok(result)
    }

    pub async fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.0.lock().await;
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)",
            params![key, value],
        )?;
        Ok(())
    }

    pub async fn delete_setting(&self, key: &str) -> Result<()> {
        let conn = self.0.lock().await;
        conn.execute("DELETE FROM settings WHERE key = ?", params![key])?;
        Ok(())
    }

    pub async fn merge_setting_string_array_unique(
        &self,
        key: &str,
        values: &[String],
    ) -> Result<()> {
        let conn = self.0.lock().await;
        let current: Option<String> = conn
            .query_row(
                "SELECT value FROM settings WHERE key = ?",
                params![key],
                |row| row.get(0),
            )
            .optional()?;

        let mut merged: std::collections::HashSet<String> = match current {
            Some(raw) => serde_json::from_str::<Vec<String>>(&raw)
                .unwrap_or_default()
                .into_iter()
                .collect(),
            None => std::collections::HashSet::new(),
        };

        for value in values {
            merged.insert(value.clone());
        }

        let encoded = serde_json::to_string(&merged.into_iter().collect::<Vec<_>>())?;
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)",
            params![key, encoded],
        )?;

        Ok(())
    }

    pub async fn get_all_settings(&self) -> Result<std::collections::HashMap<String, String>> {
        let conn = self.0.lock().await;
        let mut stmt = conn.prepare("SELECT key, value FROM settings")?;

        let settings = stmt
            .query_map([], |row| {
                let key: String = row.get("key")?;
                let value: String = row.get("value")?;
                Ok((key, value))
            })?
            .collect::<std::result::Result<std::collections::HashMap<String, String>, _>>()?;

        Ok(settings)
    }

    pub async fn list_scoped_secret_records(&self) -> Result<Vec<ScopedSecretRecord>> {
        let conn = self.0.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, key, value, scope, workspace_path, artifact_id, created_at, updated_at
             FROM scoped_secrets
             ORDER BY scope, key, workspace_path, artifact_id",
        )?;

        let secrets = stmt
            .query_map([], |row| {
                let scope_raw: String = row.get("scope")?;
                let scope = SecretScope::from_str(&scope_raw).map_err(|err| {
                    rusqlite::Error::FromSqlConversionFailure(
                        3,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            err.to_string(),
                        )),
                    )
                })?;

                Ok(ScopedSecretRecord {
                    id: row.get("id")?,
                    key: row.get("key")?,
                    value: row.get("value")?,
                    scope,
                    workspace_path: row.get("workspace_path")?,
                    artifact_id: row.get("artifact_id")?,
                    created_at: parse_timestamp_or_now(row.get("created_at")?),
                    updated_at: parse_timestamp_or_now(row.get("updated_at")?),
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(secrets)
    }

    pub async fn list_scoped_secrets(&self) -> Result<Vec<ScopedSecret>> {
        Ok(self
            .list_scoped_secret_records()
            .await?
            .into_iter()
            .map(|secret| ScopedSecret {
                id: secret.id,
                key: secret.key,
                value: secret.value,
                scope: secret.scope,
                workspace_path: secret.workspace_path,
                artifact_id: secret.artifact_id,
                created_at: secret.created_at,
                updated_at: secret.updated_at,
            })
            .collect())
    }

    pub async fn upsert_scoped_secret(
        &self,
        input: UpsertScopedSecretInput,
    ) -> Result<ScopedSecret> {
        let conn = self.0.lock().await;
        let now = chrono::Utc::now().timestamp();

        let existing: Option<(String, i64)> = conn
            .query_row(
                "SELECT id, created_at
                 FROM scoped_secrets
                 WHERE scope = ?
                   AND key = ?
                   AND ((workspace_path IS NULL AND ? IS NULL) OR workspace_path = ?)
                   AND ((artifact_id IS NULL AND ? IS NULL) OR artifact_id = ?)",
                params![
                    input.scope.as_str(),
                    input.key,
                    input.workspace_path.as_deref(),
                    input.workspace_path.as_deref(),
                    input.artifact_id.as_deref(),
                    input.artifact_id.as_deref(),
                ],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;

        let (id, created_at) = if let Some((id, created_at)) = existing {
            conn.execute(
                "UPDATE scoped_secrets
                 SET key = ?, value = '', updated_at = ?
                 WHERE id = ?",
                params![input.key, now, id],
            )?;
            (id, created_at)
        } else {
            let id = uuid::Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO scoped_secrets (id, key, value, scope, workspace_path, artifact_id, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    id,
                    input.key,
                    "",
                    input.scope.as_str(),
                    input.workspace_path.as_deref(),
                    input.artifact_id.as_deref(),
                    now,
                    now,
                ],
            )?;
            (id, now)
        };

        Ok(ScopedSecret {
            id,
            key: input.key,
            value: String::new(),
            scope: input.scope,
            workspace_path: input.workspace_path,
            artifact_id: input.artifact_id,
            created_at: parse_timestamp_or_now(created_at),
            updated_at: parse_timestamp_or_now(now),
        })
    }

    pub async fn clear_scoped_secret_plaintext_value(&self, id: &str) -> Result<()> {
        let conn = self.0.lock().await;
        conn.execute(
            "UPDATE scoped_secrets SET value = '', updated_at = ? WHERE id = ?",
            params![chrono::Utc::now().timestamp(), id],
        )?;
        Ok(())
    }

    #[cfg(test)]
    pub async fn force_set_scoped_secret_plaintext_value(
        &self,
        key: &str,
        value: &str,
    ) -> Result<()> {
        let conn = self.0.lock().await;
        conn.execute(
            "UPDATE scoped_secrets SET value = ?, updated_at = ? WHERE key = ?",
            params![value, chrono::Utc::now().timestamp(), key],
        )?;
        Ok(())
    }

    pub async fn delete_scoped_secret(&self, input: DeleteScopedSecretInput) -> Result<()> {
        let conn = self.0.lock().await;
        conn.execute(
            "DELETE FROM scoped_secrets
             WHERE scope = ?
               AND key = ?
               AND ((workspace_path IS NULL AND ? IS NULL) OR workspace_path = ?)
               AND ((artifact_id IS NULL AND ? IS NULL) OR artifact_id = ?)",
            params![
                input.scope.as_str(),
                input.key,
                input.workspace_path.as_deref(),
                input.workspace_path.as_deref(),
                input.artifact_id.as_deref(),
                input.artifact_id.as_deref(),
            ],
        )?;
        Ok(())
    }

    pub async fn get_database_path(&self) -> Result<String> {
        let conn = self.0.lock().await;
        let path: String = conn.query_row("PRAGMA database_list", [], |row| row.get(2))?;
        Ok(path)
    }

    pub async fn update_rule_file_index(
        &self,
        rule_id: &str,
        location: &StorageLocation,
    ) -> Result<()> {
        let conn = self.0.lock().await;
        let file_path = match location {
            StorageLocation::Global => crate::file_storage::get_global_rules_dir()?
                .to_string_lossy()
                .to_string(),
            StorageLocation::Local(path) => path.to_string_lossy().to_string(),
        };

        conn.execute(
            "INSERT OR REPLACE INTO rule_file_index (rule_id, file_path) VALUES (?, ?)",
            params![rule_id, file_path],
        )?;

        Ok(())
    }

    pub async fn get_rule_file_path(&self, rule_id: &str) -> Result<Option<String>> {
        let conn = self.0.lock().await;
        let result: Option<String> = conn
            .query_row(
                "SELECT file_path FROM rule_file_index WHERE rule_id = ?",
                params![rule_id],
                |row| row.get(0),
            )
            .optional()?;

        Ok(result)
    }

    pub async fn remove_rule_file_index(&self, rule_id: &str) -> Result<()> {
        let conn = self.0.lock().await;
        conn.execute(
            "DELETE FROM rule_file_index WHERE rule_id = ?",
            params![rule_id],
        )?;
        Ok(())
    }

    pub async fn import_rule(&self, rule: Rule, mode: crate::models::ImportMode) -> Result<()> {
        let conn = self.0.lock().await;
        let now = chrono::Utc::now().timestamp();

        let target_paths_json = rule
            .target_paths
            .as_ref()
            .map(|p| serde_json::to_string(p).unwrap_or_default());

        let enabled_adapters_json = serde_json::to_string(&rule.enabled_adapters)?;

        let sql = match mode {
            crate::models::ImportMode::Overwrite => {
                log::info!("Import: Overwriting rule {}", rule.id);
                "INSERT OR REPLACE INTO rules (id, name, description, content, scope, target_paths, enabled_adapters, enabled, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
            }
            crate::models::ImportMode::Skip => {
                "INSERT OR IGNORE INTO rules (id, name, description, content, scope, target_paths, enabled_adapters, enabled, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
            }
        };

        conn.execute(
            sql,
            params![
                rule.id,
                rule.name,
                rule.description,
                rule.content,
                rule.scope.as_str(),
                target_paths_json,
                enabled_adapters_json,
                rule.enabled,
                rule.created_at.timestamp(),
                now
            ],
        )?;
        Ok(())
    }

    pub async fn import_command(
        &self,
        command: Command,
        mode: crate::models::ImportMode,
    ) -> Result<()> {
        let conn = self.0.lock().await;
        let now = chrono::Utc::now().timestamp();
        let arguments_json = serde_json::to_string(&command.arguments)?;
        let slash_adapters_json = serde_json::to_string(&command.slash_command_adapters)?;
        let target_paths_json = serde_json::to_string(&command.target_paths)?;

        let sql = match mode {
            crate::models::ImportMode::Overwrite => {
                log::info!("Import: Overwriting command {}", command.id);
                "INSERT OR REPLACE INTO commands (id, name, description, script, arguments, is_placeholder, generate_slash_commands, slash_command_adapters, target_paths, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
            }
            crate::models::ImportMode::Skip => {
                "INSERT OR IGNORE INTO commands (id, name, description, script, arguments, is_placeholder, generate_slash_commands, slash_command_adapters, target_paths, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
            }
        };

        conn.execute(
            sql,
            params![
                command.id,
                command.name,
                command.description,
                command.script,
                arguments_json,
                command.is_placeholder,
                command.generate_slash_commands,
                slash_adapters_json,
                target_paths_json,
                command.created_at.timestamp(),
                now
            ],
        )?;
        Ok(())
    }

    pub async fn import_skill(&self, skill: Skill, mode: crate::models::ImportMode) -> Result<()> {
        let conn = self.0.lock().await;
        let now = chrono::Utc::now().timestamp();
        let input_schema_json = serde_json::to_string(&skill.input_schema)?;
        let target_adapters_json = serde_json::to_string(&skill.target_adapters)?;
        let target_paths_json = serde_json::to_string(&skill.target_paths)?;

        let sql = match mode {
            crate::models::ImportMode::Overwrite => {
                log::info!("Import: Overwriting skill {}", skill.id);
                "INSERT OR REPLACE INTO skills (id, name, description, instructions, input_schema, enabled, directory_path, entry_point, target_adapters, target_paths, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
            }
            crate::models::ImportMode::Skip => {
                "INSERT OR IGNORE INTO skills (id, name, description, instructions, input_schema, enabled, directory_path, entry_point, target_adapters, target_paths, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
            }
        };

        conn.execute(
            sql,
            params![
                skill.id,
                skill.name,
                skill.description,
                skill.instructions,
                input_schema_json,
                skill.enabled,
                skill.directory_path,
                skill.entry_point,
                target_adapters_json,
                target_paths_json,
                skill.created_at.timestamp(),
                now
            ],
        )?;
        Ok(())
    }

    pub async fn import_configuration(
        &self,
        config: crate::models::ExportConfiguration,
        mode: crate::models::ImportMode,
    ) -> Result<()> {
        for rule in config.rules {
            self.import_rule(rule, mode).await?;
        }

        for command in config.commands {
            self.import_command(command, mode).await?;
        }

        for skill in config.skills {
            self.import_skill(skill, mode).await?;
        }
        Ok(())
    }

    pub async fn get_storage_mode(&self) -> Result<String> {
        let mode = self.get_setting("storage_mode").await?;
        Ok(mode.unwrap_or_else(|| "sqlite".to_string()))
    }

    pub async fn set_storage_mode(&self, mode: &str) -> Result<()> {
        self.set_setting("storage_mode", mode).await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn log_reconciliation(
        &self,
        operation: ReconcileOperation,
        artifact_type: Option<&str>,
        adapter: Option<AdapterType>,
        scope: Option<Scope>,
        path: &str,
        result: ReconcileResultType,
        error_message: Option<&str>,
    ) -> Result<()> {
        let conn = self.0.lock().await;
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT INTO reconciliation_logs (id, timestamp, operation, artifact_type, adapter, scope, path, result, error_message)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                id,
                now,
                operation.as_str(),
                artifact_type,
                adapter.map(|a| a.as_str()),
                scope.map(|s| s.as_str()),
                path,
                result.as_str(),
                error_message
            ],
        )?;

        Ok(())
    }

    pub async fn get_reconciliation_logs(&self, limit: i64) -> Result<Vec<ReconciliationLogEntry>> {
        let conn = self.0.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, operation, artifact_type, adapter, scope, path, result, error_message
             FROM reconciliation_logs
             ORDER BY timestamp DESC
             LIMIT ?",
        )?;

        let logs = stmt
            .query_map(rusqlite::params![limit], |row| {
                let op_str: String = row.get("operation")?;
                let operation =
                    ReconcileOperation::from_str(&op_str).unwrap_or(ReconcileOperation::Check);

                let adapter_str: Option<String> = row.get("adapter")?;
                let adapter = adapter_str.and_then(|s| AdapterType::from_str(&s).ok());

                let scope_str: Option<String> = row.get("scope")?;
                let scope = scope_str.and_then(|s| Scope::from_str(&s).ok());

                let res_str: String = row.get("result")?;
                let result =
                    ReconcileResultType::from_str(&res_str).unwrap_or(ReconcileResultType::Failed);

                Ok(ReconciliationLogEntry {
                    id: row.get("id")?,
                    timestamp: parse_timestamp_or_now(row.get("timestamp")?),
                    operation,
                    artifact_type: row.get("artifact_type")?,
                    adapter,
                    scope,
                    path: row.get("path")?,
                    result,
                    error_message: row.get("error_message")?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(logs)
    }

    pub async fn get_last_reconciliation_op_per_path(
        &self,
    ) -> Result<std::collections::HashMap<String, (String, DateTime<Utc>)>> {
        let conn = self.0.lock().await;
        let mut stmt = conn.prepare(
            "SELECT path, operation, timestamp 
             FROM reconciliation_logs 
             WHERE id IN (SELECT MAX(id) FROM reconciliation_logs GROUP BY path)",
        )?;

        let rows = stmt.query_map([], |row| {
            let path: String = row.get("path")?;
            let operation: String = row.get("operation")?;
            let timestamp: DateTime<Utc> = parse_timestamp_or_now(row.get("timestamp")?);
            Ok((path, (operation, timestamp)))
        })?;

        let mut ops = std::collections::HashMap::new();
        for (path, (operation, timestamp)) in rows.flatten() {
            ops.insert(path, (operation, timestamp));
        }

        Ok(ops)
    }

    pub async fn clear_reconciliation_logs(&self) -> Result<()> {
        let conn = self.0.lock().await;
        conn.execute("DELETE FROM reconciliation_logs", [])?;
        Ok(())
    }

    pub async fn upsert_sync_manifest(
        &self,
        input: CreateSyncManifestInput,
    ) -> Result<SyncManifestEntry> {
        let conn = self.0.lock().await;
        let id = input
            .id
            .clone()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT INTO sync_manifest (id, path, artifact_id, artifact_type, adapter, scope, written_at, content_hash)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(path) DO UPDATE SET
                 artifact_id = excluded.artifact_id,
                 artifact_type = excluded.artifact_type,
                 adapter = excluded.adapter,
                 scope = excluded.scope,
                 written_at = excluded.written_at,
                 content_hash = excluded.content_hash",
            rusqlite::params![
                id,
                input.path,
                input.artifact_id,
                input.artifact_type.as_str(),
                input.adapter.as_str(),
                input.scope.as_str(),
                now,
                input.content_hash
            ],
        )?;

        Ok(SyncManifestEntry {
            id,
            path: input.path,
            artifact_id: input.artifact_id,
            artifact_type: input.artifact_type,
            adapter: input.adapter,
            scope: input.scope,
            written_at: chrono::Utc::now(),
            content_hash: input.content_hash,
        })
    }

    pub async fn get_sync_manifest_by_path(&self, path: &str) -> Result<Option<SyncManifestEntry>> {
        let conn = self.0.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, path, artifact_id, artifact_type, adapter, scope, written_at, content_hash
             FROM sync_manifest WHERE path = ?1",
        )?;

        let result = stmt
            .query_row(rusqlite::params![path], |row| {
                Ok(SyncManifestEntry {
                    id: row.get("id")?,
                    path: row.get("path")?,
                    artifact_id: row.get("artifact_id")?,
                    artifact_type: row
                        .get::<_, String>("artifact_type")?
                        .parse()
                        .map_err(|e| {
                            rusqlite::Error::FromSqlConversionFailure(
                                3,
                                rusqlite::types::Type::Text,
                                Box::new(std::io::Error::new(
                                    std::io::ErrorKind::InvalidData,
                                    format!("Invalid artifact_type: {}", e),
                                )),
                            )
                        })?,
                    adapter: row.get::<_, String>("adapter")?.parse().map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            4,
                            rusqlite::types::Type::Text,
                            Box::new(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                format!("Invalid adapter: {}", e),
                            )),
                        )
                    })?,
                    scope: row.get::<_, String>("scope")?.parse().map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            5,
                            rusqlite::types::Type::Text,
                            Box::new(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                format!("Invalid scope: {}", e),
                            )),
                        )
                    })?,
                    written_at: parse_timestamp_or_now(row.get("written_at")?),
                    content_hash: row.get("content_hash")?,
                })
            })
            .optional()?;

        Ok(result)
    }

    pub async fn list_sync_manifest(
        &self,
        filter: SyncManifestFilter,
    ) -> Result<Vec<SyncManifestEntry>> {
        let conn = self.0.lock().await;

        let mut sql = "SELECT id, path, artifact_id, artifact_type, adapter, scope, written_at, content_hash FROM sync_manifest WHERE 1=1".to_string();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(adapter) = &filter.adapter {
            sql.push_str(" AND adapter = ?");
            params_vec.push(Box::new(adapter.as_str().to_string()));
        }
        if let Some(artifact_type) = &filter.artifact_type {
            sql.push_str(" AND artifact_type = ?");
            params_vec.push(Box::new(artifact_type.as_str().to_string()));
        }
        if let Some(artifact_id) = &filter.artifact_id {
            sql.push_str(" AND artifact_id = ?");
            params_vec.push(Box::new(artifact_id.clone()));
        }
        if let Some(scope) = &filter.scope {
            sql.push_str(" AND scope = ?");
            params_vec.push(Box::new(scope.as_str().to_string()));
        }

        let params: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();

        let mut stmt = conn.prepare(&sql)?;
        let entries = stmt
            .query_map(params.as_slice(), |row| {
                Ok(SyncManifestEntry {
                    id: row.get("id")?,
                    path: row.get("path")?,
                    artifact_id: row.get("artifact_id")?,
                    artifact_type: row
                        .get::<_, String>("artifact_type")?
                        .parse()
                        .map_err(|e| {
                            rusqlite::Error::FromSqlConversionFailure(
                                3,
                                rusqlite::types::Type::Text,
                                Box::new(std::io::Error::new(
                                    std::io::ErrorKind::InvalidData,
                                    format!("Invalid artifact_type: {}", e),
                                )),
                            )
                        })?,
                    adapter: row.get::<_, String>("adapter")?.parse().map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            4,
                            rusqlite::types::Type::Text,
                            Box::new(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                format!("Invalid adapter: {}", e),
                            )),
                        )
                    })?,
                    scope: row.get::<_, String>("scope")?.parse().map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            5,
                            rusqlite::types::Type::Text,
                            Box::new(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                format!("Invalid scope: {}", e),
                            )),
                        )
                    })?,
                    written_at: parse_timestamp_or_now(row.get("written_at")?),
                    content_hash: row.get("content_hash")?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(entries)
    }

    pub async fn delete_sync_manifest_by_path(&self, path: &str) -> Result<()> {
        let conn = self.0.lock().await;
        conn.execute(
            "DELETE FROM sync_manifest WHERE path = ?1",
            rusqlite::params![path],
        )?;
        Ok(())
    }

    pub async fn delete_sync_manifest_by_filter(
        &self,
        filter: SyncManifestFilter,
    ) -> Result<usize> {
        let conn = self.0.lock().await;

        let mut sql = "DELETE FROM sync_manifest WHERE 1=1".to_string();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(adapter) = &filter.adapter {
            sql.push_str(" AND adapter = ?");
            params_vec.push(Box::new(adapter.as_str().to_string()));
        }
        if let Some(artifact_type) = &filter.artifact_type {
            sql.push_str(" AND artifact_type = ?");
            params_vec.push(Box::new(artifact_type.as_str().to_string()));
        }
        if let Some(artifact_id) = &filter.artifact_id {
            sql.push_str(" AND artifact_id = ?");
            params_vec.push(Box::new(artifact_id.clone()));
        }
        if let Some(scope) = &filter.scope {
            sql.push_str(" AND scope = ?");
            params_vec.push(Box::new(scope.as_str().to_string()));
        }

        let params: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
        let count = conn.execute(&sql, params.as_slice())?;

        Ok(count)
    }

    pub async fn get_all_tool_sync_preferences(&self) -> Result<Vec<ToolSyncPreferences>> {
        let conn = self.0.lock().await;
        let mut stmt = conn.prepare(
            "SELECT tool_id, sync_rules, sync_commands, sync_skills FROM tool_sync_preferences",
        )?;

        let prefs = stmt
            .query_map([], |row| {
                let tool_id_str: String = row.get("tool_id")?;
                let tool_id = AdapterType::from_str(&tool_id_str).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("Invalid tool_id: {}", e),
                        )),
                    )
                })?;
                Ok(ToolSyncPreferences {
                    tool_id,
                    sync_rules: row.get::<_, i32>("sync_rules")? != 0,
                    sync_commands: row.get::<_, i32>("sync_commands")? != 0,
                    sync_skills: row.get::<_, i32>("sync_skills")? != 0,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(prefs)
    }

    pub async fn get_tool_sync_preferences(
        &self,
        tool_id: &AdapterType,
    ) -> Result<Option<ToolSyncPreferences>> {
        let conn = self.0.lock().await;
        let mut stmt = conn.prepare(
            "SELECT tool_id, sync_rules, sync_commands, sync_skills FROM tool_sync_preferences WHERE tool_id = ?1",
        )?;

        let result = stmt
            .query_row(rusqlite::params![tool_id.as_str()], |row| {
                let tool_id_str: String = row.get("tool_id")?;
                let tool_id = AdapterType::from_str(&tool_id_str).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("Invalid tool_id: {}", e),
                        )),
                    )
                })?;
                Ok(ToolSyncPreferences {
                    tool_id,
                    sync_rules: row.get::<_, i32>("sync_rules")? != 0,
                    sync_commands: row.get::<_, i32>("sync_commands")? != 0,
                    sync_skills: row.get::<_, i32>("sync_skills")? != 0,
                })
            })
            .optional()?;

        Ok(result)
    }

    pub async fn upsert_tool_sync_preferences(
        &self,
        input: UpsertToolSyncPreferencesInput,
    ) -> Result<ToolSyncPreferences> {
        let conn = self.0.lock().await;

        let sync_rules = input.sync_rules.unwrap_or(true) as i32;
        let sync_commands = input.sync_commands.unwrap_or(true) as i32;
        let sync_skills = input.sync_skills.unwrap_or(true) as i32;

        conn.execute(
            "INSERT INTO tool_sync_preferences (tool_id, sync_rules, sync_commands, sync_skills)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(tool_id) DO UPDATE SET
                 sync_rules = excluded.sync_rules,
                 sync_commands = excluded.sync_commands,
                 sync_skills = excluded.sync_skills",
            rusqlite::params![
                input.tool_id.as_str(),
                sync_rules,
                sync_commands,
                sync_skills
            ],
        )?;

        Ok(ToolSyncPreferences {
            tool_id: input.tool_id,
            sync_rules: sync_rules != 0,
            sync_commands: sync_commands != 0,
            sync_skills: sync_skills != 0,
        })
    }
}

fn run_migrations(conn: &mut Connection) -> Result<()> {
    let transaction = conn.transaction()?;

    let current_version: i32 = transaction
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .unwrap_or(0);

    if current_version < 1 {
        transaction.execute(
            "CREATE TABLE IF NOT EXISTS rules (
                id TEXT PRIMARY KEY NOT NULL,
                name TEXT NOT NULL,
                content TEXT NOT NULL,
                scope TEXT NOT NULL,
                target_paths TEXT,
                enabled_adapters TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )?;

        transaction.execute(
            "CREATE TABLE IF NOT EXISTS sync_history (
                file_path TEXT PRIMARY KEY NOT NULL,
                content_hash TEXT NOT NULL,
                last_sync_at INTEGER NOT NULL
            )",
            [],
        )?;

        transaction.execute(
            "CREATE TABLE IF NOT EXISTS sync_logs (
                id TEXT PRIMARY KEY NOT NULL,
                timestamp INTEGER NOT NULL,
                files_written INTEGER NOT NULL,
                status TEXT NOT NULL,
                triggered_by TEXT NOT NULL
            )",
            [],
        )?;

        transaction.execute(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY NOT NULL,
                value TEXT NOT NULL
            )",
            [],
        )?;

        transaction.execute(
            "CREATE INDEX IF NOT EXISTS idx_rules_scope ON rules(scope)",
            [],
        )?;
    }

    if current_version < 2 {
        transaction.execute(
            "CREATE INDEX IF NOT EXISTS idx_sync_logs_timestamp ON sync_logs(timestamp)",
            [],
        )?;
    }

    if current_version < 3 {
        transaction.execute(
            "CREATE TABLE IF NOT EXISTS rule_file_index (
                rule_id TEXT PRIMARY KEY NOT NULL,
                file_path TEXT NOT NULL,
                content_hash TEXT,
                last_modified INTEGER
            )",
            [],
        )?;

        transaction.execute(
            "CREATE INDEX IF NOT EXISTS idx_rule_file_index_path ON rule_file_index(file_path)",
            [],
        )?;
    }

    if current_version < 4 {
        transaction.execute(
            "CREATE TABLE IF NOT EXISTS commands (
                id TEXT PRIMARY KEY NOT NULL,
                name TEXT NOT NULL,
                description TEXT NOT NULL,
                script TEXT NOT NULL,
                arguments TEXT NOT NULL,
                expose_via_mcp INTEGER NOT NULL DEFAULT 1,
                target_paths TEXT NOT NULL DEFAULT '[]',
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )?;

        transaction.execute(
            "CREATE INDEX IF NOT EXISTS idx_commands_updated_at ON commands(updated_at)",
            [],
        )?;

        transaction.execute(
            "CREATE TABLE IF NOT EXISTS execution_logs (
                id TEXT PRIMARY KEY NOT NULL,
                command_id TEXT NOT NULL,
                command_name TEXT NOT NULL,
                arguments TEXT NOT NULL,
                stdout TEXT NOT NULL,
                stderr TEXT NOT NULL,
                exit_code INTEGER NOT NULL,
                duration_ms INTEGER NOT NULL,
                executed_at INTEGER NOT NULL,
                triggered_by TEXT NOT NULL
            )",
            [],
        )?;

        transaction.execute(
            "CREATE INDEX IF NOT EXISTS idx_execution_logs_executed_at ON execution_logs(executed_at)",
            [],
        )?;
    }

    if current_version < 5 {
        transaction.execute(
            "CREATE TABLE IF NOT EXISTS skills (
                id TEXT PRIMARY KEY NOT NULL,
                name TEXT NOT NULL,
                description TEXT NOT NULL,
                instructions TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )?;

        transaction.execute(
            "CREATE INDEX IF NOT EXISTS idx_skills_updated_at ON skills(updated_at)",
            [],
        )?;
    }

    if current_version < 6 {
        let mut stmt = transaction.prepare("PRAGMA table_info(skills)")?;
        let cols: Vec<String> = stmt
            .query_map([], |row| row.get(1))?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        if !cols.iter().any(|c| c == "input_schema") {
            transaction.execute(
                "ALTER TABLE skills ADD COLUMN input_schema TEXT NOT NULL DEFAULT '[]'",
                [],
            )?;
        }
    }

    if current_version < 7 {
        let mut stmt = transaction.prepare("PRAGMA table_info(skills)")?;
        let cols: Vec<String> = stmt
            .query_map([], |row| row.get(1))?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        if !cols.iter().any(|c| c == "directory_path") {
            transaction.execute(
                "ALTER TABLE skills ADD COLUMN directory_path TEXT NOT NULL DEFAULT ''",
                [],
            )?;
        }
        if !cols.iter().any(|c| c == "entry_point") {
            transaction.execute(
                "ALTER TABLE skills ADD COLUMN entry_point TEXT NOT NULL DEFAULT ''",
                [],
            )?;
        }
    }

    if current_version < 8 {
        let mut stmt = transaction.prepare("PRAGMA table_info(skills)")?;
        let cols: Vec<String> = stmt
            .query_map([], |row| row.get(1))?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        if !cols.iter().any(|c| c == "scope") {
            transaction.execute(
                "ALTER TABLE skills ADD COLUMN scope TEXT NOT NULL DEFAULT 'global'",
                [],
            )?;
        }
    }

    if current_version < 9 {
        // Add slash command support columns to commands table
        let mut stmt = transaction.prepare("PRAGMA table_info(commands)")?;
        let cols: Vec<String> = stmt
            .query_map([], |row| row.get(1))?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        if !cols.iter().any(|c| c == "generate_slash_commands") {
            transaction.execute(
                "ALTER TABLE commands ADD COLUMN generate_slash_commands INTEGER NOT NULL DEFAULT 0",
                [],
            )?;
        }
        if !cols.iter().any(|c| c == "slash_command_adapters") {
            transaction.execute(
                "ALTER TABLE commands ADD COLUMN slash_command_adapters TEXT NOT NULL DEFAULT '[]'",
                [],
            )?;
        }
    }

    if current_version < 10 {
        let mut stmt = transaction.prepare("PRAGMA table_info(commands)")?;
        let cols: Vec<String> = stmt
            .query_map([], |row| row.get(1))?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        if !cols.iter().any(|c| c == "target_paths") {
            transaction.execute(
                "ALTER TABLE commands ADD COLUMN target_paths TEXT NOT NULL DEFAULT '[]'",
                [],
            )?;
        }
    }

    if current_version < 11 {
        let mut stmt = transaction.prepare("PRAGMA table_info(rules)")?;
        let cols: Vec<String> = stmt
            .query_map([], |row| row.get(1))?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        if !cols.iter().any(|c| c == "description") {
            transaction.execute(
                "ALTER TABLE rules ADD COLUMN description TEXT NOT NULL DEFAULT ''",
                [],
            )?;
        }

        let mut stmt = transaction.prepare("PRAGMA table_info(commands)")?;
        let cols: Vec<String> = stmt
            .query_map([], |row| row.get(1))?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        if !cols.iter().any(|c| c == "is_placeholder") {
            transaction.execute(
                "ALTER TABLE commands ADD COLUMN is_placeholder INTEGER NOT NULL DEFAULT 0",
                [],
            )?;
        }
    }

    if current_version < 12 {
        transaction.execute(
            "CREATE TABLE IF NOT EXISTS reconciliation_logs (
                id TEXT PRIMARY KEY NOT NULL,
                timestamp INTEGER NOT NULL,
                operation TEXT NOT NULL,
                artifact_type TEXT,
                adapter TEXT,
                scope TEXT,
                path TEXT NOT NULL,
                result TEXT NOT NULL,
                error_message TEXT
            )",
            [],
        )?;

        transaction.execute(
            "CREATE INDEX IF NOT EXISTS idx_reconciliation_logs_timestamp ON reconciliation_logs(timestamp)",
            [],
        )?;
    }

    if current_version < 13 {
        add_column_if_missing(
            &transaction,
            "skills",
            "target_adapters",
            "TEXT NOT NULL DEFAULT '[]'",
        )?;
        add_column_if_missing(
            &transaction,
            "skills",
            "target_paths",
            "TEXT NOT NULL DEFAULT '[]'",
        )?;
    }

    if current_version < 14 {
        add_column_if_missing(&transaction, "commands", "timeout_ms", "INTEGER")?;
        add_column_if_missing(&transaction, "commands", "max_retries", "INTEGER")?;
    }

    if current_version < 15 {
        add_column_if_missing(&transaction, "execution_logs", "failure_class", "TEXT")?;
        add_column_if_missing(&transaction, "execution_logs", "adapter_context", "TEXT")?;
        add_column_if_missing(
            &transaction,
            "execution_logs",
            "is_redacted",
            "INTEGER NOT NULL DEFAULT 0",
        )?;
        add_column_if_missing(
            &transaction,
            "execution_logs",
            "attempt_number",
            "INTEGER NOT NULL DEFAULT 1",
        )?;

        transaction.execute(
            "CREATE INDEX IF NOT EXISTS idx_execution_logs_command_id ON execution_logs(command_id)",
            [],
        )?;
    }

    if current_version < 16 {
        add_column_if_missing(&transaction, "commands", "base_path", "TEXT")?;
        add_column_if_missing(&transaction, "skills", "base_path", "TEXT")?;
    }

    if current_version < 17 {
        transaction.execute(
            "CREATE TABLE IF NOT EXISTS scoped_secrets (
                id TEXT PRIMARY KEY NOT NULL,
                key TEXT NOT NULL COLLATE NOCASE,
                value TEXT NOT NULL,
                scope TEXT NOT NULL,
                workspace_path TEXT,
                artifact_id TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )?;

        transaction.execute(
            "CREATE INDEX IF NOT EXISTS idx_scoped_secrets_scope ON scoped_secrets(scope, workspace_path, artifact_id)",
            [],
        )?;
        transaction.execute(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_scoped_secrets_lookup
             ON scoped_secrets(scope, key, ifnull(workspace_path, ''), ifnull(artifact_id, ''))",
            [],
        )?;
    }

    if current_version < 19 {
        transaction.execute(
            "CREATE TABLE IF NOT EXISTS observability_events (
                id TEXT PRIMARY KEY NOT NULL,
                timestamp INTEGER NOT NULL,
                event_type TEXT NOT NULL,
                status TEXT NOT NULL,
                source TEXT NOT NULL,
                entity_kind TEXT,
                entity_id TEXT,
                entity_name TEXT,
                summary TEXT NOT NULL,
                metadata TEXT,
                stdout_excerpt TEXT,
                stderr_excerpt TEXT,
                duration_ms INTEGER,
                exit_code INTEGER,
                failure_class TEXT,
                attempt_number INTEGER,
                is_redacted INTEGER NOT NULL DEFAULT 0
            )",
            [],
        )?;
        transaction.execute(
            "CREATE INDEX IF NOT EXISTS idx_observability_events_timestamp
             ON observability_events(timestamp DESC)",
            [],
        )?;
        transaction.execute(
            "CREATE INDEX IF NOT EXISTS idx_observability_events_type_status
             ON observability_events(event_type, status, timestamp DESC)",
            [],
        )?;
        transaction.execute(
            "CREATE INDEX IF NOT EXISTS idx_observability_events_entity_name
             ON observability_events(entity_name)",
            [],
        )?;
    }

    if current_version < 20 {
        transaction.execute(
            "ALTER TABLE observability_events ADD COLUMN workspace_path TEXT",
            [],
        )?;
        transaction.execute(
            "CREATE INDEX IF NOT EXISTS idx_observability_events_workspace_path
             ON observability_events(workspace_path)",
            [],
        )?;
    }

    if current_version < 21 {
        let mut stmt = transaction.prepare("PRAGMA table_info(commands)")?;
        let cols: Vec<String> = stmt
            .query_map([], |row| row.get(1))?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        if cols.iter().any(|c| c == "expose_via_mcp") {
            transaction.execute("ALTER TABLE commands DROP COLUMN expose_via_mcp", [])?;
        }
    }

    if current_version < 22 {
        transaction.execute(
            "CREATE TABLE IF NOT EXISTS sync_manifest (
                id TEXT PRIMARY KEY NOT NULL,
                path TEXT NOT NULL UNIQUE,
                artifact_id TEXT NOT NULL,
                artifact_type TEXT NOT NULL,
                adapter TEXT NOT NULL,
                scope TEXT NOT NULL,
                written_at INTEGER NOT NULL,
                content_hash TEXT NOT NULL
            )",
            [],
        )?;
        transaction.execute(
            "CREATE INDEX IF NOT EXISTS idx_sync_manifest_path ON sync_manifest(path)",
            [],
        )?;
        transaction.execute(
            "CREATE INDEX IF NOT EXISTS idx_sync_manifest_artifact_id ON sync_manifest(artifact_id)",
            [],
        )?;
        transaction.execute(
            "CREATE INDEX IF NOT EXISTS idx_sync_manifest_adapter ON sync_manifest(adapter)",
            [],
        )?;
    }

    if current_version < 23 {
        transaction.execute(
            "CREATE TABLE IF NOT EXISTS tool_sync_preferences (
                tool_id TEXT PRIMARY KEY NOT NULL,
                sync_rules INTEGER NOT NULL DEFAULT 1,
                sync_commands INTEGER NOT NULL DEFAULT 1,
                sync_skills INTEGER NOT NULL DEFAULT 1
            )",
            [],
        )?;
    }

    transaction.execute("PRAGMA user_version = 23", [])?;
    transaction.commit()?;

    Ok(())
}

fn add_column_if_missing(
    transaction: &rusqlite::Transaction,
    table: &str,
    column: &str,
    definition: &str,
) -> Result<()> {
    let mut stmt = transaction.prepare(&format!("PRAGMA table_info({})", table))?;
    let exists = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .any(|c| c.as_ref().map(|s| s == column).unwrap_or(false));

    if !exists {
        transaction.execute(
            &format!("ALTER TABLE {} ADD COLUMN {} {}", table, column, definition),
            [],
        )?;
    }
    Ok(())
}

pub fn get_app_data_path(app_handle: &tauri::AppHandle) -> Result<PathBuf> {
    app_handle
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Path(e.to_string()))
}

pub fn default_app_data_dir() -> Result<PathBuf> {
    let base = dirs::data_local_dir()
        .or_else(dirs::data_dir)
        .ok_or_else(|| AppError::Path("Could not determine data directory".to_string()))?;
    Ok(base.join("RuleWeaver"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        CreateCommandInput, CreateSkillInput, ObservabilityEventStatus, ObservabilityEventType,
        SkillParameter, SkillParameterType, UpdateCommandInput, UpdateSkillInput,
    };

    #[tokio::test]
    async fn test_skill_crud() {
        let db = Database::new_in_memory().await.unwrap();

        // 1. Create
        let input = CreateSkillInput {
            id: None,
            name: "Test Skill".to_string(),
            description: "A test skill".to_string(),
            instructions: "echo 'hello'".to_string(),
            input_schema: vec![SkillParameter {
                name: "param1".to_string(),
                description: "desc".to_string(),
                param_type: SkillParameterType::String,
                required: true,
                default_value: None,
                enum_values: None,
            }],
            directory_path: "/test/path".to_string(),
            entry_point: "main.sh".to_string(),
            scope: Scope::Global,
            enabled: true,
            ..Default::default()
        };

        let created = db.create_skill(input.clone()).await.unwrap();
        assert_eq!(created.name, "Test Skill");
        assert_eq!(created.input_schema.len(), 1);
        assert_eq!(created.directory_path, "/test/path");

        // 2. Read
        let fetched = db.get_skill_by_id(&created.id).await.unwrap();
        assert_eq!(created.id, fetched.id);
        assert_eq!(fetched.entry_point, "main.sh");

        let all = db.get_all_skills().await.unwrap();
        assert_eq!(all.len(), 1);

        // 3. Update
        let update_input = UpdateSkillInput {
            name: Some("Updated Skill".to_string()),
            ..Default::default()
        };
        let updated = db.update_skill(&created.id, update_input).await.unwrap();
        assert_eq!(updated.name, "Updated Skill");
        // Ensure other fields remain unchanged
        assert_eq!(updated.directory_path, "/test/path");

        // 4. Delete
        db.delete_skill(&created.id).await.unwrap();
        assert!(db.get_skill_by_id(&created.id).await.is_err());
        assert_eq!(db.get_all_skills().await.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_command_target_paths_roundtrip() {
        let db = Database::new_in_memory().await.unwrap();

        let created = db
            .create_command(CreateCommandInput {
                id: None,
                name: "Build".to_string(),
                description: "Run build".to_string(),
                script: "npm run build".to_string(),
                arguments: vec![],
                is_placeholder: false,
                generate_slash_commands: false,
                slash_command_adapters: vec![],
                target_paths: vec!["C:/repo-a".to_string()],
                base_path: None,
                timeout_ms: None,
                max_retries: None,
            })
            .await
            .unwrap();

        assert_eq!(created.target_paths, vec!["C:/repo-a".to_string()]);

        let updated = db
            .update_command(
                &created.id,
                UpdateCommandInput {
                    target_paths: Some(vec!["C:/repo-b".to_string(), "C:/repo-c".to_string()]),
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        assert_eq!(
            updated.target_paths,
            vec!["C:/repo-b".to_string(), "C:/repo-c".to_string()]
        );
    }

    #[tokio::test]
    async fn test_scoped_secret_crud_roundtrip() {
        let db = Database::new_in_memory().await.unwrap();

        let created = db
            .upsert_scoped_secret(UpsertScopedSecretInput {
                key: "PROJECT_API_KEY".to_string(),
                value: "repo-a".to_string(),
                scope: SecretScope::Workspace,
                workspace_path: Some("c:/repo-a".to_string()),
                artifact_id: None,
            })
            .await
            .unwrap();
        assert_eq!(created.key, "PROJECT_API_KEY");

        let all = db.list_scoped_secrets().await.unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].value, "");
        assert_eq!(all[0].workspace_path.as_deref(), Some("c:/repo-a"));

        let updated = db
            .upsert_scoped_secret(UpsertScopedSecretInput {
                key: "project_api_key".to_string(),
                value: "repo-b".to_string(),
                scope: SecretScope::Workspace,
                workspace_path: Some("c:/repo-a".to_string()),
                artifact_id: None,
            })
            .await
            .unwrap();
        assert_eq!(updated.id, created.id);
        assert_eq!(updated.value, "");

        db.delete_scoped_secret(DeleteScopedSecretInput {
            key: "PROJECT_API_KEY".to_string(),
            scope: SecretScope::Workspace,
            workspace_path: Some("c:/repo-a".to_string()),
            artifact_id: None,
        })
        .await
        .unwrap();

        assert!(db.list_scoped_secrets().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_observability_events_filtering_and_lookup() {
        let db = Database::new_in_memory().await.unwrap();

        db.add_observability_event(&ObservabilityEventInput {
            event_type: ObservabilityEventType::CommandRun,
            status: ObservabilityEventStatus::Error,
            source: "command-test",
            entity_kind: Some("command"),
            entity_id: Some("cmd-1"),
            entity_name: Some("Deploy docs"),
            workspace_path: Some("c:/repos/docs"),
            summary: "Command execution failed",
            metadata: Some("{\"toolName\":\"Deploy docs\"}"),
            stdout_excerpt: None,
            stderr_excerpt: Some("timeout"),
            duration_ms: Some(500),
            exit_code: Some(1),
            failure_class: Some("timeout"),
            attempt_number: Some(1),
            is_redacted: true,
        })
        .await
        .unwrap();

        db.add_observability_event(&ObservabilityEventInput {
            event_type: ObservabilityEventType::SkillRun,
            status: ObservabilityEventStatus::Success,
            source: "skill-runner",
            entity_kind: Some("skill"),
            entity_id: Some("skill-1"),
            entity_name: Some("Summarize Repo"),
            workspace_path: Some("c:/repos/app"),
            summary: "Skill execution succeeded",
            metadata: Some("{\"triggeredBy\":\"skill-runner\"}"),
            stdout_excerpt: None,
            stderr_excerpt: None,
            duration_ms: Some(220),
            exit_code: Some(0),
            failure_class: None,
            attempt_number: Some(1),
            is_redacted: false,
        })
        .await
        .unwrap();

        let filtered = db
            .list_observability_events(&ObservabilityEventFilter {
                event_type: Some(ObservabilityEventType::CommandRun),
                status: Some(ObservabilityEventStatus::Error),
                entity_name: Some("deploy".to_string()),
                workspace_path: Some("repos/docs".to_string()),
                query: Some("timeout".to_string()),
                limit: Some(25),
                ..Default::default()
            })
            .await
            .unwrap();

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].entity_name.as_deref(), Some("Deploy docs"));
        assert_eq!(filtered[0].workspace_path.as_deref(), Some("c:/repos/docs"));
        assert!(filtered[0].is_redacted);

        let looked_up = db
            .get_observability_events_by_ids(&[filtered[0].id.clone()])
            .await
            .unwrap();
        assert_eq!(looked_up.len(), 1);
        assert_eq!(looked_up[0].summary, "Command execution failed");
    }

    #[tokio::test]
    async fn test_observability_export_writes_redacted_entries() {
        let db = Database::new_in_memory().await.unwrap();

        db.add_observability_event(&ObservabilityEventInput {
            event_type: ObservabilityEventType::SkillRun,
            status: ObservabilityEventStatus::Success,
            source: "skill-runner",
            entity_kind: Some("skill"),
            entity_id: Some("skill-1"),
            entity_name: Some("Summarize Repo"),
            workspace_path: Some("c:/repos/app"),
            summary: "Skill execution succeeded",
            metadata: Some("{\"triggeredBy\":\"skill-runner\"}"),
            stdout_excerpt: Some("token=***REDACTED***"),
            stderr_excerpt: None,
            duration_ms: Some(220),
            exit_code: Some(0),
            failure_class: None,
            attempt_number: Some(1),
            is_redacted: true,
        })
        .await
        .unwrap();

        let export_path = std::env::temp_dir().join(format!(
            "ruleweaver-observability-{}.json",
            uuid::Uuid::new_v4()
        ));

        crate::observability::export_events(
            &db,
            &export_path,
            None,
            &ObservabilityEventFilter {
                event_type: Some(ObservabilityEventType::SkillRun),
                limit: Some(25),
                ..Default::default()
            },
        )
        .await
        .unwrap();

        let export_body = std::fs::read_to_string(&export_path).unwrap();
        std::fs::remove_file(&export_path).unwrap();

        assert!(export_body.contains("***REDACTED***"));
        assert!(export_body.contains("\"eventCount\": 1"));
        assert!(export_body.contains("\"isRedacted\": true"));
    }

    #[tokio::test]
    async fn test_sync_manifest_crud() {
        let db = Database::new_in_memory().await.unwrap();

        let input = CreateSyncManifestInput {
            id: None,
            path: "/home/user/.gemini/GEMINI.md".to_string(),
            artifact_id: "rule-123".to_string(),
            artifact_type: crate::models::registry::ArtifactType::Rule,
            adapter: AdapterType::Gemini,
            scope: Scope::Global,
            content_hash: "abc123hash".to_string(),
        };

        let created = db.upsert_sync_manifest(input.clone()).await.unwrap();
        assert_eq!(created.path, "/home/user/.gemini/GEMINI.md");
        assert_eq!(created.adapter, AdapterType::Gemini);
        assert_eq!(created.scope, Scope::Global);

        let fetched = db
            .get_sync_manifest_by_path("/home/user/.gemini/GEMINI.md")
            .await
            .unwrap();
        assert!(fetched.is_some());
        let fetched = fetched.unwrap();
        assert_eq!(fetched.id, created.id);
        assert_eq!(fetched.content_hash, "abc123hash");

        let all = db
            .list_sync_manifest(SyncManifestFilter::default())
            .await
            .unwrap();
        assert_eq!(all.len(), 1);

        let filtered = db
            .list_sync_manifest(SyncManifestFilter {
                adapter: Some(AdapterType::Gemini),
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(filtered.len(), 1);

        let filtered_none = db
            .list_sync_manifest(SyncManifestFilter {
                adapter: Some(AdapterType::ClaudeCode),
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(filtered_none.len(), 0);

        db.delete_sync_manifest_by_path("/home/user/.gemini/GEMINI.md")
            .await
            .unwrap();
        assert!(db
            .get_sync_manifest_by_path("/home/user/.gemini/GEMINI.md")
            .await
            .unwrap()
            .is_none());
    }

    #[tokio::test]
    async fn test_sync_manifest_upsert_updates_existing() {
        let db = Database::new_in_memory().await.unwrap();

        let input1 = CreateSyncManifestInput {
            id: None,
            path: "/home/user/.gemini/GEMINI.md".to_string(),
            artifact_id: "rule-123".to_string(),
            artifact_type: crate::models::registry::ArtifactType::Rule,
            adapter: AdapterType::Gemini,
            scope: Scope::Global,
            content_hash: "hash1".to_string(),
        };

        db.upsert_sync_manifest(input1).await.unwrap();

        let input2 = CreateSyncManifestInput {
            id: None,
            path: "/home/user/.gemini/GEMINI.md".to_string(),
            artifact_id: "rule-456".to_string(),
            artifact_type: crate::models::registry::ArtifactType::Rule,
            adapter: AdapterType::Gemini,
            scope: Scope::Global,
            content_hash: "hash2".to_string(),
        };

        let created2 = db.upsert_sync_manifest(input2).await.unwrap();

        assert_eq!(created2.artifact_id, "rule-456");
        assert_eq!(created2.content_hash, "hash2");

        let fetched = db
            .get_sync_manifest_by_path("/home/user/.gemini/GEMINI.md")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(fetched.artifact_id, "rule-456");
        assert_eq!(fetched.content_hash, "hash2");

        let all = db
            .list_sync_manifest(SyncManifestFilter::default())
            .await
            .unwrap();
        assert_eq!(all.len(), 1);
    }

    #[tokio::test]
    async fn test_tool_sync_preferences_crud() {
        let db = Database::new_in_memory().await.unwrap();

        let input = UpsertToolSyncPreferencesInput {
            tool_id: AdapterType::Gemini,
            sync_rules: Some(true),
            sync_commands: Some(false),
            sync_skills: Some(true),
        };

        let created = db.upsert_tool_sync_preferences(input).await.unwrap();
        assert_eq!(created.tool_id, AdapterType::Gemini);
        assert!(created.sync_rules);
        assert!(!created.sync_commands);
        assert!(created.sync_skills);

        let fetched = db
            .get_tool_sync_preferences(&AdapterType::Gemini)
            .await
            .unwrap();
        assert!(fetched.is_some());
        let fetched = fetched.unwrap();
        assert_eq!(fetched.tool_id, AdapterType::Gemini);
        assert!(!fetched.sync_commands);

        let all = db.get_all_tool_sync_preferences().await.unwrap();
        assert_eq!(all.len(), 1);

        let not_found = db
            .get_tool_sync_preferences(&AdapterType::ClaudeCode)
            .await
            .unwrap();
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_tool_sync_preferences_defaults() {
        let db = Database::new_in_memory().await.unwrap();

        let input = UpsertToolSyncPreferencesInput {
            tool_id: AdapterType::Gemini,
            sync_rules: None,
            sync_commands: None,
            sync_skills: None,
        };

        let created = db.upsert_tool_sync_preferences(input).await.unwrap();
        assert!(created.sync_rules);
        assert!(created.sync_commands);
        assert!(created.sync_skills);
    }
}
