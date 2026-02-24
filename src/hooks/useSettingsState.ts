import { useEffect, useState, useCallback } from "react";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { enable, disable, isEnabled } from "@tauri-apps/plugin-autostart";
import { api } from "@/lib/tauri";
import { toast } from "@/lib/toast-helpers";
import type { useToast } from "@/components/ui/toast";
import { useRepositoryRoots } from "@/hooks/useRepositoryRoots";
import { ADAPTERS, type AdapterType, type Rule } from "@/types/rule";
import type { CommandModel } from "@/types/command";
import type { Skill } from "@/types/skill";

const ADAPTER_SETTINGS_KEY = "adapter_settings";

interface AdapterSettings {
  [key: string]: boolean;
}

interface MigrationProgress {
  total: number;
  migrated: number;
  current_rule?: string;
  status: "NotStarted" | "InProgress" | "Completed" | "Failed" | "RolledBack";
}

interface McpStatus {
  running: boolean;
  port: number;
  uptime_seconds: number;
}

interface McpInstructions {
  claude_code_json: string;
  opencode_json: string;
  standalone_command: string;
}

interface ImportPreview {
  path: string;
  rules: Rule[];
  commands: CommandModel[];
  skills: Skill[];
}

export interface UseSettingsStateReturn {
  appDataPath: string;
  appVersion: string;
  isLoading: boolean;
  adapterSettings: AdapterSettings;
  hasChanges: boolean;
  isSaving: boolean;
  repositoryRoots: string[];
  repoPathsDirty: boolean;
  isSavingRepos: boolean;
  storageMode: "sqlite" | "file";
  storageInfo: Record<string, string> | null;
  isMigratingStorage: boolean;
  backupPath: string;
  migrationProgress: MigrationProgress | null;
  isRollingBack: boolean;
  isVerifyingMigration: boolean;
  mcpStatus: McpStatus | null;
  mcpInstructions: McpInstructions | null;
  isMcpLoading: boolean;
  mcpAutoStart: boolean;
  minimizeToTray: boolean;
  launchOnStartup: boolean;
  mcpLogs: string[];
  isExporting: boolean;
  isImporting: boolean;
  importPreview: ImportPreview | null;
  isImportDialogOpen: boolean;
  importMode: "overwrite" | "skip";
  isCheckingUpdates: boolean;
  updateData: Update | null;
  isUpdateDialogOpen: boolean;
  isUpdating: boolean;
  handlers: {
    toggleAdapter: (adapterId: AdapterType) => void;
    saveSettings: () => Promise<void>;
    handleOpenAppData: () => Promise<void>;
    addRepositoryRoot: () => Promise<void>;
    removeRepositoryRoot: (path: string) => Promise<void>;
    saveRepositoryRoots: () => Promise<void>;
    migrateToFileStorage: () => Promise<void>;
    rollbackMigration: () => Promise<void>;
    verifyMigration: () => Promise<void>;
    startMcp: () => Promise<void>;
    stopMcp: () => Promise<void>;
    refreshMcpStatus: () => Promise<void>;
    toggleMcpAutoStart: (enabled: boolean) => Promise<void>;
    toggleMinimizeToTray: (enabled: boolean) => Promise<void>;
    toggleLaunchOnStartup: (enabled: boolean) => Promise<void>;
    handleExport: () => Promise<void>;
    handleImport: () => Promise<void>;
    executeImport: () => Promise<void>;
    handleCheckUpdates: () => Promise<void>;
    confirmUpdate: () => Promise<void>;
    syncAllSlashCommands: () => Promise<void>;
    setIsImportDialogOpen: (open: boolean) => void;
    setImportMode: (mode: "overwrite" | "skip") => void;
    setIsUpdateDialogOpen: (open: boolean) => void;
  };
}

