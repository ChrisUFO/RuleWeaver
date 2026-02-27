import { useEffect, useState, useMemo, useRef } from "react";
import type { DragEvent } from "react";
import { cn } from "@/lib/utils";
import { Plus, Upload, FileText, FolderUp, Link, Clipboard } from "lucide-react";
import { ImportDialog, type ImportSourceMode } from "@/components/import/ImportDialog";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Checkbox } from "@/components/ui/checkbox";
import { useRulesStore } from "@/stores/rulesStore";
import { useToast } from "@/components/ui/toast";
import { RulesListSkeleton } from "@/components/ui/skeleton";
import { RulesFilterBar } from "@/components/rules/RulesFilterBar";
import { parseSortValue } from "@/components/rules/filter-utils";
import { RuleCard } from "@/components/rules/RuleCard";
import { toast } from "@/lib/toast-helpers";
import { type Rule, type AdapterType } from "@/types/rule";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { RuleTemplateBrowser } from "@/components/rules/RuleTemplateBrowser";
import { useKeyboardShortcuts, SHORTCUTS } from "@/hooks/useKeyboardShortcuts";

interface RulesListProps {
  onSelectRule: (rule: Rule) => void;
  onCreateRule: () => void;
}

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
  const [importDialogOpen, setImportDialogOpen] = useState(false);
  const [initialImportMode, setInitialImportMode] = useState<ImportSourceMode | null>(null);
  const [isDragImportActive, setIsDragImportActive] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);

  useKeyboardShortcuts({
    shortcuts: [
      {
        ...SHORTCUTS.DUPLICATE,
        action: () => {
          if (selectedIds.size === 1) {
            const id = Array.from(selectedIds)[0];
            const rule = rules.find((r) => r.id === id);
            if (rule) handleDuplicate(rule);
          }
        },
      },
    ],
  });

  useEffect(() => {
    fetchRules();
  }, [fetchRules]);

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

  const { sortField, sortDirection } = useMemo(() => parseSortValue(sortValue), [sortValue]);

  const filteredAndSortedRules = useMemo(() => {
    const result = (rules || []).filter((rule) => {
      const matchesSearch =
        rule.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
        rule.description.toLowerCase().includes(searchQuery.toLowerCase()) ||
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
      toast.error(addToast, { title: "Delete Failed", error });
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
      toast.error(addToast, { title: "Bulk Delete Failed", error });
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
      toast.error(addToast, { title: "Bulk Update Failed", error });
    }
  };

  const confirmDelete = (rule: Rule) => {
    setRuleToDelete(rule);
    setDeleteDialogOpen(true);
    setMenuOpen(null);
  };

  const handleDuplicate = async (rule: Rule) => {
    try {
      const newRule = await duplicateRule(rule);
      onSelectRule(newRule);
      addToast({
        title: "Rule Duplicated",
        description: `"${rule.name}" has been duplicated`,
        variant: "success",
      });
    } catch (error) {
      toast.error(addToast, { title: "Duplicate Failed", error });
    }
    setMenuOpen(null);
  };

  const clearFilters = () => {
    setSearchQuery("");
    setScopeFilter("all");
    setAdapterFilter("all");
    setSortValue("name-asc");
  };

  const openImport = (mode: ImportSourceMode) => {
    setInitialImportMode(mode);
    setImportDialogOpen(true);
  };

  const handleDragImportOver = (event: DragEvent<HTMLDivElement>) => {
    event.preventDefault();
    setIsDragImportActive(true);
  };

  const handleDragImportLeave = (event: DragEvent<HTMLDivElement>) => {
    event.preventDefault();
    setIsDragImportActive(false);
  };

  const handleDropImport = async (event: DragEvent<HTMLDivElement>) => {
    event.preventDefault();
    setIsDragImportActive(false);

    const droppedFiles = event.dataTransfer.files;
    if (!droppedFiles || droppedFiles.length === 0) {
      return;
    }

    const fileWithPath = droppedFiles[0] as File & { path?: string };
    const filePath = fileWithPath.path;
    if (!filePath) {
      addToast({
        title: "Drop Not Supported",
        description: "Use Import File if your platform does not expose drop file paths.",
        variant: "error",
      });
      return;
    }

    setInitialImportMode("file");
    setImportDialogOpen(true);
    // Note: The new ImportDialog doesn't currently auto-scan specific files passed from parent,
    // but the user can then just select the file again or we can enhance ImportDialog later.
    // For now, it's enough to open the dialog.
  };

  const hasActiveFilters = searchQuery || scopeFilter !== "all" || adapterFilter !== "all";

  if (isLoading) {
    return <RulesListSkeleton />;
  }

  return (
    <div className="space-y-6 max-w-7xl mx-auto">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <h1 className="text-2xl font-bold">Rules</h1>
        <div className="flex items-center gap-2">
          <Button variant="outline" onClick={() => openImport("ai")}>
            <Upload className="mr-2 h-4 w-4" aria-hidden="true" />
            Import AI
          </Button>
          <Button variant="outline" onClick={() => openImport("file")}>
            <FileText className="mr-2 h-4 w-4" aria-hidden="true" />
            Import File
          </Button>
          <Button variant="outline" onClick={() => openImport("directory")}>
            <FolderUp className="mr-2 h-4 w-4" aria-hidden="true" />
            Import Folder
          </Button>
          <Button variant="outline" onClick={() => openImport("url")}>
            <Link className="mr-2 h-4 w-4" aria-hidden="true" />
            Import URL
          </Button>
          <Button variant="outline" onClick={() => openImport("clipboard")}>
            <Clipboard className="mr-2 h-4 w-4" aria-hidden="true" />
            Import Clipboard
          </Button>
          <RuleTemplateBrowser onInstalled={fetchRules} />
          <Button onClick={onCreateRule} aria-label="Create new rule">
            <Plus className="mr-2 h-4 w-4" aria-hidden="true" />
            New Rule
          </Button>
        </div>
      </div>

      <div
        className={cn(
          "rounded-xl border border-dashed p-3 text-sm transition-colors",
          isDragImportActive
            ? "border-primary bg-primary/10 text-foreground"
            : "border-white/20 text-muted-foreground"
        )}
        onDragOver={handleDragImportOver}
        onDragLeave={handleDragImportLeave}
        onDrop={handleDropImport}
      >
        Drag and drop a rule file here to scan and import.
      </div>

      <RulesFilterBar
        searchQuery={searchQuery}
        onSearchChange={setSearchQuery}
        scopeFilter={scopeFilter}
        onScopeChange={setScopeFilter}
        adapterFilter={adapterFilter}
        onAdapterChange={setAdapterFilter}
        sortValue={sortValue}
        onSortChange={setSortValue}
        onClear={clearFilters}
      />

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
              <RuleCard
                rule={rule}
                isSelected={selectedIds.has(rule.id)}
                menuOpenId={menuOpen}
                menuRef={menuRef}
                onSelect={(checked) => handleSelectOne(rule.id, checked)}
                onToggle={() => handleToggle(rule)}
                onEdit={() => onSelectRule(rule)}
                onDuplicate={() => handleDuplicate(rule)}
                onDelete={() => confirmDelete(rule)}
                onToggleMenu={() => setMenuOpen(menuOpen === rule.id ? null : rule.id)}
              />
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

      <ImportDialog
        open={importDialogOpen}
        onOpenChange={setImportDialogOpen}
        artifactType="rule"
        initialSourceMode={initialImportMode}
        onImportComplete={async () => {
          await fetchRules();
        }}
      />
    </div>
  );
}
