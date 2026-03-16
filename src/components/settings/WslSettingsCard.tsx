import { useEffect, useState } from "react";
import { Monitor, Loader2 } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Switch } from "@/components/ui/switch";
import { Select } from "@/components/ui/select";
import { api } from "@/lib/tauri";
import { toast } from "@/lib/toast-helpers";
import type { useToast } from "@/components/ui/toast";
import type { WslConfig, WslDistribution } from "@/types/wsl";

interface WslSettingsCardProps {
  addToast: ReturnType<typeof useToast>["addToast"];
}

export function WslSettingsCard({ addToast }: WslSettingsCardProps) {
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [isWslInstalled, setIsWslInstalled] = useState(false);
  const [distributions, setDistributions] = useState<WslDistribution[]>([]);
  const [config, setConfig] = useState<WslConfig | null>(null);

  useEffect(() => {
    const loadWslData = async () => {
      setIsLoading(true);
      try {
        const [installed, distros, wslConfig] = await Promise.all([
          api.wsl.isInstalled(),
          api.wsl.listDistributions(),
          api.wsl.getConfig(),
        ]);
        setIsWslInstalled(installed);
        setDistributions(distros);
        setConfig(wslConfig);
      } catch (error) {
        console.error("Failed to load WSL data:", error);
      } finally {
        setIsLoading(false);
      }
    };
    loadWslData();
  }, []);

  const handleToggleWsl = async (enabled: boolean) => {
    if (!config) return;
    setIsSaving(true);
    try {
      await api.wsl.setEnabled(enabled);
      setConfig({ ...config, enabled });
      toast.success(addToast, {
        title: enabled ? "WSL Enabled" : "WSL Disabled",
        description: enabled
          ? "Rules will be synced to WSL distributions"
          : "Rules will be synced to Windows paths",
      });
    } catch (error) {
      toast.error(addToast, { title: "Failed to update WSL setting", error });
    } finally {
      setIsSaving(false);
    }
  };

  const handleSetDefaultDistribution = async (distribution: string) => {
    if (!config) return;
    setIsSaving(true);
    try {
      const newConfig = { ...config, defaultDistribution: distribution };
      await api.wsl.setConfig(newConfig);
      setConfig(newConfig);
      toast.success(addToast, {
        title: "Default Distribution Updated",
        description: `Default WSL distribution set to ${distribution}`,
      });
    } catch (error) {
      toast.error(addToast, { title: "Failed to update default distribution", error });
    } finally {
      setIsSaving(false);
    }
  };

  if (!isWslInstalled) {
    return null;
  }

  const distributionOptions = distributions.map((d) => ({
    value: d.name,
    label: d.isDefault ? `${d.name} (default)` : d.name,
  }));

  return (
    <Card className="glass-card premium-shadow border-none overflow-hidden">
      <CardHeader className="bg-white/5 pb-4">
        <div className="flex items-center gap-2">
          <Monitor className="h-4 w-4 text-muted-foreground" />
          <CardTitle className="text-sm font-bold uppercase tracking-widest text-muted-foreground/60">
            WSL Support
          </CardTitle>
          {isLoading && <Loader2 className="h-3 w-3 animate-spin text-muted-foreground" />}
        </div>
        <CardDescription>
          Sync rules to WSL (Windows Subsystem for Linux) distributions
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4 pt-6">
        <div className="flex items-center justify-between rounded-xl border border-white/5 bg-white/5 p-4">
          <div>
            <div className="font-bold">Enable WSL Support</div>
            <div className="text-xs text-muted-foreground">
              Sync rules to WSL distributions via UNC paths
            </div>
          </div>
          <Switch
            checked={config?.enabled ?? false}
            onCheckedChange={handleToggleWsl}
            disabled={isLoading || isSaving}
          />
        </div>

        {config?.enabled && distributions.length > 0 && (
          <div className="space-y-3">
            <div className="text-sm font-medium text-muted-foreground">Default Distribution</div>
            <Select
              value={config.defaultDistribution ?? ""}
              options={distributionOptions}
              onChange={handleSetDefaultDistribution}
              placeholder="Select a distribution"
              disabled={isSaving}
            />
          </div>
        )}

        {config?.enabled && distributions.length === 0 && !isLoading && (
          <div className="rounded-xl border border-amber-500/20 bg-amber-500/5 p-3 text-xs text-muted-foreground">
            No WSL distributions found. Install a distribution using{" "}
            <code className="bg-white/10 px-1 rounded">wsl --install</code>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
