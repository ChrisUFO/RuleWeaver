import { invoke } from "@tauri-apps/api/core";
import type {
  Rule,
  CreateRuleInput,
  UpdateRuleInput,
  SyncResult,
  SyncHistoryEntry,
  Conflict,
} from "@/types/rule";

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

  app: {
    getAppDataPath: () => invoke<string>("get_app_data_path"),
    openInExplorer: (path: string) => invoke<void>("open_in_explorer", { path }),
    getVersion: () => invoke<string>("get_app_version"),
  },
};
