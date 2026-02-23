import { useEffect, useState, useCallback } from "react";
import {
  FolderOpen,
  ExternalLink,
  Info,
  RotateCcw,
  Save,
  ShieldCheck,
  Server,
  RefreshCw,
  Download,
  Upload,
} from "lucide-react";
import { save, open } from "@tauri-apps/plugin-dialog";
import { check } from "@tauri-apps/plugin-updater";
import { enable, disable, isEnabled } from "@tauri-apps/plugin-autostart";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Switch } from "@/components/ui/switch";
import { Badge } from "@/components/ui/badge";
import { api } from "@/lib/tauri";
import { useToast } from "@/components/ui/toast";
import { ADAPTERS, type AdapterType } from "@/types/rule";

const ADAPTER_SETTINGS_KEY = "adapter_settings";

interface AdapterSettings {
  [key: string]: boolean;
}

export function Settings() {
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
  const [launchOnStartup, setLaunchOnStartup] = useState(false);
  const [storageMode, setStorageMode] = useState<"sqlite" | "file">("sqlite");
  const [storageInfo, setStorageInfo] = useState<Record<string, string> | null>(null);
  const [isMigratingStorage, setIsMigratingStorage] = useState(false);
  const [backupPath, setBackupPath] = useState<string>("");
  const [migrationProgress, setMigrationProgress] = useState<{
    total: number;
    migrated: number;
    current_rule?: string;
    status: "NotStarted" | "InProgress" | "Completed" | "Failed" | "RolledBack";
  } | null>(null);
  const [isRollingBack, setIsRollingBack] = useState(false);
  const [isVerifyingMigration, setIsVerifyingMigration] = useState(false);
  const [mcpStatus, setMcpStatus] = useState<{
    running: boolean;
    port: number;
    uptime_seconds: number;
  } | null>(null);
  const [mcpInstructions, setMcpInstructions] = useState<{
    claude_code_json: string;
    opencode_json: string;
    standalone_command: string;
  } | null>(null);
  const [isMcpLoading, setIsMcpLoading] = useState(false);
  const [mcpAutoStart, setMcpAutoStart] = useState(false);
  const [minimizeToTray, setMinimizeToTray] = useState(true);
  const [mcpLogs, setMcpLogs] = useState<string[]>([]);
  const { addToast } = useToast();

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
        setAppVersion(version);
        setStorageMode(mode === "file" ? "file" : "sqlite");
        setStorageInfo(info);
        setBackupPath(savedBackupPath ?? "");
        setMigrationProgress(progress);
        setMcpStatus(mcpStatusRes);
        setMcpAutoStart(mcpAutoStartSetting === "true");
        setMinimizeToTray(minimizeToTraySetting !== "false");
        setMcpLogs(mcpLogsInitial);
        setLaunchOnStartup(autoStartEnabled);

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
  }, []);

  const handleOpenAppData = async () => {
    try {
      await api.app.openInExplorer(appDataPath);
    } catch {
      addToast({
        title: "Error",
        description: "Could not open folder",
        variant: "error",
      });
    }
  };

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

  const saveSettings = async () => {
    setIsSaving(true);
    try {
      await api.settings.set(ADAPTER_SETTINGS_KEY, JSON.stringify(adapterSettings));
      setHasChanges(false);
      addToast({
        title: "Settings Saved",
        description: "Adapter settings have been updated",
        variant: "success",
      });
    } catch (error) {
      addToast({
        title: "Save Failed",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "error",
      });
    } finally {
      setIsSaving(false);
    }
  };

  const migrateToFileStorage = async () => {
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

      addToast({
        title: "Migration Complete",
        description: `Migrated ${result.rules_migrated} rules to file storage.`,
        variant: "success",
      });
    } catch (error) {
      addToast({
        title: "Migration Failed",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "error",
      });
    } finally {
      if (poll) {
        clearInterval(poll);
      }
      setIsMigratingStorage(false);
    }
  };

  const rollbackMigration = async () => {
    if (!backupPath) {
      addToast({
        title: "Rollback Unavailable",
        description: "No backup path available for rollback.",
        variant: "error",
      });
      return;
    }

    setIsRollingBack(true);
    try {
      await api.storage.rollbackMigration(backupPath);
      setStorageMode("sqlite");
      setMigrationProgress(await api.storage.getMigrationProgress());
      addToast({
        title: "Rollback Complete",
        description: "Database backup restored and file storage disabled.",
        variant: "success",
      });
    } catch (error) {
      addToast({
        title: "Rollback Failed",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "error",
      });
    } finally {
      setIsRollingBack(false);
    }
  };

  const verifyMigration = async () => {
    setIsVerifyingMigration(true);
    try {
      const result = await api.storage.verifyMigration();
      if (result.is_valid) {
        addToast({
          title: "Migration Verified",
          description: `Verified ${result.file_rule_count} file rules match ${result.db_rule_count} database rules.`,
          variant: "success",
        });
      } else {
        addToast({
          title: "Verification Failed",
          description: `${result.missing_rules.length} missing, ${result.mismatched_rules.length} mismatched, ${result.load_errors} load errors.`,
          variant: "error",
        });
      }
    } catch (error) {
      addToast({
        title: "Verification Error",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "error",
      });
    } finally {
      setIsVerifyingMigration(false);
    }
  };

  const refreshMcpStatus = async () => {
    try {
      const [status, logs] = await Promise.all([api.mcp.getStatus(), api.mcp.getLogs(20)]);
      setMcpStatus(status);
      setMcpLogs(logs);
    } catch (error) {
      addToast({
        title: "MCP Status Error",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "error",
      });
    }
  };

  const startMcp = async () => {
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
      addToast({
        title: "MCP Started",
        description: `Server running on port ${status.port}`,
        variant: "success",
      });
    } catch (error) {
      addToast({
        title: "MCP Start Failed",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "error",
      });
    } finally {
      setIsMcpLoading(false);
    }
  };

  const stopMcp = async () => {
    setIsMcpLoading(true);
    try {
      await api.mcp.stop();
      await refreshMcpStatus();
      addToast({
        title: "MCP Stopped",
        description: "Server has been stopped",
        variant: "success",
      });
    } catch (error) {
      addToast({
        title: "MCP Stop Failed",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "error",
      });
    } finally {
      setIsMcpLoading(false);
    }
  };

  const toggleMcpAutoStart = async (enabled: boolean) => {
    setMcpAutoStart(enabled);
    try {
      await api.settings.set("mcp_auto_start", enabled ? "true" : "false");
      addToast({
        title: "MCP Setting Saved",
        description: enabled ? "MCP will auto-start on app launch" : "MCP auto-start disabled",
        variant: "success",
      });
    } catch (error) {
      setMcpAutoStart(!enabled);
      addToast({
        title: "Setting Failed",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "error",
      });
    }
  };

  const toggleMinimizeToTray = async (enabled: boolean) => {
    setMinimizeToTray(enabled);
    try {
      await api.settings.set("minimize_to_tray", enabled ? "true" : "false");
      addToast({
        title: "Window Behavior Updated",
        description: enabled
          ? "Closing the window will hide RuleWeaver to tray"
          : "Closing the window will exit RuleWeaver",
        variant: "success",
      });
    } catch (error) {
      setMinimizeToTray(!enabled);
      addToast({
        title: "Setting Failed",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "error",
      });
    }
  };

  const toggleLaunchOnStartup = async (enabled: boolean) => {
    setLaunchOnStartup(enabled);
    try {
      if (enabled) {
        await enable();
      } else {
        await disable();
      }
      addToast({
        title: "Startup Preference Saved",
        description: enabled
          ? "RuleWeaver will now launch on startup"
          : "RuleWeaver will no longer launch on startup",
        variant: "success",
      });
    } catch (error) {
      setLaunchOnStartup(!enabled);
      addToast({
        title: "Setting Failed",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "error",
      });
    }
  };

  const handleExport = async () => {
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
      addToast({
        title: "Export Successful",
        description: `Configuration exported to ${selected}`,
        variant: "success",
      });
    } catch (error) {
      addToast({
        title: "Export Failed",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "error",
      });
    } finally {
      setIsExporting(false);
    }
  };

  const handleImport = async () => {
    try {
      const selected = await open({
        filters: [{ name: "Configuration", extensions: ["json", "yaml", "yml"] }],
        multiple: false,
      });

      if (!selected) return;

      setIsImporting(true);
      await api.storage.importConfiguration(selected as string);
      addToast({
        title: "Import Successful",
        description: "Configuration has been imported and synchronized.",
        variant: "success",
      });
      // Optionally reload data or trigger a refresh in other components
    } catch (error) {
      addToast({
        title: "Import Failed",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "error",
      });
    } finally {
      setIsImporting(false);
    }
  };

  const handleCheckUpdates = async () => {
    setIsCheckingUpdates(true);
    try {
      const update = await check();
      if (update) {
        addToast({
          title: "Update Available",
          description: `Version ${update.version} is available.`,
          variant: "success",
        });

        if (
          confirm(`New version ${update.version} is available. Would you like to install it now?`)
        ) {
          await update.downloadAndInstall();
          // The app will restart automatically after install or might need manual restart depending on platform
        }
      } else {
        addToast({
          title: "No Updates",
          description: "You are already using the latest version.",
        });
      }
    } catch (error) {
      addToast({
        title: "Update Check Failed",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "error",
      });
    } finally {
      setIsCheckingUpdates(false);
    }
  };

  return (
    <div className="space-y-6 max-w-3xl">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Settings</h1>
          <p className="text-muted-foreground">Configure RuleWeaver preferences</p>
        </div>
        {hasChanges && (
          <Button onClick={saveSettings} disabled={isSaving}>
            <Save className="mr-2 h-4 w-4" />
            {isSaving ? "Saving..." : "Save Changes"}
          </Button>
        )}
      </div>

      <Card>
        <CardHeader>
          <CardTitle>App Data</CardTitle>
          <CardDescription>
            Location where RuleWeaver stores its database and configuration
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            <code className="flex-1 p-2 rounded-md bg-muted text-sm truncate">
              {isLoading ? "Loading..." : appDataPath || "Not available"}
            </code>
            <Button
              variant="outline"
              size="icon"
              onClick={handleOpenAppData}
              disabled={isLoading}
              aria-label="Open app data folder"
            >
              <FolderOpen className="h-4 w-4" aria-hidden="true" />
            </Button>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>MCP Server</CardTitle>
          <CardDescription>
            Start and manage the local MCP server for tool integration
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between rounded-md border p-3">
            <div className="flex items-center gap-2">
              <Server className="h-4 w-4" />
              <div>
                <div className="font-medium">Status</div>
                <div className="text-sm text-muted-foreground">
                  {mcpStatus?.running ? `Running on port ${mcpStatus.port}` : "Stopped"}
                </div>
              </div>
            </div>
            <Badge variant={mcpStatus?.running ? "default" : "outline"}>
              {mcpStatus?.running ? "Running" : "Stopped"}
            </Badge>
          </div>

          {mcpStatus?.running && (
            <p className="text-xs text-muted-foreground">Uptime: {mcpStatus.uptime_seconds}s</p>
          )}

          <div className="flex flex-wrap gap-2">
            <Button onClick={startMcp} disabled={isMcpLoading || !!mcpStatus?.running}>
              Start
            </Button>
            <Button
              variant="outline"
              onClick={stopMcp}
              disabled={isMcpLoading || !mcpStatus?.running}
            >
              Stop
            </Button>
            <Button variant="outline" onClick={refreshMcpStatus} disabled={isMcpLoading}>
              <RefreshCw className="mr-2 h-4 w-4" />
              Refresh
            </Button>
          </div>

          <div className="flex items-center justify-between rounded-md border p-3">
            <div>
              <div className="font-medium">Auto-start MCP</div>
              <div className="text-xs text-muted-foreground">
                Start MCP automatically when RuleWeaver launches
              </div>
            </div>
            <Switch checked={mcpAutoStart} onCheckedChange={toggleMcpAutoStart} />
          </div>

          <div className="flex items-center justify-between rounded-md border p-3">
            <div>
              <div className="font-medium">Minimize to tray on close</div>
              <div className="text-xs text-muted-foreground">
                Keep app and MCP running when closing the main window
              </div>
            </div>
            <Switch checked={minimizeToTray} onCheckedChange={toggleMinimizeToTray} />
          </div>

          <div className="flex items-center justify-between rounded-md border p-3">
            <div>
              <div className="font-medium">Launch on startup</div>
              <div className="text-xs text-muted-foreground">
                Automatically start RuleWeaver when you log in
              </div>
            </div>
            <Switch checked={launchOnStartup} onCheckedChange={toggleLaunchOnStartup} />
          </div>

          {mcpInstructions && (
            <div className="space-y-2">
              <code className="block rounded-md bg-muted p-2 text-xs overflow-auto">
                {mcpInstructions.standalone_command}
              </code>
              <div className="grid gap-2 md:grid-cols-2">
                <code className="rounded-md bg-muted p-2 text-xs overflow-auto">
                  {mcpInstructions.claude_code_json}
                </code>
                <code className="rounded-md bg-muted p-2 text-xs overflow-auto">
                  {mcpInstructions.opencode_json}
                </code>
              </div>
            </div>
          )}

          <div className="rounded-md border p-3">
            <div className="mb-2 text-sm font-medium">Recent MCP Logs</div>
            <div className="max-h-40 space-y-1 overflow-auto text-xs text-muted-foreground">
              {mcpLogs.length === 0 && <div>No logs yet.</div>}
              {mcpLogs.map((log, idx) => (
                <div key={`${idx}-${log.slice(0, 20)}`}>{log}</div>
              ))}
            </div>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Storage</CardTitle>
          <CardDescription>
            Manage where rules are stored: legacy SQLite or file-based markdown storage
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between rounded-md border p-3">
            <div>
              <div className="font-medium">Current Mode</div>
              <div className="text-sm text-muted-foreground">
                {storageMode === "file"
                  ? "File storage (.ruleweaver/rules/*.md)"
                  : "SQLite database (legacy)"}
              </div>
            </div>
            <Badge variant={storageMode === "file" ? "default" : "outline"}>
              {storageMode === "file" ? "File" : "SQLite"}
            </Badge>
          </div>

          {storageInfo && (
            <div className="grid grid-cols-1 gap-2 text-sm text-muted-foreground md:grid-cols-3">
              <div>Rules: {storageInfo.rule_count ?? "0"}</div>
              <div>Size: {storageInfo.total_size_bytes ?? "0"} bytes</div>
              <div>Storage Exists: {storageInfo.exists ?? "false"}</div>
            </div>
          )}

          {storageMode !== "file" && (
            <Button onClick={migrateToFileStorage} disabled={isMigratingStorage || isLoading}>
              {isMigratingStorage ? "Migrating..." : "Migrate to File Storage"}
            </Button>
          )}

          {migrationProgress && (
            <div className="rounded-md border p-3 space-y-2">
              <div className="flex items-center justify-between text-sm">
                <span className="text-muted-foreground">Migration Status</span>
                <Badge variant="outline">{migrationProgress.status}</Badge>
              </div>
              <div className="text-sm">
                {migrationProgress.migrated} / {migrationProgress.total || 0} rules migrated
              </div>
              {migrationProgress.current_rule && (
                <div className="text-xs text-muted-foreground truncate">
                  Current: {migrationProgress.current_rule}
                </div>
              )}
            </div>
          )}

          {storageMode === "file" && (
            <div className="flex flex-wrap gap-2">
              <Button variant="outline" onClick={verifyMigration} disabled={isVerifyingMigration}>
                <ShieldCheck className="mr-2 h-4 w-4" />
                {isVerifyingMigration ? "Verifying..." : "Verify Migration"}
              </Button>
              <Button
                variant="outline"
                onClick={rollbackMigration}
                disabled={isRollingBack || !backupPath}
              >
                <RotateCcw className="mr-2 h-4 w-4" />
                {isRollingBack ? "Rolling Back..." : "Rollback"}
              </Button>
            </div>
          )}

          {backupPath && (
            <p className="text-xs text-muted-foreground break-all">Backup: {backupPath}</p>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Adapters</CardTitle>
          <CardDescription>
            Enable or disable adapters for syncing rules to different AI tools
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          {ADAPTERS.map((adapter) => (
            <div
              key={adapter.id}
              className="flex items-center justify-between p-3 rounded-md border"
            >
              <div className="flex items-center gap-3">
                <Switch
                  checked={adapterSettings[adapter.id] ?? true}
                  onCheckedChange={() => toggleAdapter(adapter.id)}
                  disabled={isLoading}
                />
                <div>
                  <div className="font-medium">{adapter.name}</div>
                  <div className="text-sm text-muted-foreground">{adapter.description}</div>
                </div>
              </div>
              <div className="text-right">
                <Badge variant="outline" className="font-mono text-xs">
                  {adapter.fileName}
                </Badge>
                <div className="text-xs text-muted-foreground mt-1">{adapter.globalPath}</div>
              </div>
            </div>
          ))}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Data Management</CardTitle>
          <CardDescription>
            Export and import your RuleWeaver configuration (rules, commands, and skills)
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div className="rounded-md border p-4 space-y-3">
              <div className="flex items-center gap-2 font-medium">
                <Download className="h-4 w-4" />
                Export
              </div>
              <p className="text-sm text-muted-foreground">
                Save your rules, commands, and skills to a JSON or YAML file for backup or sharing.
              </p>
              <Button
                variant="outline"
                className="w-full"
                onClick={handleExport}
                disabled={isExporting}
              >
                {isExporting ? "Exporting..." : "Export Configuration"}
              </Button>
            </div>

            <div className="rounded-md border p-4 space-y-3">
              <div className="flex items-center gap-2 font-medium">
                <Upload className="h-4 w-4" />
                Import
              </div>
              <p className="text-sm text-muted-foreground">
                Load configuration from a file. This will replace existing items with the same ID.
              </p>
              <Button
                variant="outline"
                className="w-full"
                onClick={handleImport}
                disabled={isImporting}
              >
                {isImporting ? "Importing..." : "Import Configuration"}
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>About</CardTitle>
        </CardHeader>
        <CardContent className="space-y-2">
          <div className="flex items-center justify-between">
            <span className="text-muted-foreground">Version</span>
            <div className="flex items-center gap-2">
              <span className="font-mono">{isLoading ? "..." : appVersion}</span>
              <Button
                variant="outline"
                size="sm"
                className="h-7 text-xs"
                onClick={handleCheckUpdates}
                disabled={isCheckingUpdates || isLoading}
              >
                {isCheckingUpdates ? "Checking..." : "Check for Updates"}
              </Button>
            </div>
          </div>
          <div className="flex gap-2 pt-2">
            <a
              href="https://github.com/ChrisUFO/RuleWeaver"
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium border border-input bg-background shadow-sm hover:bg-accent hover:text-accent-foreground h-8 px-3 text-xs"
            >
              <ExternalLink className="h-4 w-4" />
              GitHub
            </a>
            <a
              href="https://github.com/ChrisUFO/RuleWeaver/issues"
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium border border-input bg-background shadow-sm hover:bg-accent hover:text-accent-foreground h-8 px-3 text-xs"
            >
              <Info className="h-4 w-4" />
              Report Issue
            </a>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
