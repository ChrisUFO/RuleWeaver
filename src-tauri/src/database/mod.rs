use std::path::PathBuf;
use std::sync::Mutex;

use chrono::TimeZone;
use rusqlite::{params, Connection, OptionalExtension};
use tauri::Manager;

use crate::error::{AppError, Result};
use crate::models::{AdapterType, CreateRuleInput, Rule, Scope, SyncHistoryEntry, UpdateRuleInput};

pub struct Database(Mutex<Connection>);

impl Database {
    pub fn new(app_handle: &tauri::AppHandle) -> Result<Self> {
        let app_data_dir = app_handle
            .path()
            .app_data_dir()
            .map_err(|e| AppError::Path(e.to_string()))?;

        std::fs::create_dir_all(&app_data_dir)?;

        let db_path = app_data_dir.join("ruleweaver.db");
        let conn = Connection::open(&db_path)?;

        run_migrations(&conn)?;

        Ok(Self(Mutex::new(conn)))
    }

    pub fn get_all_rules(&self) -> Result<Vec<Rule>> {
        let conn = self.0.lock().unwrap();
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

                let scope = Scope::from_str(&scope_str).unwrap_or(Scope::Global);

                let target_paths: Option<Vec<String>> =
                    target_paths_json.and_then(|j| serde_json::from_str(&j).ok());

                let enabled_adapters: Vec<AdapterType> =
                    serde_json::from_str(&enabled_adapters_json)
                        .unwrap_or_else(|_| AdapterType::all());

                Ok(Rule {
                    id,
                    name,
                    content,
                    scope,
                    target_paths,
                    enabled_adapters,
                    enabled,
                    created_at: chrono::Utc
                        .timestamp_opt(created_at, 0)
                        .single()
                        .unwrap_or_else(|| chrono::Utc::now()),
                    updated_at: chrono::Utc
                        .timestamp_opt(updated_at, 0)
                        .single()
                        .unwrap_or_else(|| chrono::Utc::now()),
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(rules)
    }

    pub fn get_rule_by_id(&self, id: &str) -> Result<Rule> {
        let conn = self.0.lock().unwrap();
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

                let scope = Scope::from_str(&scope_str).unwrap_or(Scope::Global);
                let target_paths: Option<Vec<String>> =
                    target_paths_json.and_then(|j| serde_json::from_str(&j).ok());
                let enabled_adapters: Vec<AdapterType> =
                    serde_json::from_str(&enabled_adapters_json)
                        .unwrap_or_else(|_| AdapterType::all());

                Ok(Rule {
                    id,
                    name,
                    content,
                    scope,
                    target_paths,
                    enabled_adapters,
                    enabled,
                    created_at: chrono::Utc
                        .timestamp_opt(created_at, 0)
                        .single()
                        .unwrap_or_else(|| chrono::Utc::now()),
                    updated_at: chrono::Utc
                        .timestamp_opt(updated_at, 0)
                        .single()
                        .unwrap_or_else(|| chrono::Utc::now()),
                })
            })
            .map_err(|_| AppError::RuleNotFound { id: id.to_string() })?;

        Ok(rule)
    }

    pub fn create_rule(&self, input: CreateRuleInput) -> Result<Rule> {
        let conn = self.0.lock().unwrap();
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
                true,
                now,
                now
            ],
        )?;

        self.get_rule_by_id(&id)
    }

    pub fn update_rule(&self, id: &str, input: UpdateRuleInput) -> Result<Rule> {
        let existing = self.get_rule_by_id(id)?;
        let conn = self.0.lock().unwrap();

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
        let conn = self.0.lock().unwrap();
        conn.execute("DELETE FROM rules WHERE id = ?", params![id])?;
        Ok(())
    }

    pub fn toggle_rule(&self, id: &str, enabled: bool) -> Result<Rule> {
        let conn = self.0.lock().unwrap();
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "UPDATE rules SET enabled = ?, updated_at = ? WHERE id = ?",
            params![enabled, now, id],
        )?;

        drop(conn);
        self.get_rule_by_id(id)
    }

    pub fn get_file_hash(&self, file_path: &str) -> Result<Option<String>> {
        let conn = self.0.lock().unwrap();
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
        let conn = self.0.lock().unwrap();
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT OR REPLACE INTO sync_history (file_path, content_hash, last_sync_at)
             VALUES (?, ?, ?)",
            params![file_path, hash, now],
        )?;

        Ok(())
    }

    pub fn add_sync_log(&self, files_written: u32, status: &str, triggered_by: &str) -> Result<()> {
        let conn = self.0.lock().unwrap();
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
        let conn = self.0.lock().unwrap();
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
                    timestamp: chrono::Utc
                        .timestamp_opt(timestamp, 0)
                        .single()
                        .unwrap_or_else(|| chrono::Utc::now()),
                    files_written,
                    status,
                    triggered_by,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(entries)
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let conn = self.0.lock().unwrap();
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
        let conn = self.0.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_all_settings(&self) -> Result<std::collections::HashMap<String, String>> {
        let conn = self.0.lock().unwrap();
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
}

fn run_migrations(conn: &Connection) -> Result<()> {
    let current_version: i32 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .unwrap_or(0);

    if current_version < 1 {
        conn.execute(
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

        conn.execute(
            "CREATE TABLE IF NOT EXISTS sync_history (
                file_path TEXT PRIMARY KEY NOT NULL,
                content_hash TEXT NOT NULL,
                last_sync_at INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS sync_logs (
                id TEXT PRIMARY KEY NOT NULL,
                timestamp INTEGER NOT NULL,
                files_written INTEGER NOT NULL,
                status TEXT NOT NULL,
                triggered_by TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY NOT NULL,
                value TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_rules_scope ON rules(scope)",
            [],
        )?;

        conn.execute("PRAGMA user_version = 1", [])?;
    }

    Ok(())
}

pub fn get_app_data_path(app_handle: &tauri::AppHandle) -> Result<PathBuf> {
    app_handle
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Path(e.to_string()))
}
