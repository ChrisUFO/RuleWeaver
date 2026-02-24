import {
  Save,
  Download,
  Upload,
  ExternalLink,
  Info,
  Zap,
  ShieldAlert,
  RefreshCw,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Switch } from "@/components/ui/switch";
import { Checkbox } from "@/components/ui/checkbox";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import { useToast } from "@/components/ui/toast";
import { useSettingsState } from "@/hooks/useSettingsState";
import { McpSettingsCard } from "@/components/settings/McpSettingsCard";
import { StorageSettingsCard } from "@/components/settings/StorageSettingsCard";
import { AdapterSettingsCard } from "@/components/settings/AdapterSettingsCard";
import { RepositorySettingsCard } from "@/components/settings/RepositorySettingsCard";

export function Settings() {
  const { addToast } = useToast();
  const {
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
    handlers,
  } = useSettingsState(addToast);

  return (
    <div className="space-y-6 max-w-3xl">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Settings</h1>
          <p className="text-muted-foreground">Configure RuleWeaver preferences</p>
        </div>
        {hasChanges && (
          <Button onClick={handlers.saveSettings} disabled={isSaving}>
            <Save className="mr-2 h-4 w-4" />
            {isSaving ? "Saving..." : "Save Changes"}
          </Button>
        )}
      </div>

      <Card className="glass-card premium-shadow border-none overflow-hidden">
        <CardHeader className="bg-white/5 pb-4">
          <CardTitle className="text-sm font-semibold tracking-wide uppercase text-muted-foreground/80">
            App Data
          </CardTitle>
          <CardDescription>
            Location where RuleWeaver stores its database and configuration
          </CardDescription>
        </CardHeader>
        <CardContent className="pt-6">
          <div className="flex items-center gap-2">
            <code className="flex-1 p-2.5 rounded-lg bg-black/20 border border-white/5 text-xs font-mono truncate">
              {isLoading ? "Loading..." : appDataPath || "Not available"}
            </code>
            <Button
              variant="outline"
              size="icon"
              onClick={handlers.handleOpenAppData}
              disabled={isLoading}
              className="glass border-white/5 hover:bg-white/5"
              aria-label="Open app data folder"
            >
              <svg
                className="h-4 w-4"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
              >
                <path d="M4 20h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.93a2 2 0 0 1-1.66-.9l-.82-1.2A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13c0 1.1.9 2 2 2Z" />
              </svg>
            </Button>
          </div>
        </CardContent>
      </Card>

      <RepositorySettingsCard
        repositoryRoots={repositoryRoots}
        repoPathsDirty={repoPathsDirty}
        isSavingRepos={isSavingRepos}
        isLoading={isLoading}
        onAdd={handlers.addRepositoryRoot}
        onRemove={handlers.removeRepositoryRoot}
        onSave={handlers.saveRepositoryRoots}
      />

      <McpSettingsCard
        mcpStatus={mcpStatus}
        mcpInstructions={mcpInstructions}
        mcpLogs={mcpLogs}
        isMcpLoading={isMcpLoading}
        mcpAutoStart={mcpAutoStart}
        minimizeToTray={minimizeToTray}
        launchOnStartup={launchOnStartup}
        onStart={handlers.startMcp}
        onStop={handlers.stopMcp}
        onRefresh={handlers.refreshMcpStatus}
        onToggleAutoStart={handlers.toggleMcpAutoStart}
        onToggleMinimizeToTray={handlers.toggleMinimizeToTray}
        onToggleLaunchOnStartup={handlers.toggleLaunchOnStartup}
      />

      <StorageSettingsCard
        storageMode={storageMode}
        storageInfo={storageInfo}
        isMigratingStorage={isMigratingStorage}
        backupPath={backupPath}
        migrationProgress={migrationProgress}
        isRollingBack={isRollingBack}
        isVerifyingMigration={isVerifyingMigration}
        isLoading={isLoading}
        onMigrate={handlers.migrateToFileStorage}
        onRollback={handlers.rollbackMigration}
        onVerify={handlers.verifyMigration}
      />

      <AdapterSettingsCard
        adapterSettings={adapterSettings}
        isLoading={isLoading}
        onToggle={handlers.toggleAdapter}
      />

      <Card className="glass-card premium-shadow border-none overflow-hidden">
        <CardHeader className="bg-white/5 pb-4">
          <CardTitle className="text-sm font-semibold tracking-wide uppercase text-muted-foreground/80">
            Slash Commands
          </CardTitle>
          <CardDescription>Configure native slash command generation for AI tools</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4 pt-6">
          <div className="flex items-center justify-between rounded-md border p-3">
            <div>
              <div className="font-medium">Auto-sync on Save</div>
              <div className="text-sm text-muted-foreground">
                Automatically sync slash commands when saving a command
              </div>
            </div>
            <Switch
              checked={false}
              onCheckedChange={() => {
                addToast({
                  title: "Coming Soon",
                  description: "This feature will be available in a future update",
                  variant: "info",
                });
              }}
              disabled={isLoading}
            />
          </div>

          <div className="pt-2">
            <Button variant="outline" onClick={handlers.syncAllSlashCommands} disabled={isLoading}>
              Sync All Slash Commands
            </Button>
          </div>
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
                onClick={handlers.handleExport}
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
                onClick={handlers.handleImport}
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
                onClick={handlers.handleCheckUpdates}
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

      <Dialog open={isUpdateDialogOpen} onOpenChange={handlers.setIsUpdateDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <Zap className="h-5 w-5 text-yellow-500" />
              Update Available
            </DialogTitle>
            <DialogDescription>A new version of RuleWeaver is available.</DialogDescription>
          </DialogHeader>
          <div className="py-4 space-y-4">
            <div className="rounded-md bg-muted p-3">
              <div className="text-sm font-medium">New Version: {updateData?.version}</div>
              <div className="text-xs text-muted-foreground mt-1">
                Release notes: {updateData?.body || "No release notes available."}
              </div>
            </div>
          </div>
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => handlers.setIsUpdateDialogOpen(false)}
              disabled={isUpdating}
            >
              Later
            </Button>
            <Button onClick={handlers.confirmUpdate} disabled={isUpdating}>
              {isUpdating ? (
                <>
                  <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
                  Updating...
                </>
              ) : (
                "Update Now"
              )}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={isImportDialogOpen} onOpenChange={handlers.setIsImportDialogOpen}>
        <DialogContent className="max-w-md">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <ShieldAlert className="h-5 w-5 text-yellow-500" />
              Confirm Import
            </DialogTitle>
            <DialogDescription>
              Review the items that will be imported into your database. Existing items with the
              same ID will be overwritten.
            </DialogDescription>
          </DialogHeader>
          <div className="py-4 space-y-3">
            <div className="grid grid-cols-3 gap-2">
              <div className="rounded-md border p-3 text-center">
                <div className="text-xl font-bold">{importPreview?.rules.length || 0}</div>
                <div className="text-[10px] uppercase text-muted-foreground">Rules</div>
              </div>
              <div className="rounded-md border p-3 text-center">
                <div className="text-xl font-bold">{importPreview?.commands.length || 0}</div>
                <div className="text-[10px] uppercase text-muted-foreground">Commands</div>
              </div>
              <div className="rounded-md border p-3 text-center">
                <div className="text-xl font-bold">{importPreview?.skills.length || 0}</div>
                <div className="text-[10px] uppercase text-muted-foreground">Skills</div>
              </div>
            </div>
            <p className="text-xs text-muted-foreground italic px-1">
              Source: {importPreview?.path.split(/[/\\]/).pop()}
            </p>
            <div className="flex items-center space-x-2 pt-2 px-1">
              <Checkbox
                id="overwrite"
                checked={importMode === "overwrite"}
                onChange={(checked) => handlers.setImportMode(checked ? "overwrite" : "skip")}
              />
              <label
                htmlFor="overwrite"
                className="text-xs font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
              >
                Overwrite existing items with same ID
              </label>
            </div>
          </div>
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => handlers.setIsImportDialogOpen(false)}
              disabled={isImporting}
            >
              Cancel
            </Button>
            <Button onClick={handlers.executeImport} disabled={isImporting}>
              {isImporting ? (
                <>
                  <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
                  Importing...
                </>
              ) : (
                "Proceed with Import"
              )}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
