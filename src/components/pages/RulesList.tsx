import { useEffect, useState, useMemo, useRef } from "react";
import { cn } from "@/lib/utils";
import {
  Plus,
  Search,
  MoreVertical,
  Copy,
  Pencil,
  Trash2,
  Globe,
  FolderOpen,
  X,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent } from "@/components/ui/card";
import { Switch } from "@/components/ui/switch";
import { Select } from "@/components/ui/select";
import { Checkbox } from "@/components/ui/checkbox";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import { useRulesStore } from "@/stores/rulesStore";
import { useToast } from "@/components/ui/toast";
import { RulesListSkeleton } from "@/components/ui/skeleton";
import { ADAPTERS, type Rule, type AdapterType } from "@/types/rule";

type SortField = "name" | "createdAt" | "updatedAt" | "enabled";
type SortDirection = "asc" | "desc";

interface RulesListProps {
  onSelectRule: (rule: Rule) => void;
  onCreateRule: () => void;
}

const SORT_OPTIONS = [
  { value: "name-asc", label: "Name (A-Z)" },
  { value: "name-desc", label: "Name (Z-A)" },
  { value: "createdAt-desc", label: "Newest First" },
  { value: "createdAt-asc", label: "Oldest First" },
  { value: "updatedAt-desc", label: "Recently Updated" },
  { value: "updatedAt-asc", label: "Least Recently Updated" },
  { value: "enabled-desc", label: "Enabled First" },
  { value: "enabled-asc", label: "Disabled First" },
];

const ADAPTER_FILTER_OPTIONS = [
  { value: "all", label: "All Adapters" },
  ...ADAPTERS.map((a) => ({ value: a.id, label: a.name })),
];