export function useSettingsState(
  addToast: ReturnType<typeof useToast>["addToast"]
): UseSettingsStateReturn {
  const [appDataPath, setAppDataPath] = useState<string>("");
  const [appVersion, setAppVersion] = useState<string>("");
  const [adapterSettings, setAdapterSettings] = useState<AdapterSettings>(() => {
    const initial: AdapterSettings = {};
    ADAPTERS.forEach((a) => {
      initial[a.id] = a.enabled;
    });
    return initial;
  });
  const [hasChanges, setHasChanges] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [isExporting, setIsExporting] = useState(false);
  const [isImporting, setIsImporting] = useState(false);
  const [isCheckingUpdates, setIsCheckingUpdates] = useState(false);
  const [updateData, setUpdateData] = useState<Update | null>(null);
  const [isUpdateDialogOpen, setIsUpdateDialogOpen] = useState(false);
  const [isUpdating, setIsUpdating] = useState(false);
  const [importPreview, setImportPreview] = useState<ImportPreview | null>(null);
  const [isImportDialogOpen, setIsImportDialogOpen] = useState(false);
  const [importMode, setImportMode] = useState<"overwrite" | "skip">("overwrite");
  const [launchOnStartup, setLaunchOnStartup] = useState(false);
  const [storageMode, setStorageMode] = useState<"sqlite" | "file">("sqlite");
  const [storageInfo, setStorageInfo] = useState<Record<string, string> | null>(null);
  const [isMigratingStorage, setIsMigratingStorage] = useState(false);
  const [backupPath, setBackupPath] = useState<string>("");
  const [migrationProgress, setMigrationProgress] = useState<MigrationProgress | null>(null);
  const [isRollingBack, setIsRollingBack] = useState(false);
  const [isVerifyingMigration, setIsVerifyingMigration] = useState(false);
  const [mcpStatus, setMcpStatus] = useState<McpStatus | null>(null);
  const [mcpInstructions, setMcpInstructions] = useState<McpInstructions | null>(null);
  const [isMcpLoading, setIsMcpLoading] = useState(false);
  const [mcpAutoStart, setMcpAutoStart] = useState(false);
  const [minimizeToTray, setMinimizeToTray] = useState(true);
  const [mcpLogs, setMcpLogs] = useState<string[]>([]);
  const [repoPathsDirty, setRepoPathsDirty] = useState(false);
  const [isSavingRepos, setIsSavingRepos] = useState(false);

  const {
    roots: repositoryRoots,
    setRoots: setRepositoryRoots,
    refresh: refreshRepositoryRoots,
    save: saveRepositoryRootsSetting,
  } = useRepositoryRoots(false);

  useEffect(() => {
    const loadData = async () => {
      setIsLoading(true);
      try {
        const [
          path,
          version,
          settingsJson,
          mode,
          info,
          savedBackupPath,
          progress,
          mcpStatusRes,
          mcpAutoStartSetting,
          minimizeToTraySetting,
          mcpLogsInitial,
          autoStartEnabled,
        ] = await Promise.all([
          api.app.getAppDataPath(),
          api.app.getVersion(),
          api.settings.get(ADAPTER_SETTINGS_KEY),
          api.storage.getMode(),
          api.storage.getInfo(),
          api.settings.get("file_storage_backup_path"),
          api.storage.getMigrationProgress(),
          api.mcp.getStatus(),
          api.settings.get("mcp_auto_start"),
          api.settings.get("minimize_to_tray"),
          api.mcp.getLogs(20),
          isEnabled(),
        ]);
        setAppDataPath(path);
        try {
          const versionResponse = await fetch("/version.json");
          if (versionResponse.ok) {
            const versionData = await versionResponse.json();
            setAppVersion(versionData.version || version);
          } else {
            setAppVersion(version);
          }
        } catch {
          setAppVersion(version);
        }
        setStorageMode(mode === "file" ? "file" : "sqlite");
        setStorageInfo(info);
        setBackupPath(savedBackupPath ?? "");
        setMigrationProgress(progress);
        setMcpStatus(mcpStatusRes);
        setMcpAutoStart(mcpAutoStartSetting === "true");
        setMinimizeToTray(minimizeToTraySetting !== "false");
        setMcpLogs(mcpLogsInitial);
        setLaunchOnStartup(autoStartEnabled);
        await refreshRepositoryRoots();

        if (settingsJson) {
          try {
            const savedSettings = JSON.parse(settingsJson) as AdapterSettings;
            setAdapterSettings((prev) => ({ ...prev, ...savedSettings }));
          } catch {
            console.error("Failed to parse adapter settings");
          }
        }
      } catch (error) {
        console.error("Failed to load settings:", error);
      } finally {
        setIsLoading(false);
      }
    };
    loadData();
  }, [refreshRepositoryRoots]);

  const handleOpenAppData = useCallback(async () => {
    try {
      await api.app.openInExplorer(appDataPath);
    } catch {
      toast.error(addToast, { title: "Error", description: "Could not open folder" });
    }
  }, [appDataPath, addToast]);

  const toggleAdapter = useCallback((adapterId: AdapterType) => {
    setAdapterSettings((prev) => {
      const newSettings = {
        ...prev,
        [adapterId]: !prev[adapterId],
      };
      setHasChanges(true);
      return newSettings;
    });
  }, []);

  const saveSettings = useCallback(async () => {
    setIsSaving(true);
    try {
      await api.settings.set(ADAPTER_SETTINGS_KEY, JSON.stringify(adapterSettings));
      setHasChanges(false);
      toast.success(addToast, {
        title: "Settings Saved",
        description: "Adapter settings have been updated",
      });
    } catch (error) {
      toast.error(addToast, { title: "Save Failed", error });
    } finally {
      setIsSaving(false);
    }
  }, [adapterSettings, addToast]);

  const addRepositoryRoot = useCallback(async () => {
    const { open } = await import("@tauri-apps/plugin-dialog");
    try {
      const selected = await open({ directory: true, multiple: false });
      if (!selected || Array.isArray(selected)) return;
      setRepositoryRoots((prev) => {
        if (prev.includes(selected)) return prev;
        setRepoPathsDirty(true);
        return [...prev, selected];
      });
    } catch {
      toast.error(addToast, {
        title: "Add Repository Failed",
        description: "Could not select repository path",
      });
    }
  }, [setRepositoryRoots, addToast]);

  const removeRepositoryRoot = useCallback(
    async (path: string) => {
      try {
        const [rules, commands, skills] = await Promise.all([
          api.rules.getAll(),
          api.commands.getAll(),
          api.skills.getAll(),
        ]);

        const usedByRule = rules.some((rule) => rule.targetPaths?.some((p) => p === path));
        const usedByCommand = commands.some((command) =>
          command.target_paths?.some((p) => p === path)
        );
        const usedBySkill = skills.some(
          (skill) =>
            skill.scope === "local" && skill.directory_path && skill.directory_path.startsWith(path)
        );

        if (usedByRule || usedByCommand || usedBySkill) {
          toast.error(addToast, {
            title: "Repository In Use",
            description:
              "Cannot remove this repository root while rules, commands, or skills still reference it.",
          });
          return;
        }

        setRepositoryRoots((prev) => prev.filter((p) => p !== path));
        setRepoPathsDirty(true);
      } catch (error) {
        toast.error(addToast, {
          title: "Validation Failed",
          description:
            error instanceof Error ? error.message : "Could not validate repository usage",
        });
      }
    },
    [setRepositoryRoots, addToast]
  );

  const saveRepositoryRoots = useCallback(async () => {
    setIsSavingRepos(true);
    try {
      await saveRepositoryRootsSetting(repositoryRoots);
      setRepoPathsDirty(false);
      toast.success(addToast, {
        title: "Repositories Saved",
        description: "Repository roots updated for local artifact discovery",
      });
    } catch (error) {
      toast.error(addToast, { title: "Save Failed", error });
    } finally {
      setIsSavingRepos(false);
    }
  }, [repositoryRoots, saveRepositoryRootsSetting, addToast]);

  const migrateToFileStorage = useCallback(async () => {
    setIsMigratingStorage(true);
    let poll: ReturnType<typeof setInterval> | null = null;
    try {
      poll = setInterval(async () => {
        try {
          const progress = await api.storage.getMigrationProgress();
          setMigrationProgress(progress);
        } catch {
          // no-op during migration polling
        }
      }, 500);

      const result = await api.storage.migrateToFileStorage();
      clearInterval(poll);
      poll = null;

      if (!result.success) {
        throw new Error(
          result.errors[0]?.error ?? "Migration completed with errors. Check logs for details."
        );
      }

      setBackupPath(result.backup_path ?? "");

      const [mode, info] = await Promise.all([api.storage.getMode(), api.storage.getInfo()]);
      setStorageMode(mode === "file" ? "file" : "sqlite");
      setStorageInfo(info);
      setMigrationProgress(await api.storage.getMigrationProgress());

      toast.success(addToast, {
        title: "Migration Complete",
        description: `Migrated ${result.rules_migrated} rules to file storage.`,
      });
    } catch (error) {
      toast.error(addToast, { title: "Migration Failed", error });
    } finally {
      if (poll) {
        clearInterval(poll);
      }
      setIsMigratingStorage(false);
    }
  }, [addToast]);

  const rollbackMigration = useCallback(async () => {
    if (!backupPath) {
      toast.error(addToast, {
        title: "Rollback Unavailable",
        description: "No backup path available for rollback.",
      });
      return;
    }

    setIsRollingBack(true);
    try {
      await api.storage.rollbackMigration(backupPath);
      setStorageMode("sqlite");
      setMigrationProgress(await api.storage.getMigrationProgress());
      toast.success(addToast, {
        title: "Rollback Complete",
        description: "Database backup restored and file storage disabled.",
      });
    } catch (error) {
      toast.error(addToast, { title: "Rollback Failed", error });
    } finally {
      setIsRollingBack(false);
    }
  }, [backupPath, addToast]);

  const verifyMigration = useCallback(async () => {
    setIsVerifyingMigration(true);
    try {
      const result = await api.storage.verifyMigration();
      if (result.is_valid) {
        toast.success(addToast, {
          title: "Migration Verified",
          description: `Verified ${result.file_rule_count} file rules match ${result.db_rule_count} database rules.`,
        });
      } else {
        toast.error(addToast, {
          title: "Verification Failed",
          description: `${result.missing_rules.length} missing, ${result.mismatched_rules.length} mismatched, ${result.load_errors} load errors.`,
        });
      }
    } catch (error) {
      toast.error(addToast, { title: "Verification Error", error });
    } finally {
      setIsVerifyingMigration(false);
    }
  }, [addToast]);

  const refreshMcpStatus = useCallback(async () => {
    try {
      const [status, logs] = await Promise.all([api.mcp.getStatus(), api.mcp.getLogs(20)]);
      setMcpStatus(status);
      setMcpLogs(logs);
    } catch (error) {
      toast.error(addToast, { title: "MCP Status Error", error });
    }
  }, [addToast]);

  const startMcp = useCallback(async () => {
    setIsMcpLoading(true);
    try {
      await api.mcp.start();
      const [status, instructions] = await Promise.all([
        api.mcp.getStatus(),
        api.mcp.getInstructions(),
      ]);
      setMcpStatus(status);
      setMcpInstructions(instructions);
      setMcpLogs(await api.mcp.getLogs(20));
      toast.success(addToast, {
        title: "MCP Started",
        description: `Server running on port ${status.port}`,
      });
    } catch (error) {
      toast.error(addToast, { title: "MCP Start Failed", error });
    } finally {
      setIsMcpLoading(false);
    }
  }, [addToast]);

  const stopMcp = useCallback(async () => {
    setIsMcpLoading(true);
    try {
      await api.mcp.stop();
      await refreshMcpStatus();
      toast.success(addToast, {
        title: "MCP Stopped",
        description: "Server has been stopped",
      });
    } catch (error) {
      toast.error(addToast, { title: "MCP Stop Failed", error });
    } finally {
      setIsMcpLoading(false);
    }
  }, [refreshMcpStatus, addToast]);

  const toggleMcpAutoStart = useCallback(
    async (enabled: boolean) => {
      setMcpAutoStart(enabled);
      try {
        await api.settings.set("mcp_auto_start", enabled ? "true" : "false");
        toast.success(addToast, {
          title: "MCP Setting Saved",
          description: enabled ? "MCP will auto-start on app launch" : "MCP auto-start disabled",
        });
      } catch (error) {
        setMcpAutoStart(!enabled);
        toast.error(addToast, { title: "Setting Failed", error });
      }
    },
    [addToast]
  );

  const toggleMinimizeToTray = useCallback(
    async (enabled: boolean) => {
      setMinimizeToTray(enabled);
      try {
        await api.settings.set("minimize_to_tray", enabled ? "true" : "false");
        toast.success(addToast, {
          title: "Window Behavior Updated",
          description: enabled
            ? "Closing the window will hide RuleWeaver to tray"
            : "Closing the window will exit RuleWeaver",
        });
      } catch (error) {
        setMinimizeToTray(!enabled);
        toast.error(addToast, { title: "Setting Failed", error });
      }
    },
    [addToast]
  );

  const toggleLaunchOnStartup = useCallback(
    async (enabled: boolean) => {
      setLaunchOnStartup(enabled);
      try {
        if (enabled) {
          await enable();
        } else {
          await disable();
        }
        toast.success(addToast, {
          title: "Startup Preference Saved",
          description: enabled
            ? "RuleWeaver will now launch on startup"
            : "RuleWeaver will no longer launch on startup",
        });
      } catch (error) {
        setLaunchOnStartup(!enabled);
        toast.error(addToast, { title: "Setting Failed", error });
      }
    },
    [addToast]
  );

  const handleExport = useCallback(async () => {
    const { save } = await import("@tauri-apps/plugin-dialog");
    try {
      const selected = await save({
        filters: [
          { name: "JSON", extensions: ["json"] },
          { name: "YAML", extensions: ["yaml", "yml"] },
        ],
        defaultPath: `ruleweaver-config-${new Date().toISOString().split("T")[0]}.json`,
      });

      if (!selected) return;

      setIsExporting(true);
      await api.storage.exportConfiguration(selected);
      toast.success(addToast, {
        title: "Export Successful",
        description: `Configuration exported to ${selected}`,
      });
    } catch (error) {
      toast.error(addToast, { title: "Export Failed", error });
    } finally {
      setIsExporting(false);
    }
  }, [addToast]);

  const handleImport = useCallback(async () => {
    const { open } = await import("@tauri-apps/plugin-dialog");
    try {
      const selected = await open({
        filters: [{ name: "Configuration", extensions: ["json", "yaml", "yml"] }],
        multiple: false,
      });

      if (!selected) return;

      setIsImporting(true);
      const preview = await api.storage.previewImport(selected as string);
      setImportPreview({
        path: selected as string,
        rules: preview.rules,
        commands: preview.commands,
        skills: preview.skills,
      });
      setIsImportDialogOpen(true);
    } catch (error) {
      toast.error(addToast, { title: "Import Error", error });
    } finally {
      setIsImporting(false);
    }
  }, [addToast]);

  const executeImport = useCallback(async () => {
    if (!importPreview) return;

    setIsImporting(true);
    try {
      await api.storage.importConfiguration(importPreview.path, importMode);
      toast.success(addToast, {
        title: "Import Successful",
        description: `Configuration imported using ${importMode} mode.`,
      });
      setIsImportDialogOpen(false);
      setImportPreview(null);
    } catch (error) {
      toast.error(addToast, { title: "Import Failed", error });
    } finally {
      setIsImporting(false);
    }
  }, [importPreview, importMode, addToast]);

  const handleCheckUpdates = useCallback(async () => {
    setIsCheckingUpdates(true);
    try {
      const update = await check();
      if (update) {
        setUpdateData(update);
        setIsUpdateDialogOpen(true);
      } else {
        toast.info(addToast, {
          title: "No Updates",
          description: "You are already using the latest version.",
        });
      }
    } catch (error) {
      toast.error(addToast, { title: "Update Check Failed", error });
    } finally {
      setIsCheckingUpdates(false);
    }
  }, [addToast]);

  const confirmUpdate = useCallback(async () => {
    if (!updateData) return;
    setIsUpdating(true);
    try {
      await updateData.downloadAndInstall();
    } catch (error) {
      toast.error(addToast, { title: "Update Failed", error });
      setIsUpdating(false);
    }
  }, [updateData, addToast]);

  const syncAllSlashCommands = useCallback(async () => {
    try {
      const result = await api.slashCommands.syncAll(true);
      toast[`${result.errors.length > 0 ? "warning" : "success"}`](addToast, {
        title: "Slash Commands Synced",
        description: `Wrote ${result.files_written} files`,
      });
    } catch (error) {
      toast.error(addToast, { title: "Sync Failed", error });
    }
  }, [addToast]);

  return {
    appDataPath,
    appVersion,
    isLoading,
    adapterSettings,
    hasChanges,
    isSaving,
    repositoryRoots,
    repoPathsDirty,
    isSavingRepos,
    storageMode,
    storageInfo,
    isMigratingStorage,
    backupPath,
    migrationProgress,
    isRollingBack,
    isVerifyingMigration,
    mcpStatus,
    mcpInstructions,
    isMcpLoading,
    mcpAutoStart,
    minimizeToTray,
    launchOnStartup,
    mcpLogs,
    isExporting,
    isImporting,
    importPreview,
    isImportDialogOpen,
    importMode,
    isCheckingUpdates,
    updateData,
    isUpdateDialogOpen,
    isUpdating,
    handlers: {
      toggleAdapter,
      saveSettings,
      handleOpenAppData,
      addRepositoryRoot,
      removeRepositoryRoot,
      saveRepositoryRoots,
      migrateToFileStorage,
      rollbackMigration,
      verifyMigration,
      startMcp,
      stopMcp,
      refreshMcpStatus,
      toggleMcpAutoStart,
      toggleMinimizeToTray,
      toggleLaunchOnStartup,
      handleExport,
      handleImport,
      executeImport,
      handleCheckUpdates,
      confirmUpdate,
      syncAllSlashCommands,
      setIsImportDialogOpen,
      setImportMode,
      setIsUpdateDialogOpen,
    },
  };
}
