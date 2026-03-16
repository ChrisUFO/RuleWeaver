import { useEffect, useState } from "react";
import { Switch } from "@/components/ui/switch";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { useRegistryStore } from "@/stores/registryStore";
import { api } from "@/lib/tauri";
import { useToast } from "@/components/ui/toast";
import type { InstalledToolInfo, ToolSyncPreferences } from "@/types/status";
import type { AdapterType } from "@/types/rule";

// Commands column uses supportsSlashCommands because the sync engine maps both
// CommandStub and SlashCommand artifacts to the sync_commands preference field.
// All tools that support command_stubs also support slash_commands, so this
// capability check correctly covers both artifact types.
const ARTIFACT_TYPES = [
  { key: "rules", label: "Rules", capability: "supportsRules" as const },
  // Commands includes both CommandStub and SlashCommand artifacts.
  // The sync engine maps both types to `sync_commands` (see sync/mod.rs).
  // Using supportsSlashCommands as the capability indicator works because:
  // - All tools supporting command_stubs also support slash_commands
  // - Tools supporting only slash_commands (Cursor, Augment) correctly show the toggle
  // - Tools supporting neither type show "N/A" badge
  { key: "commands", label: "Commands", capability: "supportsSlashCommands" as const },
  { key: "skills", label: "Skills", capability: "supportsSkills" as const },
] as const;

type ArtifactTypeKey = (typeof ARTIFACT_TYPES)[number]["key"];

