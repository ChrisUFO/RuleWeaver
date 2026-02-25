import { Search, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Select } from "@/components/ui/select";
import { cn } from "@/lib/utils";
import { SORT_OPTIONS } from "./filter-utils";
import { useRegistryStore } from "@/stores/registryStore";

export interface RulesFilterBarProps {
  searchQuery: string;
  onSearchChange: (value: string) => void;
  scopeFilter: "all" | "global" | "local";
  onScopeChange: (value: "all" | "global" | "local") => void;
  adapterFilter: string;
  onAdapterChange: (value: string) => void;
  sortValue: string;
  onSortChange: (value: string) => void;
  onClear: () => void;
}

export function RulesFilterBar({
  searchQuery,
  onSearchChange,
  scopeFilter,
  onScopeChange,
  adapterFilter,
  onAdapterChange,
  sortValue,
  onSortChange,
  onClear,
}: RulesFilterBarProps) {
  const hasActiveFilters = searchQuery || scopeFilter !== "all" || adapterFilter !== "all";
  const { tools } = useRegistryStore();
  const adapterOptions = [
    { value: "all", label: "All Adapters" },
    ...tools.map((a) => ({ value: a.id, label: a.name })),
  ];

  return (
    <div className="flex flex-wrap items-center gap-3 p-4 glass rounded-xl border border-white/5 premium-shadow">
      <div className="relative flex-1 min-w-[200px] max-w-md">
        <Search
          className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground/60"
          aria-hidden="true"
        />
        <Input
          placeholder="Search rules..."
          value={searchQuery}
          onChange={(e) => onSearchChange(e.target.value)}
          className="pl-9 bg-white/5 border-white/5 focus-visible:ring-primary/40 rounded-lg"
          aria-label="Search rules"
        />
      </div>

      <Select
        value={sortValue}
        onChange={onSortChange}
        options={SORT_OPTIONS}
        className="w-44 bg-white/5 border-white/5 rounded-lg"
        aria-label="Sort rules"
      />

      <Select
        value={adapterFilter}
        onChange={onAdapterChange}
        options={adapterOptions}
        className="w-40 bg-white/5 border-white/5 rounded-lg"
        aria-label="Filter by adapter"
      />

      <div
        className="flex items-center gap-1.5 p-1 glass border border-white/5 rounded-lg"
        role="group"
        aria-label="Filter by scope"
      >
        <Button
          variant={scopeFilter === "all" ? "default" : "ghost"}
          size="sm"
          onClick={() => onScopeChange("all")}
          className={cn(
            "h-8 px-3 rounded-md transition-all",
            scopeFilter === "all" ? "glow-active shadow-sm" : "text-muted-foreground"
          )}
          aria-pressed={scopeFilter === "all"}
        >
          All
        </Button>
        <Button
          variant={scopeFilter === "global" ? "default" : "ghost"}
          size="sm"
          onClick={() => onScopeChange("global")}
          className={cn(
            "h-8 px-3 rounded-md transition-all",
            scopeFilter === "global" ? "glow-active shadow-sm" : "text-muted-foreground"
          )}
          aria-pressed={scopeFilter === "global"}
        >
          Global
        </Button>
        <Button
          variant={scopeFilter === "local" ? "default" : "ghost"}
          size="sm"
          onClick={() => onScopeChange("local")}
          className={cn(
            "h-8 px-3 rounded-md transition-all",
            scopeFilter === "local" ? "glow-active shadow-sm" : "text-muted-foreground"
          )}
          aria-pressed={scopeFilter === "local"}
        >
          Local
        </Button>
      </div>

      {hasActiveFilters && (
        <Button
          variant="ghost"
          size="sm"
          onClick={onClear}
          className="text-muted-foreground hover:text-foreground"
        >
          <X className="mr-1 h-3 w-3" />
          Clear
        </Button>
      )}
    </div>
  );
}
