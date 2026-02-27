use std::path::PathBuf;
use tokio::sync::Mutex;

use chrono::{DateTime, TimeZone, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use tauri::Manager;

use crate::error::{AppError, Result};
use crate::file_storage::StorageLocation;
use crate::models::{
    AdapterType, Command, CommandArgument, CreateCommandInput, CreateRuleInput, CreateSkillInput,
    ExecutionLog, ReconcileOperation, ReconcileResultType, Rule, Scope, Skill, SyncHistoryEntry,
    UpdateCommandInput, UpdateRuleInput, UpdateSkillInput,
};

fn parse_timestamp_or_now(timestamp: i64) -> DateTime<Utc> {
    chrono::Utc
        .timestamp_opt(timestamp, 0)
        .single()
        .unwrap_or_else(chrono::Utc::now)
}

pub struct Database(Mutex<Connection>);

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
    pub is_redacted: bool,
    pub attempt_number: u8,
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

impl Database {
    async fn new_with_db_path(db_path: PathBuf) -> Result<Self> {
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

        Ok(Self(Mutex::new(conn)))
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
        let conn = tokio::task::spawn_blocking(move || -> Result<Connection> {
            let mut conn = Connection::open_in_memory()?;
            run_migrations(&mut conn)?;
            Ok(conn)
        })
        .await
        .map_err(|e| AppError::Database(rusqlite::Error::ToSqlConversionFailure(Box::new(e))))??;

        Ok(Self(Mutex::new(conn)))
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
                let id: String = row.get(0)?;
                let name: String = row.get(1)?;
                let description: String = row.get(2)?;
                let content: String = row.get(3)?;
                let scope_str: String = row.get(4)?;
                let target_paths_json: Option<String> = row.get(5)?;
                let enabled_adapters_json: String = row.get(6)?;
                let enabled: bool = row.get(7)?;
                let created_at: i64 = row.get(8)?;
                let updated_at: i64 = row.get(9)?;

                let scope = Scope::from_str(&scope_str).ok_or_else(|| {
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
                            4,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })?),
                    None => None,
                };

                let enabled_adapters: Vec<AdapterType> =
                    serde_json::from_str(&enabled_adapters_json).map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            5,
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
                let id: String = row.get(0)?;
                let name: String = row.get(1)?;
                let description: String = row.get(2)?;
                let content: String = row.get(3)?;
                let scope_str: String = row.get(4)?;
                let target_paths_json: Option<String> = row.get(5)?;
                let enabled_adapters_json: String = row.get(6)?;
                let enabled: bool = row.get(7)?;
                let created_at: i64 = row.get(8)?;
                let updated_at: i64 = row.get(9)?;

                let scope = Scope::from_str(&scope_str).ok_or_else(|| {
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
                            4,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })?),
                    None => None,
                };
                let enabled_adapters: Vec<AdapterType> =
                    serde_json::from_str(&enabled_adapters_json).map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            5,
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
            "SELECT id, name, description, script, arguments, expose_via_mcp, is_placeholder, generate_slash_commands, slash_command_adapters, target_paths, created_at, updated_at, timeout_ms, max_retries
             FROM commands
             ORDER BY updated_at DESC",
        )?;

        let commands = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let name: String = row.get(1)?;
                let description: String = row.get(2)?;
                let script: String = row.get(3)?;
                let arguments_json: String = row.get(4)?;
                let expose_via_mcp: bool = row.get(5)?;
                let is_placeholder: bool = row.get(6)?;
                let generate_slash_commands: bool = row.get(7)?;
                let slash_adapters_json: String = row.get(8)?;
                let target_paths_json: String = row.get(9)?;
                let created_at: i64 = row.get(10)?;
                let updated_at: i64 = row.get(11)?;
                let timeout_ms: Option<i64> = row.get(12)?;
                let max_retries: Option<i32> = row.get(13)?;

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
                    expose_via_mcp,
                    is_placeholder,
                    generate_slash_commands,
                    slash_command_adapters,
                    target_paths,
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
            "SELECT id, name, description, script, arguments, expose_via_mcp, is_placeholder, generate_slash_commands, slash_command_adapters, target_paths, created_at, updated_at, timeout_ms, max_retries
             FROM commands
             WHERE id = ?",
        )?;

        let command = stmt
            .query_row(params![id], |row| {
                let id: String = row.get(0)?;
                let name: String = row.get(1)?;
                let description: String = row.get(2)?;
                let script: String = row.get(3)?;
                let arguments_json: String = row.get(4)?;
                let expose_via_mcp: bool = row.get(5)?;
                let is_placeholder: bool = row.get(6)?;
                let generate_slash_commands: bool = row.get(7)?;
                let slash_adapters_json: String = row.get(8)?;
                let target_paths_json: String = row.get(9)?;
                let created_at: i64 = row.get(10)?;
                let updated_at: i64 = row.get(11)?;
                let timeout_ms: Option<i64> = row.get(12)?;
                let max_retries: Option<i32> = row.get(13)?;

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
                    expose_via_mcp,
                    is_placeholder,
                    generate_slash_commands,
                    slash_command_adapters,
                    target_paths,
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
            "INSERT INTO commands (id, name, description, script, arguments, expose_via_mcp, is_placeholder, generate_slash_commands, slash_command_adapters, target_paths, created_at, updated_at, timeout_ms, max_retries)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                id,
                input.name,
                input.description,
                input.script,
                arguments_json,
                input.expose_via_mcp,
                input.is_placeholder,
                input.generate_slash_commands,
                slash_adapters_json,
                target_paths_json,
                now,
                now,
                input.timeout_ms.map(|t| t as i64),
                input.max_retries.map(|r| r as i32)
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
        let expose_via_mcp = input.expose_via_mcp.unwrap_or(existing.expose_via_mcp);
        let is_placeholder = input.is_placeholder.unwrap_or(existing.is_placeholder);
        let generate_slash_commands = input
            .generate_slash_commands
            .unwrap_or(existing.generate_slash_commands);
        let slash_command_adapters = input
            .slash_command_adapters
            .unwrap_or(existing.slash_command_adapters);
        let target_paths = input.target_paths.unwrap_or(existing.target_paths);
        let timeout_ms = input.timeout_ms.or(existing.timeout_ms);
        let max_retries = input.max_retries.or(existing.max_retries);
        let now = chrono::Utc::now().timestamp();
        let arguments_json = serde_json::to_string(&arguments)?;
        let slash_adapters_json = serde_json::to_string(&slash_command_adapters)?;
        let target_paths_json = serde_json::to_string(&target_paths)?;

        conn.execute(
            "UPDATE commands SET name = ?, description = ?, script = ?, arguments = ?, expose_via_mcp = ?, is_placeholder = ?, generate_slash_commands = ?, slash_command_adapters = ?, target_paths = ?, updated_at = ?, timeout_ms = ?, max_retries = ?
             WHERE id = ?",
            params![
                name,
                description,
                script,
                arguments_json,
                expose_via_mcp,
                is_placeholder,
                generate_slash_commands,
                slash_adapters_json,
                target_paths_json,
                now,
                timeout_ms.map(|t| t as i64),
                max_retries.map(|r| r as i32),
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
            "SELECT id, name, description, instructions, input_schema, enabled, created_at, updated_at, directory_path, entry_point, scope, target_adapters, target_paths
             FROM skills
             ORDER BY updated_at DESC",
        )?;

        let skills = stmt
            .query_map([], |row| {
                Ok(Skill {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    instructions: row.get(3)?,
                    input_schema: {
                        let raw: String = row.get(4)?;
                        serde_json::from_str(&raw).map_err(|e| {
                            rusqlite::Error::FromSqlConversionFailure(
                                4,
                                rusqlite::types::Type::Text,
                                Box::new(e),
                            )
                        })?
                    },
                    enabled: row.get(5)?,
                    created_at: parse_timestamp_or_now(row.get(6)?),
                    updated_at: parse_timestamp_or_now(row.get(7)?),
                    directory_path: row.get(8)?,
                    entry_point: row.get(9)?,
                    scope: Scope::from_str(&row.get::<_, String>(10)?).ok_or_else(|| {
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
                        let raw: String = row.get(11)?;
                        serde_json::from_str(&raw).unwrap_or_else(|e| {
                            log::warn!("Failed to parse skill JSON: {}. Falling back to empty.", e);
                            Vec::new()
                        })
                    },
                    target_paths: {
                        let raw: String = row.get(12)?;
                        serde_json::from_str(&raw).unwrap_or_else(|e| {
                            log::warn!("Failed to parse skill JSON: {}. Falling back to empty.", e);
                            Vec::new()
                        })
                    },
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(skills)
    }

    pub async fn get_skill_by_id(&self, id: &str) -> Result<Skill> {
        let conn = self.0.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, instructions, input_schema, enabled, created_at, updated_at, directory_path, entry_point, scope, target_adapters, target_paths
             FROM skills WHERE id = ?",
        )?;

        let skill = stmt
            .query_row(params![id], |row| {
                Ok(Skill {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    instructions: row.get(3)?,
                    input_schema: {
                        let raw: String = row.get(4)?;
                        serde_json::from_str(&raw).map_err(|e| {
                            rusqlite::Error::FromSqlConversionFailure(
                                4,
                                rusqlite::types::Type::Text,
                                Box::new(e),
                            )
                        })?
                    },
                    enabled: row.get(5)?,
                    created_at: parse_timestamp_or_now(row.get(6)?),
                    updated_at: parse_timestamp_or_now(row.get(7)?),
                    directory_path: row.get(8)?,
                    entry_point: row.get(9)?,
                    scope: Scope::from_str(&row.get::<_, String>(10)?).ok_or_else(|| {
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
                        let raw: String = row.get(11)?;
                        serde_json::from_str(&raw).unwrap_or_else(|e| {
                            log::warn!("Failed to parse skill JSON: {}. Falling back to empty.", e);
                            Vec::new()
                        })
                    },
                    target_paths: {
                        let raw: String = row.get(12)?;
                        serde_json::from_str(&raw).unwrap_or_else(|e| {
                            log::warn!("Failed to parse skill JSON: {}. Falling back to empty.", e);
                            Vec::new()
                        })
                    },
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
            "INSERT INTO skills (id, name, description, instructions, input_schema, enabled, directory_path, entry_point, scope, target_adapters, target_paths, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
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
                &now
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
        let now = chrono::Utc::now().timestamp();
        let input_schema_json = serde_json::to_string(&input_schema)?;
        let target_adapters_json = serde_json::to_string(&target_adapters)?;
        let target_paths_json = serde_json::to_string(&target_paths)?;

        conn.execute(
            "UPDATE skills SET name = ?, description = ?, instructions = ?, input_schema = ?, enabled = ?, directory_path = ?, entry_point = ?, scope = ?, target_adapters = ?, target_paths = ?, updated_at = ? WHERE id = ?",
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

    pub async fn get_mcp_data(&self) -> Result<(Vec<Command>, Vec<Skill>)> {
        let commands = self.get_all_commands().await?;
        let skills = self.get_all_skills().await?;
        Ok((commands, skills))
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
                let timestamp: i64 = row.get(8)?;
                Ok(ExecutionLog {
                    id: row.get(0)?,
                    command_id: row.get(1)?,
                    command_name: row.get(2)?,
                    arguments: row.get(3)?,
                    stdout: row.get(4)?,
                    stderr: row.get(5)?,
                    exit_code: row.get(6)?,
                    duration_ms: row.get::<_, i64>(7)? as u64,
                    executed_at: parse_timestamp_or_now(timestamp),
                    triggered_by: row.get(9)?,
                    failure_class: row.get(10)?,
                    adapter_context: row.get(11)?,
                    is_redacted: row.get::<_, i32>(12)? != 0,
                    attempt_number: row.get::<_, i32>(13)? as u8,
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
                let timestamp: i64 = row.get(8)?;
                Ok(ExecutionLog {
                    id: row.get(0)?,
                    command_id: row.get(1)?,
                    command_name: row.get(2)?,
                    arguments: row.get(3)?,
                    stdout: row.get(4)?,
                    stderr: row.get(5)?,
                    exit_code: row.get(6)?,
                    duration_ms: row.get::<_, i64>(7)? as u64,
                    executed_at: parse_timestamp_or_now(timestamp),
                    triggered_by: row.get(9)?,
                    failure_class: row.get(10)?,
                    adapter_context: row.get(11)?,
                    is_redacted: row.get::<_, i32>(12)? != 0,
                    attempt_number: row.get::<_, i32>(13)? as u8,
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
                let id: String = row.get(0)?;
                let timestamp: i64 = row.get(1)?;
                let files_written: u32 = row.get(2)?;
                let status: String = row.get(3)?;
                let triggered_by: String = row.get(4)?;

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
                let key: String = row.get(0)?;
                let value: String = row.get(1)?;
                Ok((key, value))
            })?
            .collect::<std::result::Result<std::collections::HashMap<String, String>, _>>()?;

        Ok(settings)
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
                "INSERT OR REPLACE INTO commands (id, name, description, script, arguments, expose_via_mcp, is_placeholder, generate_slash_commands, slash_command_adapters, target_paths, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
            }
            crate::models::ImportMode::Skip => {
                "INSERT OR IGNORE INTO commands (id, name, description, script, arguments, expose_via_mcp, is_placeholder, generate_slash_commands, slash_command_adapters, target_paths, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
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
                command.expose_via_mcp,
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
                let op_str: String = row.get(2)?;
                let operation =
                    ReconcileOperation::from_str(&op_str).unwrap_or(ReconcileOperation::Check);

                let adapter_str: Option<String> = row.get(4)?;
                let adapter = adapter_str.and_then(|s| AdapterType::from_str(&s));

                let scope_str: Option<String> = row.get(5)?;
                let scope = scope_str.and_then(|s| Scope::from_str(&s));

                let res_str: String = row.get(7)?;
                let result =
                    ReconcileResultType::from_str(&res_str).unwrap_or(ReconcileResultType::Failed);

                Ok(ReconciliationLogEntry {
                    id: row.get(0)?,
                    timestamp: parse_timestamp_or_now(row.get(1)?),
                    operation,
                    artifact_type: row.get(3)?,
                    adapter,
                    scope,
                    path: row.get(6)?,
                    result,
                    error_message: row.get(8)?,
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
            let path: String = row.get(0)?;
            let operation: String = row.get(1)?;
            let timestamp: DateTime<Utc> = parse_timestamp_or_now(row.get(2)?);
            Ok((path, operation, timestamp))
        })?;

        let mut ops = std::collections::HashMap::new();
        for (path, operation, timestamp) in rows.flatten() {
            ops.insert(path, (operation, timestamp));
        }

        Ok(ops)
    }

    pub async fn clear_reconciliation_logs(&self) -> Result<()> {
        let conn = self.0.lock().await;
        conn.execute("DELETE FROM reconciliation_logs", [])?;
        Ok(())
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

    transaction.execute("PRAGMA user_version = 15", [])?;
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
        CreateCommandInput, CreateSkillInput, SkillParameter, SkillParameterType,
        UpdateCommandInput, UpdateSkillInput,
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
                expose_via_mcp: true,
                is_placeholder: false,
                generate_slash_commands: false,
                slash_command_adapters: vec![],
                target_paths: vec!["C:/repo-a".to_string()],
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
}
