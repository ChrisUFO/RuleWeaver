import { Switch } from "@/components/ui/switch";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { type Scope, type AdapterType } from "@/types/rule";
import type { ToolEntry } from "@/types/rule";

interface RuleEditorSettingsPanelProps {
  scope: Scope;
  onScopeChange: (scope: Scope) => void;
  targetPaths: string[];
  onToggleTargetPath: (path: string, checked: boolean) => void;
  availableRepos: string[];
  tools: ToolEntry[];
  enabledAdapters: AdapterType[];
  onToggleAdapter: (adapter: AdapterType) => void;
}

export function RuleEditorSettingsPanel({
  scope,
  onScopeChange,
  targetPaths,
  onToggleTargetPath,
  availableRepos,
  tools,
  enabledAdapters,
  onToggleAdapter,
}: RuleEditorSettingsPanelProps) {
  return (
    <Card className="h-fit glass-card premium-shadow border-none overflow-hidden">
      <CardHeader className="bg-white/5 pb-4">
        <CardTitle className="text-sm font-semibold tracking-wide uppercase text-muted-foreground/80">
          Settings
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-6 pt-6">
        {/* Scope */}
        <div className="space-y-2">
          <label className="text-sm font-medium">Scope</label>
          <div className="flex items-center gap-4">
            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="radio"
                name="scope"
                checked={scope === "global"}
                onChange={() => onScopeChange("global")}
                className="h-4 w-4"
              />
              <span className="text-sm">Global</span>
            </label>
            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="radio"
                name="scope"
                checked={scope === "local"}
                onChange={() => onScopeChange("local")}
                className="h-4 w-4"
              />
              <span className="text-sm">Local</span>
            </label>
          </div>
        </div>

        {/* Target Repositories (local scope only) */}
        {scope === "local" && (
          <div className="space-y-2">
            <label className="text-sm font-medium">Target Repositories</label>
            {availableRepos.length === 0 ? (
              <p className="text-xs text-muted-foreground">
                No repositories configured. Add repository roots in Settings first.
              </p>
            ) : (
              <div className="space-y-1">
                {availableRepos.map((repoPath) => (
                  <label
                    key={repoPath}
                    className="flex items-center gap-2 p-2 rounded-md border text-xs"
                  >
                    <input
                      type="checkbox"
                      checked={targetPaths.includes(repoPath)}
                      onChange={(e) => onToggleTargetPath(repoPath, e.target.checked)}
                    />
                    <span className="truncate">{repoPath}</span>
                  </label>
                ))}
              </div>
            )}
          </div>
        )}

        {/* Adapters */}
        <div className="space-y-2">
          <label className="text-sm font-medium">Adapters</label>
          <p className="text-xs text-muted-foreground">
            Select which AI tools should receive this rule
          </p>
          <div className="space-y-2">
            {tools.map((adapter) => {
              const fileName = adapter.paths.localPathTemplate.split(/[/\\]/).pop();
              return (
                <div
                  key={adapter.id}
                  className="flex items-center justify-between p-2 rounded-md hover:bg-accent cursor-pointer transition-colors"
                  onClick={(e) => {
                    if (!(e.target as HTMLElement).closest('[role="switch"]')) {
                      onToggleAdapter(adapter.id);
                    }
                  }}
                >
                  <div className="flex items-center gap-2">
                    <Switch
                      checked={enabledAdapters.includes(adapter.id)}
                      onCheckedChange={() => onToggleAdapter(adapter.id)}
                      aria-label={`Toggle ${adapter.name} adapter`}
                    />
                    <div>
                      <div className="text-sm font-medium">{adapter.name}</div>
                      <div className="text-xs text-muted-foreground">{fileName}</div>
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
