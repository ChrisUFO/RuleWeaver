import { useEffect, useState, useCallback } from "react";
import { open, save } from "@tauri-apps/plugin-dialog";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { enable, disable, isEnabled } from "@tauri-apps/plugin-autostart";
import { api } from "@/lib/tauri";
import { toast } from "@/lib/toast-helpers";
import type { useToast } from "@/components/ui/toast";
import { useRepositoryRoots } from "@/hooks/useRepositoryRoots";
import { featureManager, FEATURE_FLAGS } from "@/lib/featureManager";
import type { AdapterType, Rule } from "@/types/rule";
import type { CommandModel } from "@/types/command";
import type { Skill } from "@/types/skill";
import type { ScopedSecret, SecretStorageStatus } from "@/types/secret";

const ADAPTER_SETTINGS_KEY = "adapter_settings";

interface AdapterSettings {
  [key: string]: boolean;
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
  scopedSecrets: ScopedSecret[];
  secretStorageStatus: SecretStorageStatus | null;
  selectedSecretWorkspace: string | null;
  isSecretsLoading: boolean;
  isSavingSecrets: boolean;
  storageInfo: Record<string, string> | null;
  minimizeToTray: boolean;
  launchOnStartup: boolean;
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
    selectSecretWorkspace: (workspacePath: string | null) => void;
    saveGlobalSecret: (key: string, value: string) => Promise<void>;
    saveWorkspaceSecret: (key: string, value: string, workspacePath: string) => Promise<void>;
    deleteGlobalSecret: (key: string) => Promise<void>;
    deleteWorkspaceSecret: (key: string, workspacePath: string) => Promise<void>;
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
  addToast: ReturnType<typeof useToast>["addToast"],
  onNavigate?: (view: string) => void
): UseSettingsStateReturn {
  const [appDataPath, setAppDataPath] = useState<string>("");
  const [appVersion, setAppVersion] = useState<string>("");
  const [adapterSettings, setAdapterSettings] = useState<AdapterSettings>({});
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
  const [storageInfo, setStorageInfo] = useState<Record<string, string> | null>(null);
  const [minimizeToTray, setMinimizeToTray] = useState(true);
  const [repoPathsDirty, setRepoPathsDirty] = useState(false);
  const [isSavingRepos, setIsSavingRepos] = useState(false);
  const [scopedSecrets, setScopedSecrets] = useState<ScopedSecret[]>([]);
  const [secretStorageStatus, setSecretStorageStatus] = useState<SecretStorageStatus | null>(null);
  const [selectedSecretWorkspace, setSelectedSecretWorkspace] = useState<string | null>(null);
  const [isSecretsLoading, setIsSecretsLoading] = useState(true);
  const [isSavingSecrets, setIsSavingSecrets] = useState(false);

  const {
    roots: repositoryRoots,
    setRoots: setRepositoryRoots,
    refresh: refreshRepositoryRoots,
    save: saveRepositoryRootsSetting,
  } = useRepositoryRoots(false);

  const refreshScopedSecrets = useCallback(async () => {
    setIsSecretsLoading(true);
    try {
      setScopedSecrets(await api.settings.listScopedSecrets());
    } finally {
      setIsSecretsLoading(false);
    }
  }, []);

  useEffect(() => {
    const loadData = async () => {
      setIsLoading(true);
      try {
        const [
          path,
          version,
          settingsJson,
          info,
          minimizeToTraySetting,
          autoStartEnabled,
          tools,
          scopedSecretsRes,
          secretStorageStatusRes,
        ] = await Promise.all([
          api.app.getAppDataPath(),
          api.app.getVersion(),
          api.settings.get(ADAPTER_SETTINGS_KEY),
          api.storage.getInfo(),
          api.settings.get("minimize_to_tray"),
          isEnabled(),
          api.registry.getTools(),
          api.settings.listScopedSecrets(),
          api.settings.getSecretStorageStatus(),
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
        setStorageInfo(info);
        setMinimizeToTray(minimizeToTraySetting !== "false");
        setLaunchOnStartup(autoStartEnabled);
        setScopedSecrets(scopedSecretsRes);
        setSecretStorageStatus(secretStorageStatusRes);
        setIsSecretsLoading(false);
        await refreshRepositoryRoots();

        let parsedSettings: AdapterSettings = {};
        if (settingsJson) {
          try {
            parsedSettings = JSON.parse(settingsJson) as AdapterSettings;
          } catch {
            console.error("Failed to parse adapter settings");
          }
        }

        const initialSettings: AdapterSettings = {};
        tools.forEach((t) => {
          initialSettings[t.id] = parsedSettings[t.id] ?? true;
        });
        setAdapterSettings(initialSettings);
      } catch (error) {
        setIsSecretsLoading(false);
        console.error("Failed to load settings:", error);
        if (featureManager.isEnabled(FEATURE_FLAGS.ENHANCED_ERROR_UX)) {
          toast.error(addToast, {
            title: "Failed to Load Settings",
            error,
            action: onNavigate
              ? { label: "Open Settings", onClick: () => onNavigate("settings") }
              : undefined,
          });
        }
      } finally {
        setIsLoading(false);
      }
    };
    loadData();
  }, [refreshRepositoryRoots, addToast, onNavigate]);

  useEffect(() => {
    if (selectedSecretWorkspace && !repositoryRoots.includes(selectedSecretWorkspace)) {
      setSelectedSecretWorkspace(null);
    }
  }, [repositoryRoots, selectedSecretWorkspace]);

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
      toast.error(addToast, {
        title: "Save Failed",
        error,
        action: featureManager.isEnabled(FEATURE_FLAGS.ENHANCED_ERROR_UX)
          ? { label: "Retry", onClick: () => saveSettings() }
          : undefined,
      });
    } finally {
      setIsSaving(false);
    }
  }, [adapterSettings, addToast]);

  const addRepositoryRoot = useCallback(async () => {
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
        const [rulesRes, commandsRes, skillsRes] = await Promise.all([
          api.rules.getAll(),
          api.commands.getAll(),
          api.skills.getAll(),
        ]);
        const allArtifacts = [...rulesRes, ...commandsRes, ...skillsRes];
        const artifactUsingPath = allArtifacts.find((a) => {
          if ("target_paths" in a && a.target_paths) {
            return a.target_paths.includes(path);
          }
          return false;
        });
        if (artifactUsingPath) {
          toast.error(addToast, {
            title: "Cannot Remove Path",
            description: `Path is still in use by ${artifactUsingPath.name}. Remove the artifact or change its path first.`,
          });
          return;
        }
        setRepositoryRoots((prev) => {
          const next = prev.filter((r) => r !== path);
          setRepoPathsDirty(prev.length !== next.length);
          return next;
        });
      } catch {
        toast.error(addToast, {
          title: "Remove Repository Failed",
          description: "Could not check for artifact dependencies",
        });
      }
    },
    [setRepositoryRoots, addToast]
  );

  const saveRepositoryRoots = useCallback(async () => {
    setIsSavingRepos(true);
    try {
      await saveRepositoryRootsSetting();
      setRepoPathsDirty(false);
      toast.success(addToast, {
        title: "Repository Roots Saved",
        description: "Local rule paths have been updated",
      });
    } catch (error) {
      toast.error(addToast, { title: "Save Failed", error });
    } finally {
      setIsSavingRepos(false);
    }
  }, [saveRepositoryRootsSetting, addToast]);

  const saveGlobalSecret = useCallback(
    async (key: string, value: string) => {
      setIsSavingSecrets(true);
      try {
        await api.settings.upsertScopedSecret({ key: key.trim(), value, scope: "global" });
        await refreshScopedSecrets();
        toast.success(addToast, {
          title: "Global Secret Saved",
          description: `${key.trim()} is now available as the global baseline`,
        });
      } catch (error) {
        toast.error(addToast, { title: "Secret Save Failed", error });
      } finally {
        setIsSavingSecrets(false);
      }
    },
    [addToast, refreshScopedSecrets]
  );

  const saveWorkspaceSecret = useCallback(
    async (key: string, value: string, workspacePath: string) => {
      setIsSavingSecrets(true);
      try {
        await api.settings.upsertScopedSecret({
          key: key.trim(),
          value,
          scope: "workspace",
          workspacePath,
        });
        await refreshScopedSecrets();
        toast.success(addToast, {
          title: "Workspace Override Saved",
          description: `${key.trim()} now overrides the global value for ${workspacePath}`,
        });
      } catch (error) {
        toast.error(addToast, { title: "Secret Save Failed", error });
      } finally {
        setIsSavingSecrets(false);
      }
    },
    [addToast, refreshScopedSecrets]
  );

  const deleteGlobalSecret = useCallback(
    async (key: string) => {
      setIsSavingSecrets(true);
      try {
        await api.settings.deleteScopedSecret({ key, scope: "global" });
        await refreshScopedSecrets();
        toast.success(addToast, {
          title: "Global Secret Deleted",
          description: `${key.trim()} has been removed from the shared baseline`,
        });
      } catch (error) {
        toast.error(addToast, { title: "Secret Delete Failed", error });
      } finally {
        setIsSavingSecrets(false);
      }
    },
    [addToast, refreshScopedSecrets]
  );

  const deleteWorkspaceSecret = useCallback(
    async (key: string, workspacePath: string) => {
      setIsSavingSecrets(true);
      try {
        await api.settings.deleteScopedSecret({ key, scope: "workspace", workspacePath });
        await refreshScopedSecrets();
        toast.success(addToast, {
          title: "Workspace Override Removed",
          description: `${key.trim()} now falls back to the inherited global value`,
        });
      } catch (error) {
        toast.error(addToast, { title: "Secret Delete Failed", error });
      } finally {
        setIsSavingSecrets(false);
      }
    },
    [addToast, refreshScopedSecrets]
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
        description: `Configuration exported to ${selected}. Secret values stay local and are never included.`,
      });
    } catch (error) {
      toast.error(addToast, { title: "Export Failed", error });
    } finally {
      setIsExporting(false);
    }
  }, [addToast]);

  const handleImport = useCallback(async () => {
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
        description: `Configuration imported using ${importMode} mode. Re-enter secrets locally because secret values are never imported.`,
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
        description: `Wrote ${result.filesWritten} files`,
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
    scopedSecrets,
    secretStorageStatus,
    selectedSecretWorkspace,
    isSecretsLoading,
    isSavingSecrets,
    storageInfo,
    minimizeToTray,
    launchOnStartup,
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
      selectSecretWorkspace: setSelectedSecretWorkspace,
      saveGlobalSecret,
      saveWorkspaceSecret,
      deleteGlobalSecret,
      deleteWorkspaceSecret,
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
