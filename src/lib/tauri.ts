import { invoke } from "@tauri-apps/api/core";
import type {
  Rule,
  CreateRuleInput,
  UpdateRuleInput,
  SyncResult,
  SyncHistoryEntry,
  Conflict,
  ImportExecutionOptions,
  ImportExecutionResult,
  ImportHistoryEntry,
  ImportScanResult,
  TemplateRule,
  ToolEntry,
} from "@/types/rule";
import type {
  CommandModel,
  CreateCommandInput,
  UpdateCommandInput,
  TestCommandResult,
  McpStatus,
  McpConnectionInstructions,
  ExecutionLog,
  TemplateCommand,
} from "@/types/command";
import type { CreateSkillInput, Skill, UpdateSkillInput, TemplateSkill } from "@/types/skill";

export const api = {
  rules: {
    getAll: () => invoke<Rule[]>("get_all_rules"),
    getById: (id: string) => invoke<Rule>("get_rule_by_id", { id }),
    create: (input: CreateRuleInput) => invoke<Rule>("create_rule", { input }),
    update: (id: string, input: UpdateRuleInput) => invoke<Rule>("update_rule", { id, input }),
    delete: (id: string) => invoke<void>("delete_rule", { id }),
    bulkDelete: (ids: string[]) => invoke<void>("bulk_delete_rules", { ids }),
    toggle: (id: string, enabled: boolean) => invoke<Rule>("toggle_rule", { id, enabled }),
    getTemplates: () => invoke<TemplateRule[]>("get_rule_templates"),
    installTemplate: (templateId: string) => invoke<Rule>("install_rule_template", { templateId }),
  },

  ruleImport: {
    scanAiToolCandidates: (options?: ImportExecutionOptions) =>
      invoke<ImportScanResult>("scan_ai_tool_import_candidates", { options }),
    scanFromFile: (path: string, options?: ImportExecutionOptions) =>
      invoke<ImportScanResult>("scan_rule_file_import", { path, options }),
    scanFromDirectory: (path: string, options?: ImportExecutionOptions) =>
      invoke<ImportScanResult>("scan_rule_directory_import", { path, options }),
    scanFromUrl: (url: string, options?: ImportExecutionOptions) =>
      invoke<ImportScanResult>("scan_rule_url_import", { url, options }),
    scanFromClipboard: (content: string, name?: string, options?: ImportExecutionOptions) =>
      invoke<ImportScanResult>("scan_rule_clipboard_import", { content, name, options }),
    importAiToolRules: (options?: ImportExecutionOptions) =>
      invoke<ImportExecutionResult>("import_ai_tool_rules", { options }),
    importAiToolCommands: (options?: ImportExecutionOptions) =>
      invoke<ImportExecutionResult>("import_ai_tool_commands", { options }),
    importAiToolSkills: (options?: ImportExecutionOptions) =>
      invoke<ImportExecutionResult>("import_ai_tool_skills", { options }),
    importFromFile: (path: string, options?: ImportExecutionOptions) =>
      invoke<ImportExecutionResult>("import_rule_from_file", { path, options }),
    importFromDirectory: (path: string, options?: ImportExecutionOptions) =>
      invoke<ImportExecutionResult>("import_rules_from_directory", { path, options }),
    importFromUrl: (url: string, options?: ImportExecutionOptions) =>
      invoke<ImportExecutionResult>("import_rule_from_url", { url, options }),
    importFromClipboard: (content: string, name?: string, options?: ImportExecutionOptions) =>
      invoke<ImportExecutionResult>("import_rule_from_clipboard", { content, name, options }),
    importCommandsFromDirectory: (path: string, options?: ImportExecutionOptions) =>
      invoke<ImportExecutionResult>("import_commands_from_directory", { path, options }),
    scanCommandDirectoryImport: (path: string, options?: ImportExecutionOptions) =>
      invoke<ImportScanResult>("scan_command_directory_import", { path, options }),
    importSkillsFromDirectory: (path: string, options?: ImportExecutionOptions) =>
      invoke<ImportExecutionResult>("import_skills_from_directory", { path, options }),
    scanSkillDirectoryImport: (path: string, options?: ImportExecutionOptions) =>
      invoke<ImportScanResult>("scan_skill_directory_import", { path, options }),
    getHistory: () => invoke<ImportHistoryEntry[]>("get_rule_import_history"),
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
    exportConfiguration: (path: string) => invoke<void>("export_configuration", { path }),
    importConfiguration: (path: string, mode: "overwrite" | "skip") =>
      invoke<void>("import_configuration", { path, mode }),
    previewImport: (path: string) =>
      invoke<{
        version: string;
        exported_at: string;
        rules: Rule[];
        commands: CommandModel[];
        skills: Skill[];
      }>("preview_import", { path }),
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
    getTemplates: () => invoke<TemplateCommand[]>("get_command_templates"),
    installTemplate: (templateId: string) =>
      invoke<CommandModel>("install_command_template", { templateId }),
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

  slashCommands: {
    remove: (commandName: string, adapters: string[]) =>
      invoke<{
        filesWritten: number;
        filesRemoved: number;
        errors: string[];
        conflicts: Array<{ commandName: string; adapterName: string; message: string }>;
      }>("remove_slash_command_files", { commandName, adapters }),
    sync: (commandId: string, isGlobal: boolean) =>
      invoke<{
        filesWritten: number;
        filesRemoved: number;
        errors: string[];
        conflicts: Array<{ commandName: string; adapterName: string; message: string }>;
      }>("sync_slash_command", { commandId, isGlobal }),
    syncAll: (isGlobal: boolean) =>
      invoke<{
        filesWritten: number;
        filesRemoved: number;
        errors: string[];
        conflicts: Array<{ commandName: string; adapterName: string; message: string }>;
      }>("sync_all_slash_commands", { isGlobal }),
    getStatus: (commandId: string) =>
      invoke<Record<string, "Synced" | "OutOfDate" | "NotSynced" | { Error: string }>>(
        "get_slash_command_status",
        { commandId }
      ),
    cleanup: (adapterName: string, isGlobal: boolean) =>
      invoke<number>("cleanup_slash_commands", { adapterName, isGlobal }),
    getAdapters: () =>
      invoke<
        Array<{
          name: string;
          supportsArgumentSubstitution: boolean;
          argumentPattern?: string;
          globalPath: string;
          localPath: string;
        }>
      >("get_slash_command_adapters"),
    testGeneration: (adapterName: string, commandId: string) =>
      invoke<string>("test_slash_command_generation", { adapterName, commandId }),
    getPath: (adapterName: string, commandName: string, isGlobal: boolean) =>
      invoke<string>("get_slash_command_path", { adapterName, commandName, isGlobal }),
  },

  app: {
    getAppDataPath: () => invoke<string>("get_app_data_path_cmd"),
    openInExplorer: (path: string) => invoke<void>("open_in_explorer", { path }),
    getVersion: () => invoke<string>("get_app_version"),
  },

  registry: {
    getTools: () => invoke<ToolEntry[]>("get_tool_registry"),
  },
};