export function RulesList({ onSelectRule, onCreateRule }: RulesListProps) {
  const {
    rules,
    fetchRules,
    toggleRule,
    deleteRule,
    bulkDeleteRules,
    duplicateRule,
    restoreRecentlyDeleted,
    isLoading,
  } = useRulesStore();
  const { addToast } = useToast();
  const [searchQuery, setSearchQuery] = useState("");
  const [scopeFilter, setScopeFilter] = useState<"all" | "global" | "local">("all");
  const [adapterFilter, setAdapterFilter] = useState<string>("all");
  const [sortValue, setSortValue] = useState<string>("name-asc");
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [ruleToDelete, setRuleToDelete] = useState<Rule | null>(null);
  const [menuOpen, setMenuOpen] = useState<string | null>(null);
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [bulkDeleteDialogOpen, setBulkDeleteDialogOpen] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    fetchRules();
  }, [fetchRules]);

  // Handle click outside to close menu
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        setMenuOpen(null);
      }
    };

    if (menuOpen) {
      document.addEventListener("mousedown", handleClickOutside);
      return () => document.removeEventListener("mousedown", handleClickOutside);
    }
  }, [menuOpen]);

  const { sortField, sortDirection } = useMemo(() => {
    const [field, dir] = sortValue.split("-") as [SortField, SortDirection];
    return { sortField: field, sortDirection: dir };
  }, [sortValue]);

  const filteredAndSortedRules = useMemo(() => {
    const result = (rules || []).filter((rule) => {
      const matchesSearch =
        rule.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
        rule.content.toLowerCase().includes(searchQuery.toLowerCase());
      const matchesScope = scopeFilter === "all" || rule.scope === scopeFilter;
      const matchesAdapter =
        adapterFilter === "all" || rule.enabledAdapters.includes(adapterFilter as AdapterType);
      return matchesSearch && matchesScope && matchesAdapter;
    });

    result.sort((a, b) => {
      let comparison = 0;
      switch (sortField) {
        case "name":
          comparison = a.name.localeCompare(b.name);
          break;
        case "createdAt":
          comparison = a.createdAt - b.createdAt;
          break;
        case "updatedAt":
          comparison = a.updatedAt - b.updatedAt;
          break;
        case "enabled":
          comparison = (a.enabled ? 1 : 0) - (b.enabled ? 1 : 0);
          break;
      }
      return sortDirection === "asc" ? comparison : -comparison;
    });

    return result;
  }, [rules, searchQuery, scopeFilter, adapterFilter, sortField, sortDirection]);

  const allSelected =
    filteredAndSortedRules.length > 0 && filteredAndSortedRules.every((r) => selectedIds.has(r.id));
  const someSelected = selectedIds.size > 0;
  const indeterminate = someSelected && !allSelected;

  const handleSelectAll = (checked: boolean) => {
    if (checked) {
      setSelectedIds(new Set(filteredAndSortedRules.map((r) => r.id)));
    } else {
      setSelectedIds(new Set());
    }
  };

  const handleSelectOne = (id: string, checked: boolean) => {
    const newSet = new Set(selectedIds);
    if (checked) {
      newSet.add(id);
    } else {
      newSet.delete(id);
    }
    setSelectedIds(newSet);
  };

  const handleToggle = async (rule: Rule) => {
    await toggleRule(rule.id, !rule.enabled);
  };

  const handleDelete = async () => {
    if (!ruleToDelete) return;
    try {
      await deleteRule(ruleToDelete.id);
      addToast({
        title: "Rule Deleted",
        description: `"${ruleToDelete.name}" has been deleted`,
        variant: "success",
        duration: 8000,
        action: {
          label: "Undo",
          onClick: async () => {
            await restoreRecentlyDeleted();
            addToast({
              title: "Rule Restored",
              description: `"${ruleToDelete.name}" has been restored`,
              variant: "success",
            });
          },
        },
      });
    } catch (error) {
      addToast({
        title: "Delete Failed",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "error",
      });
    } finally {
      setDeleteDialogOpen(false);
      setRuleToDelete(null);
    }
  };

  const handleBulkDelete = async () => {
    const count = selectedIds.size;
    try {
      await bulkDeleteRules(Array.from(selectedIds));
      addToast({
        title: "Rules Deleted",
        description: `${count} rule${count !== 1 ? "s" : ""} deleted`,
        variant: "success",
      });
      setSelectedIds(new Set());
    } catch (error) {
      addToast({
        title: "Bulk Delete Failed",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "error",
      });
    } finally {
      setBulkDeleteDialogOpen(false);
    }
  };

  const handleBulkToggle = async (enabled: boolean) => {
    const count = selectedIds.size;
    try {
      await Promise.all(Array.from(selectedIds).map((id) => toggleRule(id, enabled)));
      addToast({
        title: "Rules Updated",
        description: `${count} rule${count !== 1 ? "s" : ""} ${enabled ? "enabled" : "disabled"}`,
        variant: "success",
      });
      setSelectedIds(new Set());
    } catch (error) {
      addToast({
        title: "Bulk Update Failed",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "error",
      });
    }
  };

  const confirmDelete = (rule: Rule) => {
    setRuleToDelete(rule);
    setDeleteDialogOpen(true);
    setMenuOpen(null);
  };

  const handleDuplicate = async (rule: Rule) => {
    try {
      await duplicateRule(rule);
      addToast({
        title: "Rule Duplicated",
        description: `"${rule.name}" has been duplicated`,
        variant: "success",
      });
    } catch (error) {
      addToast({
        title: "Duplicate Failed",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "error",
      });
    }
    setMenuOpen(null);
  };

  const getAdapterBadge = (adapterId: AdapterType) => {
    const adapter = ADAPTERS.find((a) => a.id === adapterId);
    return adapter?.name || adapterId;
  };

  const clearFilters = () => {
    setSearchQuery("");
    setScopeFilter("all");
    setAdapterFilter("all");
    setSortValue("name-asc");
  };

  const hasActiveFilters = searchQuery || scopeFilter !== "all" || adapterFilter !== "all";

  if (isLoading) {
    return <RulesListSkeleton />;
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold">Rules</h1>
        <Button onClick={onCreateRule} aria-label="Create new rule">
          <Plus className="mr-2 h-4 w-4" aria-hidden="true" />
          New Rule
        </Button>
      </div>

      <div className="flex flex-wrap items-center gap-3 p-4 glass rounded-xl border border-white/5 premium-shadow">
        <div className="relative flex-1 min-w-[200px] max-w-md">
          <Search
            className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground/60"
            aria-hidden="true"
          />
          <Input
            placeholder="Search rules..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="pl-9 bg-white/5 border-white/5 focus-visible:ring-primary/40 rounded-lg"
            aria-label="Search rules"
          />
        </div>

        <Select
          value={sortValue}
          onChange={setSortValue}
          options={SORT_OPTIONS}
          className="w-44 bg-white/5 border-white/5 rounded-lg"
          aria-label="Sort rules"
        />

        <Select
          value={adapterFilter}
          onChange={setAdapterFilter}
          options={ADAPTER_FILTER_OPTIONS}
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
            onClick={() => setScopeFilter("all")}
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
            onClick={() => setScopeFilter("global")}
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
            onClick={() => setScopeFilter("local")}
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
            onClick={clearFilters}
            className="text-muted-foreground hover:text-foreground"
          >
            <X className="mr-1 h-3 w-3" />
            Clear
          </Button>
        )}
      </div>

      {someSelected && (
        <div
          className="flex items-center gap-3 p-3 bg-accent/50 rounded-md border"
          role="toolbar"
          aria-label="Bulk actions"
        >
          <span className="text-sm text-muted-foreground">{selectedIds.size} selected</span>
          <Button variant="outline" size="sm" onClick={() => handleBulkToggle(true)}>
            Enable All
          </Button>
          <Button variant="outline" size="sm" onClick={() => handleBulkToggle(false)}>
            Disable All
          </Button>
          <Button variant="destructive" size="sm" onClick={() => setBulkDeleteDialogOpen(true)}>
            Delete All
          </Button>
          <Button variant="ghost" size="sm" onClick={() => setSelectedIds(new Set())}>
            Cancel
          </Button>
        </div>
      )}

      {filteredAndSortedRules.length === 0 ? (
        <Card className="border-dashed">
          <CardContent className="flex flex-col items-center justify-center py-12">
            <p className="text-muted-foreground" role="status">
              {hasActiveFilters
                ? "No rules match your filters"
                : "No rules yet. Create your first rule to get started."}
            </p>
          </CardContent>
        </Card>
      ) : (
        <ul className="space-y-3" role="list" aria-label="Rules list">
          {filteredAndSortedRules.map((rule) => (
            <li key={rule.id}>
              <Card
                className={cn(
                  "group relative overflow-hidden transition-all duration-300",
                  "glass-card border-white/5 hover:bg-white/10 hover:translate-x-1 premium-shadow",
                  selectedIds.has(rule.id)
                    ? "ring-2 ring-primary bg-primary/5"
                    : "hover:border-primary/20"
                )}
              >
                <CardContent className="flex items-center gap-4 p-4">
                  <div onClick={(e) => e.stopPropagation()}>
                    <Checkbox
                      checked={selectedIds.has(rule.id)}
                      onChange={(checked) => handleSelectOne(rule.id, checked)}
                      aria-label={`Select ${rule.name}`}
                    />
                  </div>

                  <div
                    onClick={(e) => e.stopPropagation()}
                    role="group"
                    aria-label={`Toggle ${rule.name}`}
                  >
                    <Switch
                      checked={rule.enabled}
                      onCheckedChange={() => handleToggle(rule)}
                      aria-label={`${rule.enabled ? "Disable" : "Enable"} ${rule.name}`}
                    />
                  </div>

                  <button
                    className="flex-1 min-w-0 text-left focus:outline-none"
                    onClick={() => onSelectRule(rule)}
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

                  <div className="relative" ref={menuOpen === rule.id ? menuRef : undefined}>
                    <Button
                      variant="ghost"
                      size="icon"
                      onClick={(e) => {
                        e.stopPropagation();
                        setMenuOpen(menuOpen === rule.id ? null : rule.id);
                      }}
                      aria-label={`Actions for ${rule.name}`}
                      aria-expanded={menuOpen === rule.id}
                      aria-haspopup="menu"
                    >
                      <MoreVertical className="h-4 w-4" aria-hidden="true" />
                    </Button>
                    {menuOpen === rule.id && (
                      <div
                        className="absolute right-0 top-full mt-1 z-10 w-40 rounded-md border bg-background shadow-lg"
                        role="menu"
                        aria-label={`Actions for ${rule.name}`}
                      >
                        <button
                          className="flex w-full items-center gap-2 px-3 py-2 text-sm hover:bg-accent focus:outline-none focus:bg-accent"
                          onClick={() => {
                            onSelectRule(rule);
                            setMenuOpen(null);
                          }}
                          role="menuitem"
                        >
                          <Pencil className="h-4 w-4" aria-hidden="true" />
                          Edit
                        </button>
                        <button
                          className="flex w-full items-center gap-2 px-3 py-2 text-sm hover:bg-accent focus:outline-none focus:bg-accent"
                          onClick={() => handleDuplicate(rule)}
                          role="menuitem"
                        >
                          <Copy className="h-4 w-4" aria-hidden="true" />
                          Duplicate
                        </button>
                        <button
                          className="flex w-full items-center gap-2 px-3 py-2 text-sm text-destructive hover:bg-accent focus:outline-none focus:bg-accent"
                          onClick={() => confirmDelete(rule)}
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
            </li>
          ))}
        </ul>
      )}

      {filteredAndSortedRules.length > 0 && (
        <div className="flex items-center gap-2 py-2">
          <Checkbox
            checked={allSelected}
            indeterminate={indeterminate}
            onChange={handleSelectAll}
            aria-label="Select all rules"
          />
          <span className="text-sm text-muted-foreground">
            {allSelected ? "All" : `${selectedIds.size} of ${filteredAndSortedRules.length}`}{" "}
            selected
          </span>
        </div>
      )}

      <Dialog open={deleteDialogOpen} onOpenChange={setDeleteDialogOpen}>
        <DialogContent onClose={() => setDeleteDialogOpen(false)}>
          <DialogHeader>
            <DialogTitle>Delete Rule</DialogTitle>
            <DialogDescription>
              Are you sure you want to delete "{ruleToDelete?.name}"? This action cannot be undone.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setDeleteDialogOpen(false)}>
              Cancel
            </Button>
            <Button
              variant="destructive"
              onClick={handleDelete}
              aria-label={`Delete ${ruleToDelete?.name}`}
            >
              Delete
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={bulkDeleteDialogOpen} onOpenChange={setBulkDeleteDialogOpen}>
        <DialogContent onClose={() => setBulkDeleteDialogOpen(false)}>
          <DialogHeader>
            <DialogTitle>Delete {selectedIds.size} Rules</DialogTitle>
            <DialogDescription>
              Are you sure you want to delete {selectedIds.size} rule
              {selectedIds.size !== 1 ? "s" : ""}? This action cannot be undone.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setBulkDeleteDialogOpen(false)}>
              Cancel
            </Button>
            <Button variant="destructive" onClick={handleBulkDelete}>
              Delete All
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
