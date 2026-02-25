import { Switch } from "@/components/ui/switch";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import type { AdapterType } from "@/types/rule";
import { useRegistryStore } from "@/stores/registryStore";

interface AdapterSettingsCardProps {
  adapterSettings: Record<string, boolean>;
  isLoading: boolean;
  onToggle: (adapterId: AdapterType) => void;
}

export function AdapterSettingsCard({
  adapterSettings,
  isLoading,
  onToggle,
}: AdapterSettingsCardProps) {
  const { tools } = useRegistryStore();

  return (
    <Card className="glass-card premium-shadow border-none overflow-hidden">
      <CardHeader className="bg-white/5 pb-4">
        <CardTitle className="text-sm font-semibold tracking-wide uppercase text-muted-foreground/80">
          Adapters
        </CardTitle>
        <CardDescription>
          Enable or disable adapters for syncing rules to different AI tools
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-3 pt-6">
        {tools.map((adapter) => {
          const fileName = adapter.paths.localPathTemplate.split(/[/\\]/).pop();
          return (
            <div
              key={adapter.id}
              className="flex items-center justify-between p-3 rounded-md border"
            >
              <div className="flex items-center gap-3">
                <Switch
                  checked={adapterSettings[adapter.id] ?? true}
                  onCheckedChange={() => onToggle(adapter.id)}
                  disabled={isLoading}
                />
                <div>
                  <div className="font-medium">{adapter.name}</div>
                  <div className="text-sm text-muted-foreground">{adapter.description}</div>
                </div>
              </div>
              <div className="text-right">
                <Badge variant="outline" className="font-mono text-xs">
                  {fileName}
                </Badge>
                <div className="text-xs text-muted-foreground mt-1">{adapter.paths.globalPath}</div>
              </div>
            </div>
          );
        })}
      </CardContent>
    </Card>
  );
}
