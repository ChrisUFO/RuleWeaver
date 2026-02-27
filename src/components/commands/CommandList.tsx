import { Plus, Search, FolderUp, Copy } from "lucide-react";
import { CommandTemplateBrowser } from "./CommandTemplateBrowser";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import type { CommandModel, McpStatus } from "@/types/command";
import { WatchingIndicator } from "@/components/ui/WatchingIndicator";

interface CommandListProps {
  commands: readonly CommandModel[];
  selectedId: string;
  query: string;
  isSaving: boolean;
  isSyncing: boolean;
  mcpStatus: McpStatus | null;
  mcpJustRefreshed?: boolean;
  onSelect: (id: string) => void;
  onDuplicate: (cmd: CommandModel) => void;
  onQueryChange: (q: string) => void;
  onCreate: () => void;
  onSync: () => void;
  onImport: () => void;
}

export function CommandList({
  commands,
  selectedId,
  query,
  isSaving,
  isSyncing,
  mcpStatus,
  mcpJustRefreshed,
  onSelect,
  onDuplicate,
  onQueryChange,
  onCreate,
  onSync,
  onImport,
}: CommandListProps) {
  return (
    <Card className="glass-card premium-shadow border-none overflow-hidden">
      <CardHeader className="space-y-4 bg-white/5 pb-6">
        <div className="flex items-center justify-between">
          <CardTitle className="text-sm font-semibold tracking-wide uppercase text-muted-foreground/80">
            Commands
          </CardTitle>
          <div className="flex items-center gap-2">
            <CommandTemplateBrowser onInstalled={onCreate} />
            <Button size="sm" onClick={onCreate} disabled={isSaving} className="glow-primary h-8">
              <Plus className="mr-1.5 h-3.5 w-3.5" />
              New
            </Button>
            <Button size="sm" variant="outline" onClick={onImport} className="glass h-8">
              <FolderUp className="mr-1.5 h-3.5 w-3.5" />
              Import
            </Button>
          </div>
        </div>
        <Button
          variant="outline"
          size="sm"
          onClick={onSync}
          disabled={isSyncing}
          className="w-full glass border-white/5 hover:bg-white/5 text-xs"
        >
          {isSyncing ? "Syncing..." : "Sync Command Files"}
        </Button>
        <div className="relative">
          <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground/60" />
          <Input
            value={query}
            onChange={(e) => onQueryChange(e.target.value)}
            placeholder="Filter..."
            className="pl-9 h-9 bg-black/20 border-white/5 focus-visible:ring-primary/40 rounded-lg text-sm"
          />
        </div>
      </CardHeader>
      <CardContent className="space-y-2 pt-4 px-2">
        {commands.map((cmd) => (
          <div
            key={cmd.id}
            role="button"
            tabIndex={0}
            className={cn(
              "w-full group relative overflow-hidden flex flex-col items-start rounded-xl px-4 py-3 text-left transition-all duration-300 border cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/40",
              selectedId === cmd.id
                ? "bg-primary/10 border-primary/20 premium-shadow"
                : "hover:bg-white/5 border-transparent hover:border-white/5"
            )}
            onClick={() => onSelect(cmd.id)}
            onKeyDown={(e) => {
              if (e.key === "Enter" || e.key === " ") {
                e.preventDefault();
                onSelect(cmd.id);
              }
            }}
          >
            <div className="flex w-full items-center justify-between gap-2">
              <div
                className={cn(
                  "truncate font-semibold text-sm transition-colors",
                  selectedId === cmd.id
                    ? "text-primary"
                    : "text-foreground group-hover:text-primary/80"
                )}
              >
                {cmd.name}
              </div>
              <div className="flex items-center gap-2">
                <Button
                  variant="ghost"
                  size="icon"
                  className="h-7 w-7 opacity-0 group-hover:opacity-100 transition-opacity hover:bg-primary/20"
                  onClick={(e) => {
                    e.stopPropagation();
                    onDuplicate(cmd);
                  }}
                  title="Duplicate Command (Ctrl+D)"
                >
                  <Copy className="h-3.5 w-3.5" />
                </Button>
                {mcpStatus?.running &&
                  mcpStatus.isWatching &&
                  (cmd.targetPaths?.length || 0) > 0 && (
                    <WatchingIndicator paths={cmd.targetPaths} justRefreshed={!!mcpJustRefreshed} />
                  )}{" "}
                {cmd.exposeViaMcp ? (
                  <Badge
                    variant="default"
                    className="h-4 text-[9px] px-1.5 uppercase font-bold tracking-tighter bg-primary/20 text-primary border-primary/20"
                  >
                    MCP
                  </Badge>
                ) : (
                  <Badge
                    variant="outline"
                    className="h-4 text-[9px] px-1.5 uppercase font-bold tracking-tighter border-white/10 text-muted-foreground/60"
                  >
                    Local
                  </Badge>
                )}
              </div>
            </div>
            <div className="mt-1 truncate text-[11px] text-muted-foreground/60 group-hover:text-muted-foreground/80">
              {cmd.description}
            </div>
          </div>
        ))}
        {commands.length === 0 && (
          <p className="text-xs text-muted-foreground/60 text-center py-8">No commands found.</p>
        )}
      </CardContent>
    </Card>
  );
}
