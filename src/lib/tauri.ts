import { invoke } from "@tauri-apps/api/core";
import type {
  Rule,
  CreateRuleInput,
  UpdateRuleInput,
  SyncResult,
  SyncHistoryEntry,
  Conflict,
} from "@/types/rule";
import type {
  CommandModel,
  CreateCommandInput,
  UpdateCommandInput,
  TestCommandResult,
  McpStatus,
  McpConnectionInstructions,
  ExecutionLog,
} from "@/types/command";
import type { CreateSkillInput, Skill, UpdateSkillInput, TemplateSkill } from "@/types/skill";

export const api = {
  rules: {
    getAll: () => invoke<Rule[]>("get_all_rules"),
    getById: (id: string) => invoke<Rule>("get_rule_by_id", { id }),
    create: (input: CreateRuleInput) => invoke<Rule>("create_rule", { input }),
    update: (id: string, input: UpdateRuleInput) => invoke<Rule>("update_rule", { id, input }),
    delete: (id: string) => invoke<void>("delete_rule", { id }),
    toggle: (id: string, enabled: boolean) => invoke<Rule>("toggle_rule", { id, enabled }),
  },

  sync: {
    syncRules: () => invoke<SyncResult>("sync_rules"),
    previewSync: () => invoke<SyncResult>("preview_sync"),
    getHistory: (limit?: number) =>
      invoke<SyncHistoryEntry[]>("get_sync_history", { limit: limit ?? 50 }),
    readFileContent: (filePath: string) => invoke<string>("read_file_content", { path: filePath }),
    resolveConflict: (conflict: Conflict, resolution: "overwrite" | "keep-remote") =>
      invoke<void>("resolve_conflict", {
        filePath: conflict.filePath,
        resolution,
      }),
  },

  settings: {
    get: (key: string) => invoke<string | null>("get_setting", { key }),
    set: (key: string, value: string) => invoke<void>("set_setting", { key, value }),
    getAll: () => invoke<Record<string, string>>("get_all_settings"),
  },

  storage: {
    getMode: () => invoke<string>("get_storage_mode"),
    getInfo: () => invoke<Record<string, string>>("get_storage_info"),
    migrateToFileStorage: () =>
      invoke<{
        success: boolean;
        rules_migrated: number;
        rules_skipped: number;
        errors: Array<{ rule_id: string; rule_name: string; error: string }>;
        backup_path?: string;
        storage_dir: string;
      }>("migrate_to_file_storage"),
    rollbackMigration: (backupPath: string) =>
      invoke<void>("rollback_file_migration", { backupPath }),
    verifyMigration: () =>
      invoke<{
        is_valid: boolean;
        db_rule_count: number;
        file_rule_count: number;
        missing_rules: string[];
        extra_rules: string[];
        mismatched_rules: string[];
        load_errors: number;
      }>("verify_file_migration"),
    getMigrationProgress: () =>
      invoke<{
        total: number;
        migrated: number;
        current_rule?: string;
        status: "NotStarted" | "InProgress" | "Completed" | "Failed" | "RolledBack";
      }>("get_file_migration_progress"),
  },

  commands: {
    getAll: () => invoke<CommandModel[]>("get_all_commands"),
    getById: (id: string) => invoke<CommandModel>("get_command_by_id", { id }),
    create: (input: CreateCommandInput) => invoke<CommandModel>("create_command", { input }),
    update: (id: string, input: UpdateCommandInput) =>
      invoke<CommandModel>("update_command", { id, input }),
    delete: (id: string) => invoke<void>("delete_command", { id }),
    test: (id: string, args: Record<string, string>) =>
      invoke<TestCommandResult>("test_command", { id, args }),
    sync: () => invoke<SyncResult>("sync_commands"),
  },

  skills: {
    getAll: () => invoke<Skill[]>("get_all_skills"),
    getById: (id: string) => invoke<Skill>("get_skill_by_id", { id }),
    create: (input: CreateSkillInput) => invoke<Skill>("create_skill", { input }),
    update: (id: string, input: UpdateSkillInput) => invoke<Skill>("update_skill", { id, input }),
    delete: (id: string) => invoke<void>("delete_skill", { id }),
    getTemplates: () => invoke<TemplateSkill[]>("get_skill_templates"),
    installTemplate: (templateId: string) =>
      invoke<Skill>("install_skill_template", { templateId }),
  },

  mcp: {
    getStatus: () => invoke<McpStatus>("get_mcp_status"),
    start: () => invoke<void>("start_mcp_server"),
    stop: () => invoke<void>("stop_mcp_server"),
    restart: () => invoke<void>("restart_mcp_server"),
    getInstructions: () => invoke<McpConnectionInstructions>("get_mcp_connection_instructions"),
    getLogs: (limit?: number) => invoke<string[]>("get_mcp_logs", { limit: limit ?? 50 }),
  },

  execution: {
    getHistory: (limit?: number) =>
      invoke<ExecutionLog[]>("get_execution_history", { limit: limit ?? 100 }),
  },

  app: {
    getAppDataPath: () => invoke<string>("get_app_data_path_cmd"),
    openInExplorer: (path: string) => invoke<void>("open_in_explorer", { path }),
    getVersion: () => invoke<string>("get_app_version"),
  },
};
