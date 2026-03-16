import { useState } from "react";
import { ChevronDown, ExternalLink } from "lucide-react";
import { Switch } from "@/components/ui/switch";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible";
import { type Scope, type AdapterType } from "@/types/rule";
import type { ToolEntry } from "@/types/rule";
import { cn } from "@/lib/utils";

interface RuleEditorSettingsPanelProps {
  scope: Scope;
  onScopeChange: (scope: Scope) => void;
  targetPaths: string[];
  onToggleTargetPath: (path: string, checked: boolean) => void;
  availableRepos: string[];
  tools: ToolEntry[];
  enabledAdapters: AdapterType[];
  onToggleAdapter: (adapter: AdapterType) => void;
  getAdapterPath: (adapter: AdapterType) => string;
  onOpenFolder: (adapter: AdapterType) => Promise<void>;
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
  getAdapterPath,
  onOpenFolder,
}: RuleEditorSettingsPanelProps) {
  const [scopeOpen, setScopeOpen] = useState(true);
  const [reposOpen, setReposOpen] = useState(true);
  const [adaptersOpen, setAdaptersOpen] = useState(true);

  return (
    <Card className="h-fit glass-card premium-shadow border-none overflow-hidden">
      <CardHeader className="bg-white/5 pb-4">
        <CardTitle className="text-sm font-semibold tracking-wide uppercase text-muted-foreground/80">
          Settings
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-2 pt-4">
        <Collapsible open={scopeOpen} onOpenChange={setScopeOpen}>
          <CollapsibleTrigger className="flex items-center justify-between w-full py-2 text-sm font-medium hover:text-primary transition-colors">
            <span>Scope</span>
            <ChevronDown
              className={cn("h-4 w-4 transition-transform duration-200", scopeOpen && "rotate-180")}
            />
          </CollapsibleTrigger>
          <CollapsibleContent className="pt-2 pb-1">
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
          </CollapsibleContent>
        </Collapsible>

        {scope === "local" && (
          <Collapsible open={reposOpen} onOpenChange={setReposOpen}>
            <CollapsibleTrigger className="flex items-center justify-between w-full py-2 text-sm font-medium hover:text-primary transition-colors">
              <span>Target Repositories</span>
              <ChevronDown
                className={cn(
                  "h-4 w-4 transition-transform duration-200",
                  reposOpen && "rotate-180"
                )}
              />
            </CollapsibleTrigger>
            <CollapsibleContent className="pt-2 pb-1">
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
            </CollapsibleContent>
          </Collapsible>
        )}

        <Collapsible open={adaptersOpen} onOpenChange={setAdaptersOpen}>
          <CollapsibleTrigger className="flex items-center justify-between w-full py-2 text-sm font-medium hover:text-primary transition-colors">
            <span>Adapters</span>
            <ChevronDown
              className={cn(
                "h-4 w-4 transition-transform duration-200",
                adaptersOpen && "rotate-180"
              )}
            />
          </CollapsibleTrigger>
          <CollapsibleContent className="pt-2 pb-1">
            {tools.length === 0 ? (
              <p className="text-xs text-muted-foreground">
                No adapters available. Configure AI tools in Settings to enable rule generation.
              </p>
            ) : (
              <>
                <p className="text-xs text-muted-foreground mb-2">
                  Select which AI tools should receive this rule
                </p>
                <div className="space-y-1">
                  {tools.map((adapter) => {
                    const isEnabled = enabledAdapters.includes(adapter.id);
                    const adapterPath = getAdapterPath(adapter.id);
                    const lastSep = Math.max(
                      adapterPath.lastIndexOf("/"),
                      adapterPath.lastIndexOf("\\")
                    );
                    const displayPath =
                      lastSep >= 0 ? adapterPath.substring(lastSep + 1) : adapterPath;

                    return (
                      <div
                        key={adapter.id}
                        className="flex items-center justify-between p-2 rounded-md hover:bg-white/5 transition-colors"
                      >
                        <div
                          className="flex items-center gap-2 flex-1 cursor-pointer"
                          onClick={() => onToggleAdapter(adapter.id)}
                        >
                          <Switch
                            checked={isEnabled}
                            onCheckedChange={() => onToggleAdapter(adapter.id)}
                            aria-label={`Toggle ${adapter.name} adapter`}
                          />
                          <div className="min-w-0">
                            <div className="text-sm font-medium">{adapter.name}</div>
                            {isEnabled && adapterPath && (
                              <div className="text-[10px] text-muted-foreground/60 truncate">
                                {displayPath}
                              </div>
                            )}
                          </div>
                        </div>
                        {isEnabled && adapterPath && (
                          <Button
                            variant="ghost"
                            size="icon"
                            className="h-6 w-6 shrink-0"
                            onClick={(e) => {
                              e.stopPropagation();
                              onOpenFolder(adapter.id);
                            }}
                            title="Open in Explorer"
                          >
                            <ExternalLink className="h-3 w-3" />
                          </Button>
                        )}
                      </div>
                    );
                  })}
                </div>
              </>
            )}
          </CollapsibleContent>
        </Collapsible>
      </CardContent>
    </Card>
  );
}
