import { Server, RefreshCw } from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Switch } from "@/components/ui/switch";
import { Badge } from "@/components/ui/badge";

interface McpSettingsCardProps {
  mcpStatus: { running: boolean; port: number; uptime_seconds: number } | null;
  mcpInstructions: {
    claude_code_json: string;
    opencode_json: string;
    standalone_command: string;
  } | null;
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
  return (
    <Card className="glass-card premium-shadow border-none overflow-hidden">
      <CardHeader className="bg-white/5 pb-4">
        <CardTitle className="text-sm font-semibold tracking-wide uppercase text-muted-foreground/80">
          MCP Server
        </CardTitle>
        <CardDescription>
          Start and manage the local MCP server for tool integration
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4 pt-6">
        <div className="flex items-center justify-between rounded-xl border border-white/5 bg-white/5 p-4">
          <div className="flex items-center gap-3">
            <div
              className={cn(
                "p-2 rounded-lg",
                mcpStatus?.running ? "bg-primary/10 text-primary" : "bg-muted text-muted-foreground"
              )}
            >
              <Server className="h-4 w-4" />
            </div>
            <div>
              <div className="font-semibold text-sm">Status</div>
              <div className="text-xs text-muted-foreground">
                {mcpStatus?.running ? `Running on port ${mcpStatus.port}` : "Stopped"}
              </div>
            </div>
          </div>
          <Badge
            variant={mcpStatus?.running ? "default" : "outline"}
            className={cn(mcpStatus?.running && "glow-active border-primary/20")}
          >
            {mcpStatus?.running ? "Running" : "Stopped"}
          </Badge>
        </div>

        {mcpStatus?.running && (
          <p className="text-xs text-muted-foreground">Uptime: {mcpStatus.uptime_seconds}s</p>
        )}

        <div className="flex flex-wrap gap-2">
          <Button
            onClick={onStart}
            disabled={isMcpLoading || !!mcpStatus?.running}
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

        {mcpInstructions && (
          <div className="space-y-2">
            <code className="block rounded-md bg-muted p-2 text-xs overflow-auto">
              {mcpInstructions.standalone_command}
            </code>
            <div className="grid gap-2 md:grid-cols-2">
              <code className="rounded-md bg-muted p-2 text-xs overflow-auto">
                {mcpInstructions.claude_code_json}
              </code>
              <code className="rounded-md bg-muted p-2 text-xs overflow-auto">
                {mcpInstructions.opencode_json}
              </code>
            </div>
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
