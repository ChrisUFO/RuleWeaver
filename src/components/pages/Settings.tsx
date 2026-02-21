import { useEffect, useState, useCallback } from "react";
import { FolderOpen, ExternalLink, Info, Save } from "lucide-react";
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
  const { addToast } = useToast();

  useEffect(() => {
    const loadData = async () => {
      setIsLoading(true);
      try {
        const [path, version, settingsJson] = await Promise.all([
          api.app.getAppDataPath(),
          api.app.getVersion(),
          api.settings.get(ADAPTER_SETTINGS_KEY),
        ]);
        setAppDataPath(path);
        setAppVersion(version);

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
            <Button variant="outline" size="icon" onClick={handleOpenAppData} disabled={isLoading}>
              <FolderOpen className="h-4 w-4" />
            </Button>
          </div>
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
          <CardTitle>About</CardTitle>
        </CardHeader>
        <CardContent className="space-y-2">
          <div className="flex items-center justify-between">
            <span className="text-muted-foreground">Version</span>
            <span className="font-mono">{isLoading ? "..." : appVersion}</span>
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
