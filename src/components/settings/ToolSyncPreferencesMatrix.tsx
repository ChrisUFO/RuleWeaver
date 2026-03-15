import { useEffect, useState } from "react";
import { Switch } from "@/components/ui/switch";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { useRegistryStore } from "@/stores/registryStore";
import { api } from "@/lib/tauri";
import { useToast } from "@/components/ui/toast";
import type { ToolSyncPreferences } from "@/types/status";
import type { AdapterType } from "@/types/rule";

const ARTIFACT_TYPES = [
  { key: "rules", label: "Rules", capability: "supportsRules" as const },
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

  useEffect(() => {
    loadPreferences();
  }, []);

  const loadPreferences = async () => {
    setIsLoading(true);
    try {
      const prefs = await api.registry.getAllToolSyncPreferences();
      const prefMap = prefs.reduce(
        (acc, pref) => {
          acc[pref.toolId] = pref;
          return acc;
        },
        {} as Record<AdapterType, ToolSyncPreferences>
      );
      setPreferences(prefMap);
    } catch (error) {
      console.error("Failed to load tool sync preferences", error);
    } finally {
      setIsLoading(false);
    }
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
        description: `Updated sync preference`,
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
        <CardDescription>Control which artifact types sync to each AI tool</CardDescription>
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
                      className="text-center py-2 px-3 text-xs font-semibold uppercase text-muted-foreground"
                    >
                      {type.label}
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {tools.map((tool) => (
                  <tr key={tool.id} className="border-b border-white/5 hover:bg-white/5">
                    <td className="py-3 px-3">
                      <div className="font-medium text-sm">{tool.name}</div>
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
                ))}
              </tbody>
            </table>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
