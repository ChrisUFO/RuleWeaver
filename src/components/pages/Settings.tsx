import { useState } from "react";
import {
  Save,
  Download,
  Upload,
  ExternalLink,
  Info,
  Zap,
  ShieldAlert,
  RefreshCw,
  Settings as SettingsIcon,
  Layers,
  Database,
  Cpu,
} from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
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
import { SettingsTabs } from "@/components/settings/SettingsTabs";

const SETTINGS_TABS = [
  { id: "general", label: "General", icon: SettingsIcon },
  { id: "context", label: "Context", icon: Layers },
  { id: "capabilities", label: "Capabilities", icon: Cpu },
  { id: "infrastructure", label: "Infrastructure", icon: Database },
];

export function Settings() {
  const { addToast } = useToast();
  const [activeTab, setActiveTab] = useState("general");

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

  const containerVariants = {
    hidden: { opacity: 0, y: 10 },
    visible: { opacity: 1, y: 0 },
    exit: { opacity: 0, y: -10 },
  };

  return (
    <div className="space-y-6 max-w-5xl mx-auto">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-4xl font-black tracking-tight luminescent-text">Preferences</h1>
          <p className="text-muted-foreground font-medium">Configure your RuleWeaver environment</p>
        </div>
        <AnimatePresence>
          {hasChanges && (
            <motion.div
              initial={{ opacity: 0, scale: 0.95 }}
              animate={{ opacity: 1, scale: 1 }}
              exit={{ opacity: 0, scale: 0.95 }}
            >
              <Button onClick={handlers.saveSettings} disabled={isSaving} className="glow-primary">
                <Save className="mr-2 h-4 w-4" />
                {isSaving ? "Saving..." : "Save Changes"}
              </Button>
            </motion.div>
          )}
        </AnimatePresence>
      </div>

      <SettingsTabs tabs={SETTINGS_TABS} activeTab={activeTab} onTabChange={setActiveTab} />

      <div className="min-h-[400px]">
        <AnimatePresence mode="wait">
          {activeTab === "general" && (
            <motion.div
              key="general"
              variants={containerVariants}
              initial="hidden"
              animate="visible"
              exit="exit"
              className="space-y-6"
            >
              <Card className="glass-card premium-shadow border-none overflow-hidden">
                <CardHeader className="bg-white/5 pb-4">
                  <CardTitle className="text-sm font-bold uppercase tracking-widest text-muted-foreground/60">
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
                    >
                      <Download className="h-4 w-4" />
                    </Button>
                  </div>
                </CardContent>
              </Card>

              <Card className="glass-card premium-shadow border-none overflow-hidden">
                <CardHeader className="bg-white/5 pb-4">
                  <CardTitle className="text-sm font-bold uppercase tracking-widest text-muted-foreground/60">
                    About
                  </CardTitle>
                </CardHeader>
                <CardContent className="space-y-4 pt-6">
                  <div className="flex items-center justify-between">
                    <span className="text-muted-foreground font-medium">Version</span>
                    <div className="flex items-center gap-2">
                      <span className="font-mono text-sm px-2 py-1 bg-white/5 rounded border border-white/5">
                        {isLoading ? "..." : appVersion}
                      </span>
                      <Button
                        variant="outline"
                        size="sm"
                        className="h-8 text-[10px] font-black uppercase tracking-widest"
                        onClick={handlers.handleCheckUpdates}
                        disabled={isCheckingUpdates || isLoading}
                      >
                        {isCheckingUpdates ? "Checking..." : "Check for Updates"}
                      </Button>
                    </div>
                  </div>
                  <div className="flex gap-2 pt-2">
                    <Button variant="outline" className="flex-1 h-9 glass" asChild>
                      <a
                        href="https://github.com/ChrisUFO/RuleWeaver"
                        target="_blank"
                        rel="noopener noreferrer"
                      >
                        <ExternalLink className="mr-2 h-4 w-4" />
                        GitHub
                      </a>
                    </Button>
                    <Button variant="outline" className="flex-1 h-9 glass" asChild>
                      <a
                        href="https://github.com/ChrisUFO/RuleWeaver/issues"
                        target="_blank"
                        rel="noopener noreferrer"
                      >
                        <Info className="mr-2 h-4 w-4" />
                        Report Issue
                      </a>
                    </Button>
                  </div>
                </CardContent>
              </Card>
            </motion.div>
          )}

          {activeTab === "context" && (
            <motion.div
              key="context"
              variants={containerVariants}
              initial="hidden"
              animate="visible"
              exit="exit"
              className="space-y-6"
            >
              <RepositorySettingsCard
                repositoryRoots={repositoryRoots}
                repoPathsDirty={repoPathsDirty}
                isSavingRepos={isSavingRepos}
                isLoading={isLoading}
                onAdd={handlers.addRepositoryRoot}
                onRemove={handlers.removeRepositoryRoot}
                onSave={handlers.saveRepositoryRoots}
              />
            </motion.div>
          )}

          {activeTab === "capabilities" && (
            <motion.div
              key="capabilities"
              variants={containerVariants}
              initial="hidden"
              animate="visible"
              exit="exit"
              className="space-y-6"
            >
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

              <AdapterSettingsCard
                adapterSettings={adapterSettings}
                isLoading={isLoading}
                onToggle={handlers.toggleAdapter}
              />

              <Card className="glass-card premium-shadow border-none overflow-hidden">
                <CardHeader className="bg-white/5 pb-4">
                  <CardTitle className="text-sm font-bold uppercase tracking-widest text-muted-foreground/60">
                    Slash Commands
                  </CardTitle>
                  <CardDescription>
                    Configure native slash command generation for AI tools
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4 pt-6">
                  <div className="flex items-center justify-between rounded-xl border border-white/5 bg-white/5 p-4">
                    <div>
                      <div className="font-bold">Auto-sync on Save</div>
                      <div className="text-xs text-muted-foreground">
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
                    <Button
                      variant="outline"
                      className="glass"
                      onClick={handlers.syncAllSlashCommands}
                      disabled={isLoading}
                    >
                      <RefreshCw className="mr-2 h-4 w-4" />
                      Sync All Slash Commands
                    </Button>
                  </div>
                </CardContent>
              </Card>
            </motion.div>
          )}

          {activeTab === "infrastructure" && (
            <motion.div
              key="infrastructure"
              variants={containerVariants}
              initial="hidden"
              animate="visible"
              exit="exit"
              className="space-y-6"
            >
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

              <Card className="glass-card premium-shadow border-none overflow-hidden">
                <CardHeader className="bg-white/5 pb-4">
                  <CardTitle className="text-sm font-bold uppercase tracking-widest text-muted-foreground/60">
                    Data Management
                  </CardTitle>
                  <CardDescription>Export and import your RuleWeaver configuration</CardDescription>
                </CardHeader>
                <CardContent className="pt-6">
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div className="rounded-xl border border-white/5 bg-white/5 p-4 space-y-3">
                      <div className="flex items-center gap-2 font-bold text-sm">
                        <Download className="h-4 w-4" />
                        Export
                      </div>
                      <p className="text-xs text-muted-foreground leading-relaxed">
                        Save your rules, commands, and skills to a JSON or YAML file for backup or
                        sharing.
                      </p>
                      <Button
                        variant="outline"
                        className="w-full glass h-8 text-xs"
                        onClick={handlers.handleExport}
                        disabled={isExporting}
                      >
                        {isExporting ? "Exporting..." : "Export Configuration"}
                      </Button>
                    </div>

                    <div className="rounded-xl border border-white/5 bg-white/5 p-4 space-y-3">
                      <div className="flex items-center gap-2 font-bold text-sm">
                        <Upload className="h-4 w-4" />
                        Import
                      </div>
                      <p className="text-xs text-muted-foreground leading-relaxed">
                        Load configuration from a file. This will replace existing items with the
                        same ID.
                      </p>
                      <Button
                        variant="outline"
                        className="w-full glass h-8 text-xs"
                        onClick={handlers.handleImport}
                        disabled={isImporting}
                      >
                        {isImporting ? "Importing..." : "Import Configuration"}
                      </Button>
                    </div>
                  </div>
                </CardContent>
              </Card>
            </motion.div>
          )}
        </AnimatePresence>
      </div>

      {/* Dialogs */}
      <Dialog open={isUpdateDialogOpen} onOpenChange={handlers.setIsUpdateDialogOpen}>
        <DialogContent className="glass">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <Zap className="h-5 w-5 text-amber-500" />
              Update Available
            </DialogTitle>
            <DialogDescription>A new version of RuleWeaver is available.</DialogDescription>
          </DialogHeader>
          <div className="py-4 space-y-4">
            <div className="rounded-xl bg-white/5 border border-white/5 p-4">
              <div className="text-sm font-bold">New Version: {updateData?.version}</div>
              <div className="text-xs text-muted-foreground mt-2 leading-relaxed">
                Release notes: {updateData?.body || "No release notes available."}
              </div>
            </div>
          </div>
          <DialogFooter>
            <Button
              variant="outline"
              className="glass"
              onClick={() => handlers.setIsUpdateDialogOpen(false)}
              disabled={isUpdating}
            >
              Later
            </Button>
            <Button onClick={handlers.confirmUpdate} disabled={isUpdating} className="glow-primary">
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
        <DialogContent className="max-w-md glass">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <ShieldAlert className="h-5 w-5 text-amber-500" />
              Confirm Import
            </DialogTitle>
            <DialogDescription>
              Review the items that will be imported. Existing items with the same ID will be
              overwritten.
            </DialogDescription>
          </DialogHeader>
          <div className="py-4 space-y-4">
            <div className="grid grid-cols-3 gap-2">
              <div className="rounded-xl border border-white/5 bg-white/5 p-3 text-center">
                <div className="text-xl font-black">{importPreview?.rules.length || 0}</div>
                <div className="text-[9px] uppercase font-black text-muted-foreground/60">
                  Rules
                </div>
              </div>
              <div className="rounded-xl border border-white/5 bg-white/5 p-3 text-center">
                <div className="text-xl font-black">{importPreview?.commands.length || 0}</div>
                <div className="text-[9px] uppercase font-black text-muted-foreground/60">
                  Commands
                </div>
              </div>
              <div className="rounded-xl border border-white/5 bg-white/5 p-3 text-center">
                <div className="text-xl font-black">{importPreview?.skills.length || 0}</div>
                <div className="text-[9px] uppercase font-black text-muted-foreground/60">
                  Skills
                </div>
              </div>
            </div>
            <p className="text-[10px] text-muted-foreground font-mono px-1">
              File: {importPreview?.path.split(/[/\\]/).pop()}
            </p>
            <div className="flex items-center space-x-3 pt-2 px-1">
              <Checkbox
                id="overwrite"
                checked={importMode === "overwrite"}
                onChange={(checked: boolean) =>
                  handlers.setImportMode(checked ? "overwrite" : "skip")
                }
              />
              <label
                htmlFor="overwrite"
                className="text-xs font-bold leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
              >
                Overwrite existing items with same ID
              </label>
            </div>
          </div>
          <DialogFooter>
            <Button
              variant="outline"
              className="glass"
              onClick={() => handlers.setIsImportDialogOpen(false)}
              disabled={isImporting}
            >
              Cancel
            </Button>
            <Button
              onClick={handlers.executeImport}
              disabled={isImporting}
              className="glow-primary"
            >
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
