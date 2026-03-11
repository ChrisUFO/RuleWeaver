import { useMemo, useState } from "react";
import {
  AlertTriangle,
  CheckCircle2,
  Copy,
  Eye,
  Info,
  RefreshCw,
  Server,
  ShieldAlert,
  TerminalSquare,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Switch } from "@/components/ui/switch";
import { Badge } from "@/components/ui/badge";
import type { McpConnectionInstructions, McpDiagnostic, McpStatus } from "@/types/command";

interface McpSettingsCardProps {
  mcpStatus: McpStatus | null;
  mcpInstructions: McpConnectionInstructions | null;
  mcpLogs: readonly string[];
  isMcpLoading: boolean;
  mcpAutoStart: boolean;
  minimizeToTray: boolean;
  launchOnStartup: boolean;
  onStart: () => Promise<void>;
  onStop: () => Promise<void>;
  onRefresh: () => Promise<void>;
  onToggleAutoStart: (enabled: boolean) => Promise<void>;
  onToggleMinimizeToTray: (enabled: boolean) => Promise<void>;
  onToggleLaunchOnStartup: (enabled: boolean) => Promise<void>;
}

export function McpSettingsCard({
  mcpStatus,
  mcpInstructions,
  mcpLogs,
  isMcpLoading,
  mcpAutoStart,
  minimizeToTray,
  launchOnStartup,
  onStart,
  onStop,
  onRefresh,
  onToggleAutoStart,
  onToggleMinimizeToTray,
  onToggleLaunchOnStartup,
}: McpSettingsCardProps) {
  const [copiedLabel, setCopiedLabel] = useState<string | null>(null);

  const statusTone = useMemo(() => {
    switch (mcpStatus?.healthState) {
      case "ready":
        return {
          icon: CheckCircle2,
          iconClassName: "bg-primary/10 text-primary",
          badgeClassName: "glow-active border-primary/20",
          badgeLabel: "Ready",
        };
      case "degraded":
        return {
          icon: AlertTriangle,
          iconClassName: "bg-amber-500/10 text-amber-500",
          badgeClassName: "border-amber-500/20 bg-amber-500/10 text-amber-500",
          badgeLabel: "Degraded",
        };
      case "error":
        return {
          icon: ShieldAlert,
          iconClassName: "bg-destructive/10 text-destructive",
          badgeClassName: "border-destructive/20 bg-destructive/10 text-destructive",
          badgeLabel: "Error",
        };
      case "starting":
        return {
          icon: RefreshCw,
          iconClassName: "bg-blue-500/10 text-blue-500",
          badgeClassName: "border-blue-500/20 bg-blue-500/10 text-blue-500",
          badgeLabel: "Starting",
        };
      default:
        return {
          icon: Server,
          iconClassName: "bg-muted text-muted-foreground",
          badgeClassName: "",
          badgeLabel: "Stopped",
        };
    }
  }, [mcpStatus?.healthState]);

  const endpointUrl = mcpInstructions?.endpointUrl ?? mcpStatus?.endpointUrl ?? "";
  const apiToken = mcpInstructions?.apiToken ?? mcpStatus?.apiToken ?? "";

  const handleCopy = async (label: string, value: string) => {
    if (!value || !navigator.clipboard?.writeText) {
      return;
    }

    await navigator.clipboard.writeText(value);
    setCopiedLabel(label);
    window.setTimeout(
      () => setCopiedLabel((current) => (current === label ? null : current)),
      1500
    );
  };

  const diagnosticTone = (diagnostic: McpDiagnostic) => {
    switch (diagnostic.severity) {
      case "error":
        return {
          icon: ShieldAlert,
          className: "border-destructive/20 bg-destructive/5",
          iconClassName: "text-destructive",
        };
      case "warning":
        return {
          icon: AlertTriangle,
          className: "border-amber-500/20 bg-amber-500/5",
          iconClassName: "text-amber-500",
        };
      default:
        return {
          icon: Info,
          className: "border-blue-500/20 bg-blue-500/5",
          iconClassName: "text-blue-500",
        };
    }
  };

  const StatusIcon = statusTone.icon;

  return (
    <Card className="glass-card premium-shadow border-none overflow-hidden">
      <CardHeader className="bg-white/5 pb-4">
        <CardTitle className="text-sm font-semibold tracking-wide uppercase text-muted-foreground/80">
          MCP Server
        </CardTitle>
        <CardDescription>
          Start and manage the standalone local MCP server, then copy a working client config from
          this screen.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4 pt-6">
        <div className="flex items-center justify-between rounded-xl border border-white/5 bg-white/5 p-4">
          <div className="flex items-center gap-3">
            <div className={cn("rounded-lg p-2", statusTone.iconClassName)}>
              <StatusIcon
                className={cn("h-4 w-4", mcpStatus?.healthState === "starting" && "animate-spin")}
              />
            </div>
            <div>
              <div className="font-semibold text-sm">Status</div>
              <div className="text-xs text-muted-foreground">
                {mcpStatus?.statusMessage ?? "MCP server is stopped"}
              </div>
            </div>
          </div>
          <div className="flex items-center gap-2">
            {mcpStatus?.running && mcpStatus.isWatching && (
              <Badge
                variant="outline"
                className="bg-blue-500/10 text-blue-500 border-blue-500/20 gap-1 animate-pulse"
              >
                <Eye className="h-3 w-3" />
                Watching
              </Badge>
            )}
            <Badge
              variant={mcpStatus?.running ? "default" : "outline"}
              className={statusTone.badgeClassName}
            >
              {statusTone.badgeLabel}
            </Badge>
          </div>
        </div>

        <div className="grid gap-2 md:grid-cols-3">
          <div className="rounded-md border p-3">
            <div className="text-xs uppercase tracking-wide text-muted-foreground">Endpoint</div>
            <div className="mt-1 text-sm font-medium">
              {endpointUrl || "Unavailable until loaded"}
            </div>
          </div>
          <div className="rounded-md border p-3">
            <div className="text-xs uppercase tracking-wide text-muted-foreground">
              Exposed tools
            </div>
            <div className="mt-1 text-sm font-medium">
              {mcpStatus
                ? `${mcpStatus.availableCommands} commands • ${mcpStatus.availableSkills} skills`
                : "Loading"}
            </div>
          </div>
          <div className="rounded-md border p-3">
            <div className="text-xs uppercase tracking-wide text-muted-foreground">Uptime</div>
            <div className="mt-1 text-sm font-medium">
              {mcpStatus?.running ? `${mcpStatus.uptimeSeconds}s` : "Not running"}
            </div>
          </div>
        </div>

        <div className="flex flex-wrap gap-2">
          <Button
            onClick={onStart}
            disabled={isMcpLoading || mcpStatus?.healthState === "starting" || !!mcpStatus?.running}
            className="glow-primary"
          >
            Start Server
          </Button>
          <Button
            variant="outline"
            onClick={onStop}
            disabled={isMcpLoading || !mcpStatus?.running}
            className="glass border-white/5"
          >
            Stop
          </Button>
          <Button
            variant="ghost"
            onClick={onRefresh}
            disabled={isMcpLoading}
            className="text-muted-foreground"
          >
            <RefreshCw className={cn("mr-2 h-4 w-4", isMcpLoading && "animate-spin")} />
            Refresh
          </Button>
        </div>

        <div className="rounded-md border p-4">
          <div className="mb-2 flex items-center gap-2 text-sm font-medium">
            <TerminalSquare className="h-4 w-4 text-primary" />
            Standalone onboarding
          </div>
          <ol className="space-y-2 text-sm text-muted-foreground">
            <li>1. Start the MCP server from this card.</li>
            <li>2. Copy the endpoint, token, or JSON snippet below.</li>
            <li>3. Paste the config into Claude Code or OpenCode.</li>
            <li>
              4. Fully restart the MCP client after config changes to avoid stale protocol state.
            </li>
          </ol>
        </div>

        {!!mcpStatus?.diagnostics.length && (
          <div className="space-y-2">
            <div className="text-sm font-medium">Diagnostics</div>
            {mcpStatus.diagnostics.map((diagnostic) => {
              const tone = diagnosticTone(diagnostic);
              const DiagnosticIcon = tone.icon;

              return (
                <div key={diagnostic.code} className={cn("rounded-md border p-3", tone.className)}>
                  <div className="flex gap-3">
                    <DiagnosticIcon className={cn("mt-0.5 h-4 w-4 shrink-0", tone.iconClassName)} />
                    <div className="space-y-1">
                      <div className="text-sm font-medium">{diagnostic.title}</div>
                      <div className="text-sm text-muted-foreground">{diagnostic.message}</div>
                      {diagnostic.hint && (
                        <div className="text-xs text-muted-foreground">{diagnostic.hint}</div>
                      )}
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        )}

        <div className="flex items-center justify-between rounded-md border p-3">
          <div>
            <div className="font-medium">Auto-start MCP</div>
            <div className="text-xs text-muted-foreground">
              Start MCP automatically when RuleWeaver launches
            </div>
          </div>
          <Switch checked={mcpAutoStart} onCheckedChange={onToggleAutoStart} />
        </div>

        <div className="flex items-center justify-between rounded-md border p-3">
          <div>
            <div className="font-medium">Minimize to tray on close</div>
            <div className="text-xs text-muted-foreground">
              Keep app and MCP running when closing the main window
            </div>
          </div>
          <Switch checked={minimizeToTray} onCheckedChange={onToggleMinimizeToTray} />
        </div>

        <div className="flex items-center justify-between rounded-xl border border-white/5 bg-white/5 p-4 transition-colors hover:bg-white/10">
          <div>
            <div className="font-medium text-sm">Launch on startup</div>
            <div className="text-[10px] uppercase tracking-wider text-muted-foreground/60">
              Automatically start RuleWeaver when you log in
            </div>
          </div>
          <Switch checked={launchOnStartup} onCheckedChange={onToggleLaunchOnStartup} />
        </div>

        {(mcpInstructions || endpointUrl || apiToken) && (
          <div className="space-y-2">
            <div className="text-sm font-medium">Connection details</div>
            <div className="grid gap-2 md:grid-cols-2">
              <div className="rounded-md border p-3">
                <div className="mb-2 flex items-center justify-between gap-2">
                  <div className="text-xs uppercase tracking-wide text-muted-foreground">
                    Endpoint URL
                  </div>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => handleCopy("endpoint", endpointUrl)}
                  >
                    <Copy className="mr-2 h-3.5 w-3.5" />
                    Copy endpoint
                  </Button>
                </div>
                <code className="block overflow-auto rounded-md bg-muted p-2 text-xs">
                  {endpointUrl}
                </code>
              </div>
              <div className="rounded-md border p-3">
                <div className="mb-2 flex items-center justify-between gap-2">
                  <div className="text-xs uppercase tracking-wide text-muted-foreground">
                    API token
                  </div>
                  <Button variant="ghost" size="sm" onClick={() => handleCopy("token", apiToken)}>
                    <Copy className="mr-2 h-3.5 w-3.5" />
                    Copy token
                  </Button>
                </div>
                <code className="block overflow-auto rounded-md bg-muted p-2 text-xs">
                  {apiToken || "Unavailable"}
                </code>
              </div>
            </div>

            {mcpInstructions && (
              <>
                <div className="rounded-md border p-3">
                  <div className="mb-2 flex items-center justify-between gap-2">
                    <div className="text-xs uppercase tracking-wide text-muted-foreground">
                      Standalone command
                    </div>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() =>
                        handleCopy("standalone command", mcpInstructions.standaloneCommand)
                      }
                    >
                      <Copy className="mr-2 h-3.5 w-3.5" />
                      Copy command
                    </Button>
                  </div>
                  <code className="block overflow-auto rounded-md bg-muted p-2 text-xs">
                    {mcpInstructions.standaloneCommand}
                  </code>
                </div>

                <div className="grid gap-2 md:grid-cols-2">
                  <div className="rounded-md border p-3">
                    <div className="mb-2 flex items-center justify-between gap-2">
                      <div className="text-xs uppercase tracking-wide text-muted-foreground">
                        Claude Code JSON
                      </div>
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() =>
                          handleCopy("claude code json", mcpInstructions.claudeCodeJson)
                        }
                      >
                        <Copy className="mr-2 h-3.5 w-3.5" />
                        Copy JSON
                      </Button>
                    </div>
                    <code className="block overflow-auto rounded-md bg-muted p-2 text-xs">
                      {mcpInstructions.claudeCodeJson}
                    </code>
                  </div>
                  <div className="rounded-md border p-3">
                    <div className="mb-2 flex items-center justify-between gap-2">
                      <div className="text-xs uppercase tracking-wide text-muted-foreground">
                        OpenCode JSON
                      </div>
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => handleCopy("opencode json", mcpInstructions.opencodeJson)}
                      >
                        <Copy className="mr-2 h-3.5 w-3.5" />
                        Copy JSON
                      </Button>
                    </div>
                    <code className="block overflow-auto rounded-md bg-muted p-2 text-xs">
                      {mcpInstructions.opencodeJson}
                    </code>
                  </div>
                </div>
              </>
            )}

            {copiedLabel && (
              <div className="text-xs text-muted-foreground">
                Copied {copiedLabel} to clipboard.
              </div>
            )}
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
  );
}