export function ToolSyncPreferencesMatrix() {
  const { tools } = useRegistryStore();
  const { addToast } = useToast();
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [preferences, setPreferences] = useState<Record<AdapterType, ToolSyncPreferences>>(
    {} as Record<AdapterType, ToolSyncPreferences>
  );
  const [installedTools, setInstalledTools] = useState<InstalledToolInfo[]>([]);

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    setIsLoading(true);
    try {
      const [prefs, installed] = await Promise.all([
        api.registry.getAllToolSyncPreferences(),
        api.registry.detectInstalledTools(),
      ]);
      const prefMap = prefs.reduce(
        (acc, pref) => {
          acc[pref.toolId] = pref;
          return acc;
        },
        {} as Record<AdapterType, ToolSyncPreferences>
      );
      setPreferences(prefMap);
      setInstalledTools(installed);
    } catch (error) {
      console.error("Failed to load tool sync preferences", error);
    } finally {
      setIsLoading(false);
    }
  };

  const isToolInstalled = (toolId: AdapterType): boolean => {
    const tool = installedTools.find((t) => t.adapter === toolId);
    return tool?.isInstalled ?? false;
  };

  const handleToggle = async (toolId: AdapterType, artifactType: ArtifactTypeKey) => {
    const currentPref = preferences[toolId] || {
      toolId,
      syncRules: true,
      syncCommands: true,
      syncSkills: true,
    };

    const key =
      artifactType === "rules"
        ? "syncRules"
        : artifactType === "commands"
          ? "syncCommands"
          : "syncSkills";
    const newPref: ToolSyncPreferences = {
      ...currentPref,
      [key]: !currentPref[key],
    };

    const toolName = tools.find((t) => t.id === toolId)?.name ?? toolId;
    const enabled = currentPref[key];

    setPreferences((prev) => ({ ...prev, [toolId]: newPref }));
    setIsSaving(true);

    try {
      await api.registry.upsertToolSyncPreferences({
        toolId: newPref.toolId,
        syncRules: newPref.syncRules,
        syncCommands: newPref.syncCommands,
        syncSkills: newPref.syncSkills,
      });
      addToast({
        title: "Preference Saved",
        description: `${artifactType} sync ${enabled ? "enabled" : "disabled"} for ${toolName}`,
        variant: "success",
      });
    } catch {
      addToast({
        title: "Error",
        description: `Failed to save preference`,
        variant: "error",
      });
      setPreferences((prev) => ({ ...prev, [toolId]: currentPref }));
    } finally {
      setIsSaving(false);
    }
  };

  const handleToggleRow = async (toolId: AdapterType) => {
    const tool = tools.find((t) => t.id === toolId);
    if (!tool) return;

    const currentPref = preferences[toolId] || {
      toolId,
      syncRules: true,
      syncCommands: true,
      syncSkills: true,
    };

    const allEnabled = [
      tool.capabilities.supportsRules ? currentPref.syncRules : true,
      tool.capabilities.supportsSlashCommands ? currentPref.syncCommands : true,
      tool.capabilities.supportsSkills ? currentPref.syncSkills : true,
    ].every(Boolean);

    const newPref: ToolSyncPreferences = {
      ...currentPref,
      syncRules: tool.capabilities.supportsRules ? !allEnabled : currentPref.syncRules,
      syncCommands: tool.capabilities.supportsSlashCommands
        ? !allEnabled
        : currentPref.syncCommands,
      syncSkills: tool.capabilities.supportsSkills ? !allEnabled : currentPref.syncSkills,
    };

    setPreferences((prev) => ({ ...prev, [toolId]: newPref }));
    setIsSaving(true);

    try {
      await api.registry.upsertToolSyncPreferences({
        toolId: newPref.toolId,
        syncRules: newPref.syncRules,
        syncCommands: newPref.syncCommands,
        syncSkills: newPref.syncSkills,
      });
      addToast({
        title: "Preferences Saved",
        description: `${allEnabled ? "Disabled" : "Enabled"} all for ${tool.name}`,
        variant: "success",
      });
    } catch {
      addToast({
        title: "Error",
        description: `Failed to save preferences`,
        variant: "error",
      });
      setPreferences((prev) => ({ ...prev, [toolId]: currentPref }));
    } finally {
      setIsSaving(false);
    }
  };

  const handleToggleColumn = async (artifactType: ArtifactTypeKey) => {
    const key =
      artifactType === "rules"
        ? "syncRules"
        : artifactType === "commands"
          ? "syncCommands"
          : "syncSkills";
    const capability =
      artifactType === "rules"
        ? "supportsRules"
        : artifactType === "commands"
          ? "supportsSlashCommands"
          : "supportsSkills";

    const supportedTools = tools.filter((t) => t.capabilities[capability]);
    const allEnabled = supportedTools.every((t) => {
      const pref = preferences[t.id];
      return pref ? pref[key] : true;
    });

    const updates: Promise<ToolSyncPreferences>[] = supportedTools.map(async (tool) => {
      const currentPref = preferences[tool.id] || {
        toolId: tool.id,
        syncRules: true,
        syncCommands: true,
        syncSkills: true,
      };
      const newPref: ToolSyncPreferences = {
        ...currentPref,
        [key]: !allEnabled,
      };
      setPreferences((prev) => ({ ...prev, [tool.id]: newPref }));
      return api.registry.upsertToolSyncPreferences({
        toolId: newPref.toolId,
        syncRules: newPref.syncRules,
        syncCommands: newPref.syncCommands,
        syncSkills: newPref.syncSkills,
      });
    });

    setIsSaving(true);
    try {
      await Promise.all(updates);
      addToast({
        title: "Preferences Saved",
        description: `${allEnabled ? "Disabled" : "Enabled"} ${artifactType} for all tools`,
        variant: "success",
      });
    } catch {
      addToast({
        title: "Error",
        description: `Failed to save preferences`,
        variant: "error",
      });
      loadData();
    } finally {
      setIsSaving(false);
    }
  };

  const getPrefValue = (toolId: AdapterType, artifactType: ArtifactTypeKey): boolean => {
    const pref = preferences[toolId];
    if (!pref) return true;
    const key =
      artifactType === "rules"
        ? "syncRules"
        : artifactType === "commands"
          ? "syncCommands"
          : "syncSkills";
    return pref[key] as boolean;
  };

  return (
    <Card className="glass-card premium-shadow border-none overflow-hidden">
      <CardHeader className="bg-white/5 pb-4">
        <CardTitle className="text-sm font-semibold tracking-wide uppercase text-muted-foreground/80">
          Tool Sync Preferences
        </CardTitle>
        <CardDescription>
          Control which artifact types sync to each AI tool. Click headers to toggle all.
        </CardDescription>
      </CardHeader>
      <CardContent className="pt-6">
        {isLoading ? (
          <div className="text-center py-8 text-muted-foreground">Loading preferences...</div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="border-b">
                  <th className="text-left py-2 px-3 text-xs font-semibold uppercase text-muted-foreground">
                    Tool
                  </th>
                  {ARTIFACT_TYPES.map((type) => (
                    <th
                      key={type.key}
                      className="text-center py-2 px-3 text-xs font-semibold uppercase text-muted-foreground cursor-pointer hover:text-foreground transition-colors"
                      onClick={() => handleToggleColumn(type.key)}
                      title={`Click to toggle all ${type.label.toLowerCase()}`}
                    >
                      {type.label}
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {tools.map((tool) => {
                  const installed = isToolInstalled(tool.id);
                  return (
                    <tr key={tool.id} className="border-b border-white/5 hover:bg-white/5">
                      <td
                        className="py-3 px-3 cursor-pointer"
                        onClick={() => handleToggleRow(tool.id)}
                        title={`Click to toggle all for ${tool.name}`}
                      >
                        <div className="flex items-center gap-2">
                          <span className="font-medium text-sm hover:text-foreground transition-colors">
                            {tool.name}
                          </span>
                          {!installed && (
                            <Badge
                              variant="outline"
                              className="text-[10px] px-1.5 py-0.5 border-amber-500/30 text-amber-500"
                            >
                              Not Installed
                            </Badge>
                          )}
                        </div>
                      </td>
                      {ARTIFACT_TYPES.map((type) => {
                        const supported = tool.capabilities[type.capability];
                        const enabled = getPrefValue(tool.id, type.key);
                        return (
                          <td key={type.key} className="text-center py-3 px-3">
                            {supported ? (
                              <Switch
                                checked={enabled}
                                onCheckedChange={() => handleToggle(tool.id, type.key)}
                                disabled={isSaving}
                              />
                            ) : (
                              <Badge variant="outline" className="text-xs opacity-50">
                                N/A
                              </Badge>
                            )}
                          </td>
                        );
                      })}
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
