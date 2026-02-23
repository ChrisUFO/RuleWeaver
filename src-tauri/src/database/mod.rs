use std::path::PathBuf;
use std::sync::Mutex;

use chrono::{DateTime, TimeZone, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use tauri::Manager;

use crate::error::{AppError, Result};
use crate::file_storage::StorageLocation;
use crate::models::{
    AdapterType, Command, CommandArgument, CreateCommandInput, CreateRuleInput, CreateSkillInput,
    ExecutionLog, Rule, Scope, Skill, SyncHistoryEntry, UpdateCommandInput, UpdateRuleInput,
    UpdateSkillInput,
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
}

impl Database {
    fn new_with_db_path(db_path: PathBuf) -> Result<Self> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut conn = Connection::open(&db_path)?;
        run_migrations(&mut conn)?;
        Ok(Self(Mutex::new(conn)))
    }

    pub fn new(app_handle: &tauri::AppHandle) -> Result<Self> {
        let app_data_dir = app_handle
            .path()
            .app_data_dir()
            .map_err(|e| AppError::Path(e.to_string()))?;
        let db_path = app_data_dir.join("ruleweaver.db");
        Self::new_with_db_path(db_path)
    }

    pub fn new_for_cli() -> Result<Self> {
        let app_data_dir = default_app_data_dir()?;
        let db_path = app_data_dir.join("ruleweaver.db");
        Self::new_with_db_path(db_path)
    }

    #[cfg(test)]
    pub fn new_in_memory() -> Result<Self> {
        let mut conn = Connection::open_in_memory()?;
        run_migrations(&mut conn)?;
        Ok(Self(Mutex::new(conn)))
    }

    /// Re-establishes the database connection and runs migrations.
    /// Useful for recovering from disk disconnections or handling external database modifications.
    #[allow(dead_code)]
    pub fn reconnect(&self) -> Result<()> {
        let db_path = {
            let conn = self.0.lock().map_err(|_| AppError::DatabasePoisoned)?;
            let path: String = conn.query_row("PRAGMA database_list", [], |row| row.get(2))?;
            PathBuf::from(path)
        };

        let mut conn = Connection::open(&db_path)?;
        run_migrations(&mut conn)?;

        let mut guard = self.0.lock().map_err(|_| AppError::DatabasePoisoned)?;
        *guard = conn;
        Ok(())
    }

    pub fn get_all_rules(&self) -> Result<Vec<Rule>> {
        let conn = self.0.lock().map_err(|_| AppError::DatabasePoisoned)?;
        let mut stmt = conn.prepare(
            "SELECT id, name, content, scope, target_paths, enabled_adapters, enabled, created_at, updated_at 
             FROM rules 
             ORDER BY updated_at DESC"
        )?;

        let rules = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let name: String = row.get(1)?;
                let content: String = row.get(2)?;
                let scope_str: String = row.get(3)?;
                let target_paths_json: Option<String> = row.get(4)?;
                let enabled_adapters_json: String = row.get(5)?;
                let enabled: bool = row.get(6)?;
                let created_at: i64 = row.get(7)?;
                let updated_at: i64 = row.get(8)?;

                let scope = Scope::from_str(&scope_str).ok_or_else(|| {
                    rusqlite::Error::FromSqlConversionFailure(
                        3,
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

    pub fn get_rule_by_id(&self, id: &str) -> Result<Rule> {
        let conn = self.0.lock().map_err(|_| AppError::DatabasePoisoned)?;
        let mut stmt = conn.prepare(
            "SELECT id, name, content, scope, target_paths, enabled_adapters, enabled, created_at, updated_at 
             FROM rules 
             WHERE id = ?"
        )?;

        let rule = stmt
            .query_row(params![id], |row| {
                let id: String = row.get(0)?;
                let name: String = row.get(1)?;
                let content: String = row.get(2)?;
                let scope_str: String = row.get(3)?;
                let target_paths_json: Option<String> = row.get(4)?;
                let enabled_adapters_json: String = row.get(5)?;
                let enabled: bool = row.get(6)?;
                let created_at: i64 = row.get(7)?;
                let updated_at: i64 = row.get(8)?;

                let scope = Scope::from_str(&scope_str).ok_or_else(|| {
                    rusqlite::Error::FromSqlConversionFailure(
                        3,
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

    pub fn create_rule(&self, input: CreateRuleInput) -> Result<Rule> {
        let conn = self.0.lock().map_err(|_| AppError::DatabasePoisoned)?;
        let now = chrono::Utc::now().timestamp();
        let id = uuid::Uuid::new_v4().to_string();

        let target_paths_json = input
            .target_paths
            .as_ref()
            .map(|p| serde_json::to_string(p).unwrap_or_default());

        let enabled_adapters_json = serde_json::to_string(&input.enabled_adapters)?;

        conn.execute(
            "INSERT INTO rules (id, name, content, scope, target_paths, enabled_adapters, enabled, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                id,
                input.name,
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
        self.get_rule_by_id(&id)
    }

    pub fn update_rule(&self, id: &str, input: UpdateRuleInput) -> Result<Rule> {
        let existing = self.get_rule_by_id(id)?;
        let conn = self.0.lock().map_err(|_| AppError::DatabasePoisoned)?;

        let name = input.name.unwrap_or(existing.name);
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
            "UPDATE rules SET name = ?, content = ?, scope = ?, target_paths = ?, enabled_adapters = ?, enabled = ?, updated_at = ?
             WHERE id = ?",
            params![
                name,
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
        self.get_rule_by_id(id)
    }

    pub fn delete_rule(&self, id: &str) -> Result<()> {
        let conn = self.0.lock().map_err(|_| AppError::DatabasePoisoned)?;
        conn.execute("DELETE FROM rules WHERE id = ?", params![id])?;
        Ok(())
    }

    pub fn toggle_rule(&self, id: &str, enabled: bool) -> Result<Rule> {
        let conn = self.0.lock().map_err(|_| AppError::DatabasePoisoned)?;
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "UPDATE rules SET enabled = ?, updated_at = ? WHERE id = ?",
            params![enabled, now, id],
        )?;

        drop(conn);
        self.get_rule_by_id(id)
    }

    pub fn get_all_commands(&self) -> Result<Vec<Command>> {
        let conn = self.0.lock().map_err(|_| AppError::DatabasePoisoned)?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, script, arguments, expose_via_mcp, created_at, updated_at
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
                let created_at: i64 = row.get(6)?;
                let updated_at: i64 = row.get(7)?;

                let arguments: Vec<CommandArgument> = serde_json::from_str(&arguments_json)
                    .map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            4,
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
                    created_at: parse_timestamp_or_now(created_at),
                    updated_at: parse_timestamp_or_now(updated_at),
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(commands)
    }

    pub fn get_command_by_id(&self, id: &str) -> Result<Command> {
        let conn = self.0.lock().map_err(|_| AppError::DatabasePoisoned)?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, script, arguments, expose_via_mcp, created_at, updated_at
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
                let created_at: i64 = row.get(6)?;
                let updated_at: i64 = row.get(7)?;

                let arguments: Vec<CommandArgument> = serde_json::from_str(&arguments_json)
                    .map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            4,
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

    pub fn create_command(&self, input: CreateCommandInput) -> Result<Command> {
        let conn = self.0.lock().map_err(|_| AppError::DatabasePoisoned)?;
        let now = chrono::Utc::now().timestamp();
        let id = uuid::Uuid::new_v4().to_string();
        let arguments_json = serde_json::to_string(&input.arguments)?;
        let slash_adapters_json = serde_json::to_string(&input.slash_command_adapters)?;

        conn.execute(
            "INSERT INTO commands (id, name, description, script, arguments, expose_via_mcp, generate_slash_commands, slash_command_adapters, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                id,
                input.name,
                input.description,
                input.script,
                arguments_json,
                input.expose_via_mcp,
                input.generate_slash_commands,
                slash_adapters_json,
                now,
                now
            ],
        )?;

        drop(conn);
        self.get_command_by_id(&id)
    }

    pub fn update_command(&self, id: &str, input: UpdateCommandInput) -> Result<Command> {
        let existing = self.get_command_by_id(id)?;
        let conn = self.0.lock().map_err(|_| AppError::DatabasePoisoned)?;

        let name = input.name.unwrap_or(existing.name);
        let description = input.description.unwrap_or(existing.description);
        let script = input.script.unwrap_or(existing.script);
        let arguments = input.arguments.unwrap_or(existing.arguments);
        let expose_via_mcp = input.expose_via_mcp.unwrap_or(existing.expose_via_mcp);
        let generate_slash_commands = input
            .generate_slash_commands
            .unwrap_or(existing.generate_slash_commands);
        let slash_command_adapters = input
            .slash_command_adapters
            .unwrap_or(existing.slash_command_adapters);
        let now = chrono::Utc::now().timestamp();
        let arguments_json = serde_json::to_string(&arguments)?;
        let slash_adapters_json = serde_json::to_string(&slash_command_adapters)?;

        conn.execute(
            "UPDATE commands SET name = ?, description = ?, script = ?, arguments = ?, expose_via_mcp = ?, generate_slash_commands = ?, slash_command_adapters = ?, updated_at = ?
             WHERE id = ?",
            params![
                name,
                description,
                script,
                arguments_json,
                expose_via_mcp,
                generate_slash_commands,
                slash_adapters_json,
                now,
                id
            ],
        )?;

        drop(conn);
        self.get_command_by_id(id)
    }

    pub fn delete_command(&self, id: &str) -> Result<()> {
        let conn = self.0.lock().map_err(|_| AppError::DatabasePoisoned)?;
        conn.execute("DELETE FROM commands WHERE id = ?", params![id])?;
        Ok(())
    }

    pub fn get_all_skills(&self) -> Result<Vec<Skill>> {
        let conn = self.0.lock().map_err(|_| AppError::DatabasePoisoned)?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, instructions, input_schema, enabled, created_at, updated_at, directory_path, entry_point, scope
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
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(skills)
    }

    pub fn get_skill_by_id(&self, id: &str) -> Result<Skill> {
        let conn = self.0.lock().map_err(|_| AppError::DatabasePoisoned)?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, instructions, input_schema, enabled, created_at, updated_at, directory_path, entry_point, scope
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

    pub fn create_skill(&self, input: CreateSkillInput) -> Result<Skill> {
        let conn = self.0.lock().map_err(|_| AppError::DatabasePoisoned)?;
        let now = chrono::Utc::now().timestamp();
        let id = input.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let input_schema_json = serde_json::to_string(&input.input_schema)?;

        conn.execute(
            "INSERT INTO skills (id, name, description, instructions, input_schema, enabled, directory_path, entry_point, scope, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
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
                &now,
                &now
            ],
        )?;

        drop(conn);
        self.get_skill_by_id(&id)
    }

    pub fn update_skill(&self, id: &str, input: UpdateSkillInput) -> Result<Skill> {
        let existing = self.get_skill_by_id(id)?;
        let conn = self.0.lock().map_err(|_| AppError::DatabasePoisoned)?;

        let name = input.name.unwrap_or(existing.name);
        let description = input.description.unwrap_or(existing.description);
        let instructions = input.instructions.unwrap_or(existing.instructions);
        let input_schema = input.input_schema.unwrap_or(existing.input_schema);
        let enabled = input.enabled.unwrap_or(existing.enabled);
        let directory_path = input.directory_path.unwrap_or(existing.directory_path);
        let entry_point = input.entry_point.unwrap_or(existing.entry_point);
        let scope = input.scope.unwrap_or(existing.scope);
        let now = chrono::Utc::now().timestamp();
        let input_schema_json = serde_json::to_string(&input_schema)?;

        conn.execute(
            "UPDATE skills SET name = ?, description = ?, instructions = ?, input_schema = ?, enabled = ?, directory_path = ?, entry_point = ?, scope = ?, updated_at = ? WHERE id = ?",
            params![
                &name,
                &description,
                &instructions,
                &input_schema_json,
                &enabled,
                &directory_path,
                &entry_point,
                &scope.as_str(),
                &now,
                &id
            ],
        )?;

        drop(conn);
        self.get_skill_by_id(id)
    }

    pub fn delete_skill(&self, id: &str) -> Result<()> {
        let conn = self.0.lock().map_err(|_| AppError::DatabasePoisoned)?;
        conn.execute("DELETE FROM skills WHERE id = ?", params![id])?;
        Ok(())
    }

    pub fn get_mcp_data(&self) -> Result<(Vec<Command>, Vec<Skill>)> {
        let commands = self.get_all_commands()?;
        let skills = self.get_all_skills()?;
        Ok((commands, skills))
    }

    pub fn add_execution_log(&self, input: &ExecutionLogInput<'_>) -> Result<()> {
        let conn = self.0.lock().map_err(|_| AppError::DatabasePoisoned)?;
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT INTO execution_logs (id, command_id, command_name, arguments, stdout, stderr, exit_code, duration_ms, executed_at, triggered_by)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
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
                input.triggered_by
            ],
        )?;

        Ok(())
    }

    pub fn get_execution_history(&self, limit: u32) -> Result<Vec<ExecutionLog>> {
        let conn = self.0.lock().map_err(|_| AppError::DatabasePoisoned)?;
        let mut stmt = conn.prepare(
            "SELECT id, command_id, command_name, arguments, stdout, stderr, exit_code, duration_ms, executed_at, triggered_by
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
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(rows)
    }

    pub fn get_file_hash(&self, file_path: &str) -> Result<Option<String>> {
        let conn = self.0.lock().map_err(|_| AppError::DatabasePoisoned)?;
        let result: Option<String> = conn
            .query_row(
                "SELECT content_hash FROM sync_history WHERE file_path = ?",
                params![file_path],
                |row| row.get(0),
            )
            .optional()?;

        Ok(result)
    }

    pub fn set_file_hash(&self, file_path: &str, hash: &str) -> Result<()> {
        let conn = self.0.lock().map_err(|_| AppError::DatabasePoisoned)?;
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT OR REPLACE INTO sync_history (file_path, content_hash, last_sync_at)
             VALUES (?, ?, ?)",
            params![file_path, hash, now],
        )?;

        Ok(())
    }

    pub fn add_sync_log(&self, files_written: u32, status: &str, triggered_by: &str) -> Result<()> {
        let conn = self.0.lock().map_err(|_| AppError::DatabasePoisoned)?;
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT INTO sync_logs (id, timestamp, files_written, status, triggered_by)
             VALUES (?, ?, ?, ?, ?)",
            params![id, now, files_written, status, triggered_by],
        )?;

        Ok(())
    }

    pub fn get_sync_history(&self, limit: u32) -> Result<Vec<SyncHistoryEntry>> {
        let conn = self.0.lock().map_err(|_| AppError::DatabasePoisoned)?;
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

    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let conn = self.0.lock().map_err(|_| AppError::DatabasePoisoned)?;
        let result: Option<String> = conn
            .query_row(
                "SELECT value FROM settings WHERE key = ?",
                params![key],
                |row| row.get(0),
            )
            .optional()?;

        Ok(result)
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.0.lock().map_err(|_| AppError::DatabasePoisoned)?;
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_all_settings(&self) -> Result<std::collections::HashMap<String, String>> {
        let conn = self.0.lock().map_err(|_| AppError::DatabasePoisoned)?;
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

    pub fn get_database_path(&self) -> Result<String> {
        let conn = self.0.lock().map_err(|_| AppError::DatabasePoisoned)?;
        let path: String = conn.query_row("PRAGMA database_list", [], |row| row.get(2))?;
        Ok(path)
    }

    pub fn update_rule_file_index(&self, rule_id: &str, location: &StorageLocation) -> Result<()> {
        let conn = self.0.lock().map_err(|_| AppError::DatabasePoisoned)?;
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

    pub fn get_rule_file_path(&self, rule_id: &str) -> Result<Option<String>> {
        let conn = self.0.lock().map_err(|_| AppError::DatabasePoisoned)?;
        let result: Option<String> = conn
            .query_row(
                "SELECT file_path FROM rule_file_index WHERE rule_id = ?",
                params![rule_id],
                |row| row.get(0),
            )
            .optional()?;

        Ok(result)
    }

    pub fn remove_rule_file_index(&self, rule_id: &str) -> Result<()> {
        let conn = self.0.lock().map_err(|_| AppError::DatabasePoisoned)?;
        conn.execute(
            "DELETE FROM rule_file_index WHERE rule_id = ?",
            params![rule_id],
        )?;
        Ok(())
    }

    pub fn import_rule(&self, rule: Rule, mode: crate::models::ImportMode) -> Result<()> {
        let conn = self.0.lock().map_err(|_| AppError::DatabasePoisoned)?;
        let now = chrono::Utc::now().timestamp();

        let target_paths_json = rule
            .target_paths
            .as_ref()
            .map(|p| serde_json::to_string(p).unwrap_or_default());

        let enabled_adapters_json = serde_json::to_string(&rule.enabled_adapters)?;

        let sql = match mode {
            crate::models::ImportMode::Overwrite => {
                log::info!("Import: Overwriting rule {}", rule.id);
                "INSERT OR REPLACE INTO rules (id, name, content, scope, target_paths, enabled_adapters, enabled, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
            }
            crate::models::ImportMode::Skip => {
                "INSERT OR IGNORE INTO rules (id, name, content, scope, target_paths, enabled_adapters, enabled, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
            }
        };

        conn.execute(
            sql,
            params![
                rule.id,
                rule.name,
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

    pub fn import_command(&self, command: Command, mode: crate::models::ImportMode) -> Result<()> {
        let conn = self.0.lock().map_err(|_| AppError::DatabasePoisoned)?;
        let now = chrono::Utc::now().timestamp();
        let arguments_json = serde_json::to_string(&command.arguments)?;

        let sql = match mode {
            crate::models::ImportMode::Overwrite => {
                log::info!("Import: Overwriting command {}", command.id);
                "INSERT OR REPLACE INTO commands (id, name, description, script, arguments, expose_via_mcp, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
            }
            crate::models::ImportMode::Skip => {
                "INSERT OR IGNORE INTO commands (id, name, description, script, arguments, expose_via_mcp, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
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
                command.created_at.timestamp(),
                now
            ],
        )?;
        Ok(())
    }

    pub fn import_skill(&self, skill: Skill, mode: crate::models::ImportMode) -> Result<()> {
        let conn = self.0.lock().map_err(|_| AppError::DatabasePoisoned)?;
        let now = chrono::Utc::now().timestamp();
        let input_schema_json = serde_json::to_string(&skill.input_schema)?;

        let sql = match mode {
            crate::models::ImportMode::Overwrite => {
                log::info!("Import: Overwriting skill {}", skill.id);
                "INSERT OR REPLACE INTO skills (id, name, description, instructions, input_schema, enabled, directory_path, entry_point, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
            }
            crate::models::ImportMode::Skip => {
                "INSERT OR IGNORE INTO skills (id, name, description, instructions, input_schema, enabled, directory_path, entry_point, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
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
                skill.created_at.timestamp(),
                now
            ],
        )?;
        Ok(())
    }

    pub fn import_configuration(
        &self,
        config: crate::models::ExportConfiguration,
        mode: crate::models::ImportMode,
    ) -> Result<()> {
        for rule in config.rules {
            self.import_rule(rule, mode)?;
        }

        for command in config.commands {
            self.import_command(command, mode)?;
        }

        for skill in config.skills {
            self.import_skill(skill, mode)?;
        }
        Ok(())
    }

    pub fn get_storage_mode(&self) -> Result<String> {
        let mode = self.get_setting("storage_mode")?;
        Ok(mode.unwrap_or_else(|| "sqlite".to_string()))
    }

    pub fn set_storage_mode(&self, mode: &str) -> Result<()> {
        self.set_setting("storage_mode", mode)
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

    transaction.execute("PRAGMA user_version = 9", [])?;
    transaction.commit()?;

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
    use crate::models::{CreateSkillInput, SkillParameter, SkillParameterType, UpdateSkillInput};

    #[test]
    fn test_skill_crud() {
        let db = Database::new_in_memory().unwrap();

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
        };

        let created = db.create_skill(input.clone()).unwrap();
        assert_eq!(created.name, "Test Skill");
        assert_eq!(created.input_schema.len(), 1);
        assert_eq!(created.directory_path, "/test/path");

        // 2. Read
        let fetched = db.get_skill_by_id(&created.id).unwrap();
        assert_eq!(created.id, fetched.id);
        assert_eq!(fetched.entry_point, "main.sh");

        let all = db.get_all_skills().unwrap();
        assert_eq!(all.len(), 1);

        // 3. Update
        let update_input = UpdateSkillInput {
            name: Some("Updated Skill".to_string()),
            ..Default::default()
        };
        let updated = db.update_skill(&created.id, update_input).unwrap();
        assert_eq!(updated.name, "Updated Skill");
        // Ensure other fields remain unchanged
        assert_eq!(updated.directory_path, "/test/path");

        // 4. Delete
        db.delete_skill(&created.id).unwrap();
        assert!(db.get_skill_by_id(&created.id).is_err());
        assert_eq!(db.get_all_skills().unwrap().len(), 0);
    }
}
