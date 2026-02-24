import { MoreVertical, Copy, Pencil, Trash2, Globe, FolderOpen } from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Switch } from "@/components/ui/switch";
import { Checkbox } from "@/components/ui/checkbox";
import { ADAPTERS, type Rule, type AdapterType } from "@/types/rule";
import type { RefObject } from "react";

interface RuleCardProps {
  rule: Rule;
  isSelected: boolean;
  menuOpenId: string | null;
  menuRef: RefObject<HTMLDivElement | null>;
  onSelect: (checked: boolean) => void;
  onToggle: () => void;
  onEdit: () => void;
  onDuplicate: () => void;
  onDelete: () => void;
  onToggleMenu: () => void;
}

function getAdapterBadge(adapterId: AdapterType): string {
  const adapter = ADAPTERS.find((a) => a.id === adapterId);
  return adapter?.name || adapterId;
}

export function RuleCard({
  rule,
  isSelected,
  menuOpenId,
  menuRef,
  onSelect,
  onToggle,
  onEdit,
  onDuplicate,
  onDelete,
  onToggleMenu,
}: RuleCardProps) {
  return (
    <Card
      className={cn(
        "group relative overflow-hidden transition-all duration-300",
        "glass-card border-white/5 hover:bg-white/10 hover:translate-x-1 premium-shadow",
        isSelected ? "ring-2 ring-primary bg-primary/5" : "hover:border-primary/20"
      )}
    >
      <CardContent className="flex items-center gap-4 p-4">
        <div onClick={(e) => e.stopPropagation()}>
          <Checkbox checked={isSelected} onChange={onSelect} aria-label={`Select ${rule.name}`} />
        </div>

        <div onClick={(e) => e.stopPropagation()} role="group" aria-label={`Toggle ${rule.name}`}>
          <Switch
            checked={rule.enabled}
            onCheckedChange={onToggle}
            aria-label={`${rule.enabled ? "Disable" : "Enable"} ${rule.name}`}
          />
        </div>

        <button
          className="flex-1 min-w-0 text-left focus:outline-none"
          onClick={onEdit}
          aria-label={`Edit rule: ${rule.name}`}
        >
          <div className="flex items-center gap-2">
            <span className="font-medium truncate">{rule.name}</span>
            <Badge
              variant={rule.scope === "global" ? "default" : "secondary"}
              aria-label={`${rule.scope} scope`}
            >
              {rule.scope === "global" ? (
                <Globe className="mr-1 h-3 w-3" aria-hidden="true" />
              ) : (
                <FolderOpen className="mr-1 h-3 w-3" aria-hidden="true" />
              )}
              {rule.scope === "global" ? "Global" : "Local"}
            </Badge>
          </div>
          <p className="text-sm text-muted-foreground truncate">
            {rule.content.substring(0, 100)}
            {rule.content.length > 100 && "..."}
          </p>
          <div className="flex items-center gap-1 mt-2" aria-label="Adapters">
            {rule.enabledAdapters.map((adapter) => (
              <Badge key={adapter} variant="outline" className="text-xs">
                {getAdapterBadge(adapter)}
              </Badge>
            ))}
          </div>
        </button>

        <div className="relative" ref={menuOpenId === rule.id ? menuRef : undefined}>
          <Button
            variant="ghost"
            size="icon"
            onClick={(e) => {
              e.stopPropagation();
              onToggleMenu();
            }}
            aria-label={`Actions for ${rule.name}`}
            aria-expanded={menuOpenId === rule.id}
            aria-haspopup="menu"
          >
            <MoreVertical className="h-4 w-4" aria-hidden="true" />
          </Button>
          {menuOpenId === rule.id && (
            <div
              className="absolute right-0 top-full mt-1 z-10 w-40 rounded-md border bg-background shadow-lg"
              role="menu"
              aria-label={`Actions for ${rule.name}`}
            >
              <button
                className="flex w-full items-center gap-2 px-3 py-2 text-sm hover:bg-accent focus:outline-none focus:bg-accent"
                onClick={() => {
                  onEdit();
                  onToggleMenu();
                }}
                role="menuitem"
              >
                <Pencil className="h-4 w-4" aria-hidden="true" />
                Edit
              </button>
              <button
                className="flex w-full items-center gap-2 px-3 py-2 text-sm hover:bg-accent focus:outline-none focus:bg-accent"
                onClick={() => {
                  onDuplicate();
                  onToggleMenu();
                }}
                role="menuitem"
              >
                <Copy className="h-4 w-4" aria-hidden="true" />
                Duplicate
              </button>
              <button
                className="flex w-full items-center gap-2 px-3 py-2 text-sm text-destructive hover:bg-accent focus:outline-none focus:bg-accent"
                onClick={() => {
                  onDelete();
                  onToggleMenu();
                }}
                role="menuitem"
              >
                <Trash2 className="h-4 w-4" aria-hidden="true" />
                Delete
              </button>
            </div>
          )}
        </div>
      </CardContent>
    </Card>
  );
}
