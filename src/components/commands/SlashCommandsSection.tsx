import { cn } from "@/lib/utils";
import { Switch } from "@/components/ui/switch";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import type { AdapterInfo, SlashSyncStatus } from "@/hooks/useCommandsState";

interface TierConfig {
  tier: number;
  label: string;
  color: string;
}

interface SlashCommandsSectionProps {
  generateSlashCommands: boolean;
  slashCommandAdapters: readonly string[];
  availableAdapters: readonly AdapterInfo[];
  slashStatus: Record<string, SlashSyncStatus>;
  tierConfig: Record<string, TierConfig>;
  onToggleGenerate: (checked: boolean) => void;
  onToggleAdapter: (adapter: string) => void;
  onRepairAdapter: (adapter: string) => void;
}

function statusLabel(status: SlashSyncStatus): string {
  if (status === "Synced") return "Synced";
  if (status === "OutOfDate") return "Out of date";
  if (status === "NotSynced") return "Not synced";
  if (typeof status === "object" && "Error" in status) return "Error";
  return "";
}

function statusColor(status: SlashSyncStatus): string {
  if (status === "Synced")
    return "bg-green-500/20 text-green-400 border-green-500/30";
  if (status === "OutOfDate")
    return "bg-yellow-500/20 text-yellow-400 border-yellow-500/30";
  if (status === "NotSynced")
    return "bg-white/10 text-muted-foreground border-white/10";
  return "bg-red-500/20 text-red-400 border-red-500/30";
}

function needsRepair(status: SlashSyncStatus): boolean {
  return status === "OutOfDate" || (typeof status === "object" && "Error" in status);
}

export function SlashCommandsSection({
  generateSlashCommands,
  slashCommandAdapters,
  availableAdapters,
  slashStatus,
  tierConfig,
  onToggleGenerate,
  onToggleAdapter,
  onRepairAdapter,
}: SlashCommandsSectionProps) {
  return (
    <div className="rounded-xl border border-white/5 bg-white/5 p-4 space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <div className="font-semibold text-sm">Generate Slash Commands</div>
          <div className="text-[10px] uppercase tracking-wider text-muted-foreground/60">
            Create native /command triggers in AI tools.
          </div>
        </div>
        <Switch
          checked={generateSlashCommands}
          onCheckedChange={onToggleGenerate}
          aria-label="Generate slash commands"
        />
      </div>

      {generateSlashCommands && availableAdapters.length > 0 && (
        <div className="space-y-3 pt-2 border-t border-white/5">
          <div className="flex items-center justify-between">
            <div className="text-xs font-medium text-muted-foreground">Target AI Tools</div>
            <div className="text-[10px] text-muted-foreground/60">
              {slashCommandAdapters.length} selected
            </div>
          </div>
          <div className="flex flex-wrap gap-2">
            {availableAdapters.map((adapter) => {
              const isSelected = slashCommandAdapters.includes(adapter.name);
              const tierInfo = tierConfig[adapter.name] || {
                tier: 2,
                label: "Beta",
                color: "bg-yellow-500/20 text-yellow-400 border-yellow-500/30",
              };
              const syncStatus = isSelected ? slashStatus[adapter.name] : undefined;

              return (
                <div key={adapter.name} className="flex flex-col items-start gap-1">
                  <button
                    onClick={() => onToggleAdapter(adapter.name)}
                    title={`Will write to: ${adapter.name}/command.md`}
                    className={cn(
                      "group flex items-center gap-1.5 px-3 py-1.5 rounded-full text-xs font-medium transition-all duration-200",
                      isSelected
                        ? "bg-primary text-primary-foreground shadow-lg shadow-primary/20 scale-105"
                        : "bg-white/5 text-muted-foreground hover:bg-white/15 hover:scale-105 border border-transparent hover:border-white/10"
                    )}
                  >
                    <span>{adapter.name}</span>
                    {isSelected && (
                      <Badge
                        variant="outline"
                        className={cn("ml-1 text-[9px] px-1 py-0 h-4 border-0", tierInfo.color)}
                      >
                        {tierInfo.label}
                      </Badge>
                    )}
                  </button>

                  {isSelected && syncStatus !== undefined && (
                    <div className="flex items-center gap-1 pl-1">
                      <Badge
                        variant="outline"
                        className={cn("text-[9px] px-1.5 py-0 h-4", statusColor(syncStatus))}
                      >
                        {statusLabel(syncStatus)}
                      </Badge>
                      {needsRepair(syncStatus) && (
                        <Button
                          variant="ghost"
                          size="sm"
                          className="h-4 px-1.5 text-[9px] text-primary/70 hover:text-primary"
                          onClick={(e) => {
                            e.stopPropagation();
                            onRepairAdapter(adapter.name);
                          }}
                        >
                          Repair
                        </Button>
                      )}
                    </div>
                  )}
                </div>
              );
            })}
          </div>
          {slashCommandAdapters.length > 0 && (
            <div className="text-[10px] text-muted-foreground/80 bg-white/5 rounded-md p-2">
              Files will be created in each tool's commands directory
            </div>
          )}
        </div>
      )}
    </div>
  );
}
