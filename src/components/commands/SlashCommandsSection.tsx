import { cn } from "@/lib/utils";
import { Switch } from "@/components/ui/switch";
import { Badge } from "@/components/ui/badge";
import type { AdapterInfo } from "@/hooks/useCommandsState";

interface TierConfig {
  tier: number;
  label: string;
  color: string;
}

interface SlashCommandsSectionProps {
  generateSlashCommands: boolean;
  slashCommandAdapters: readonly string[];
  availableAdapters: readonly AdapterInfo[];
  tierConfig: Record<string, TierConfig>;
  onToggleGenerate: (checked: boolean) => void;
  onToggleAdapter: (adapter: string) => void;
}

export function SlashCommandsSection({
  generateSlashCommands,
  slashCommandAdapters,
  availableAdapters,
  tierConfig,
  onToggleGenerate,
  onToggleAdapter,
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

              return (
                <button
                  key={adapter.name}
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
