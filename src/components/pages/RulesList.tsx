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
  Upload,
  FolderUp,
  Link,
  Clipboard,
  FileText,
} from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";
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
import {
  ADAPTERS,
  type Rule,
  type AdapterType,
  type ImportCandidate,
  type ImportConflictMode,
  type ImportExecutionResult,
  type ImportHistoryEntry,
} from "@/types/rule";
import { api } from "@/lib/tauri";

type SortField = "name" | "createdAt" | "updatedAt" | "enabled";
type SortDirection = "asc" | "desc";

interface RulesListProps {
  onSelectRule: (rule: Rule) => void;
  onCreateRule: () => void;
}

type ImportSourceMode = "ai" | "file" | "directory" | "url" | "clipboard";

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
  const [isImporting, setIsImporting] = useState(false);
  const [isScanningImport, setIsScanningImport] = useState(false);
  const [importDialogOpen, setImportDialogOpen] = useState(false);
  const [importCandidates, setImportCandidates] = useState<ImportCandidate[]>([]);
  const [selectedImportIds, setSelectedImportIds] = useState<Set<string>>(new Set());
  const [importScanErrors, setImportScanErrors] = useState<string[]>([]);
  const [importConflictMode, setImportConflictMode] = useState<ImportConflictMode>("rename");
  const [importResult, setImportResult] = useState<ImportExecutionResult | null>(null);
  const [importHistory, setImportHistory] = useState<ImportHistoryEntry[]>([]);
  const [importSourceMode, setImportSourceMode] = useState<ImportSourceMode>("ai");
  const [importSourceValue, setImportSourceValue] = useState("");
  const [clipboardImportName, setClipboardImportName] = useState<string | undefined>(undefined);
  const [urlImportDialogOpen, setUrlImportDialogOpen] = useState(false);
  const [urlImportValue, setUrlImportValue] = useState("");
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

  const handleImportResult = async (title: string, result: ImportExecutionResult) => {
    await fetchRules();
    addToast({
      title,
      description: `${result.imported.length} imported, ${result.skipped.length} skipped, ${result.conflicts.length} conflicts`,
      variant: result.errors.length > 0 ? "error" : "success",
    });
    if (result.errors.length > 0) {
      addToast({
        title: "Import Warnings",
        description: result.errors[0],
        variant: "error",
      });
    }
    const history = await api.ruleImport.getHistory();
    setImportHistory(history);
  };

  const openImportPreview = async (
    mode: ImportSourceMode,
    sourceValue: string,
    candidates: ImportCandidate[],
    errors: string[]
  ) => {
    setImportSourceMode(mode);
    setImportSourceValue(sourceValue);
    if (mode !== "clipboard") {
      setClipboardImportName(undefined);
    }
    setImportResult(null);
    setImportCandidates(candidates);
    setImportScanErrors(errors);
    setSelectedImportIds(new Set(candidates.map((c) => c.id)));
    setImportDialogOpen(true);
    const history = await api.ruleImport.getHistory();
    setImportHistory(history);
  };

  const scanAiToolRules = async () => {
    setIsScanningImport(true);
    try {
      const scan = await api.ruleImport.scanAiToolCandidates();
      await openImportPreview("ai", "", scan.candidates, scan.errors);
    } catch (error) {
      addToast({
        title: "Scan Failed",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "error",
      });
    } finally {
      setIsScanningImport(false);
    }
  };

  const scanImportFromFile = async () => {
    const selected = await open({
      multiple: false,
      filters: [{ name: "Rule Files", extensions: ["md", "txt", "json", "yaml", "yml"] }],
    });
    if (!selected || Array.isArray(selected)) return;

    setIsScanningImport(true);
    try {
      const scan = await api.ruleImport.scanFromFile(selected);
      await openImportPreview("file", selected, scan.candidates, scan.errors);
    } catch (error) {
      addToast({
        title: "Scan Failed",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "error",
      });
    } finally {
      setIsScanningImport(false);
    }
  };

  const scanImportFromDirectory = async () => {
    const selected = await open({ directory: true, multiple: false });
    if (!selected || Array.isArray(selected)) return;

    setIsScanningImport(true);
    try {
      const scan = await api.ruleImport.scanFromDirectory(selected);
      await openImportPreview("directory", selected, scan.candidates, scan.errors);
    } catch (error) {
      addToast({
        title: "Scan Failed",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "error",
      });
    } finally {
      setIsScanningImport(false);
    }
  };

  const scanImportFromUrl = async (url: string) => {
    if (!url.trim()) return;

    setIsScanningImport(true);
    try {
      const scan = await api.ruleImport.scanFromUrl(url);
      await openImportPreview("url", url, scan.candidates, scan.errors);
    } catch (error) {
      addToast({
        title: "Scan Failed",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "error",
      });
    } finally {
      setIsScanningImport(false);
    }
  };

  const openUrlImportDialog = () => {
    setUrlImportDialogOpen(true);
  };

  const submitUrlImportScan = async () => {
    const value = urlImportValue.trim();
    if (!value) {
      addToast({
        title: "URL Required",
        description: "Enter a URL to scan for import",
        variant: "error",
      });
      return;
    }

    setUrlImportDialogOpen(false);
    await scanImportFromUrl(value);
  };

  const scanImportFromClipboard = async () => {
    setIsScanningImport(true);
    try {
      const text = await navigator.clipboard.readText();
      if (!text.trim()) {
        addToast({
          title: "Clipboard Empty",
          description: "No text found in clipboard",
          variant: "error",
        });
        return;
      }

      const name = window.prompt("Optional name for clipboard import:") || undefined;
      setClipboardImportName(name);
      const scan = await api.ruleImport.scanFromClipboard(text, name);
      await openImportPreview("clipboard", text, scan.candidates, scan.errors);
    } catch (error) {
      addToast({
        title: "Scan Failed",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "error",
      });
    } finally {
      setIsScanningImport(false);
    }
  };

  const toggleImportCandidate = (id: string, checked: boolean) => {
    setSelectedImportIds((prev) => {
      const next = new Set(prev);
      if (checked) {
        next.add(id);
      } else {
        next.delete(id);
      }
      return next;
    });
  };

  const toggleSelectAllImportCandidates = (checked: boolean) => {
    if (checked) {
      setSelectedImportIds(new Set(importCandidates.map((c) => c.id)));
    } else {
      setSelectedImportIds(new Set());
    }
  };

  const executeImport = async () => {
    if (selectedImportIds.size === 0) {
      addToast({
        title: "No Candidates Selected",
        description: "Select at least one candidate to import",
        variant: "error",
      });
      return;
    }

    setIsImporting(true);
    try {
      const options = {
        conflictMode: importConflictMode,
        selectedCandidateIds: Array.from(selectedImportIds),
      };

      let result: ImportExecutionResult;
      if (importSourceMode === "ai") {
        result = await api.ruleImport.importAiToolRules(options);
      } else if (importSourceMode === "file") {
        result = await api.ruleImport.importFromFile(importSourceValue, options);
      } else if (importSourceMode === "directory") {
        result = await api.ruleImport.importFromDirectory(importSourceValue, options);
      } else if (importSourceMode === "url") {
        result = await api.ruleImport.importFromUrl(importSourceValue, options);
      } else {
        result = await api.ruleImport.importFromClipboard(
          importSourceValue,
          clipboardImportName,
          options
        );
      }

      setImportResult(result);
      await handleImportResult("Import Complete", result);
      if (importSourceMode === "clipboard") {
        setClipboardImportName(undefined);
      }
    } catch (error) {
      addToast({
        title: "Import Failed",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "error",
      });
    } finally {
      setIsImporting(false);
    }
  };

  const hasActiveFilters = searchQuery || scopeFilter !== "all" || adapterFilter !== "all";

  if (isLoading) {
    return <RulesListSkeleton />;
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold">Rules</h1>
        <div className="flex items-center gap-2">
          <Button
            variant="outline"
            onClick={scanAiToolRules}
            disabled={isImporting || isScanningImport}
          >
            <Upload className="mr-2 h-4 w-4" aria-hidden="true" />
            {isScanningImport ? "Scanning..." : "Import AI"}
          </Button>
          <Button
            variant="outline"
            onClick={scanImportFromFile}
            disabled={isImporting || isScanningImport}
          >
            <FileText className="mr-2 h-4 w-4" aria-hidden="true" />
            Import File
          </Button>
          <Button
            variant="outline"
            onClick={scanImportFromDirectory}
            disabled={isImporting || isScanningImport}
          >
            <FolderUp className="mr-2 h-4 w-4" aria-hidden="true" />
            Import Folder
          </Button>
          <Button
            variant="outline"
            onClick={openUrlImportDialog}
            disabled={isImporting || isScanningImport}
          >
            <Link className="mr-2 h-4 w-4" aria-hidden="true" />
            Import URL
          </Button>
          <Button
            variant="outline"
            onClick={scanImportFromClipboard}
            disabled={isImporting || isScanningImport}
          >
            <Clipboard className="mr-2 h-4 w-4" aria-hidden="true" />
            Import Clipboard
          </Button>
          <Button onClick={onCreateRule} aria-label="Create new rule">
            <Plus className="mr-2 h-4 w-4" aria-hidden="true" />
            New Rule
          </Button>
        </div>
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

      <Dialog open={importDialogOpen} onOpenChange={setImportDialogOpen}>
        <DialogContent onClose={() => setImportDialogOpen(false)}>
          <DialogHeader>
            <DialogTitle>
              {importSourceMode === "ai"
                ? "Import Existing AI Tool Rules"
                : importSourceMode === "file"
                  ? "Import Rules From File"
                  : importSourceMode === "directory"
                    ? "Import Rules From Folder"
                    : importSourceMode === "url"
                      ? "Import Rules From URL"
                      : "Import Rules From Clipboard"}
            </DialogTitle>
            <DialogDescription>
              Review discovered candidates, choose conflict handling, and import selected rules.
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-3 max-h-[50vh] overflow-y-auto">
            {importSourceMode !== "ai" && importSourceValue && (
              <div className="rounded-md border p-3 text-xs text-muted-foreground break-all">
                Source: {importSourceMode === "clipboard" ? "Clipboard text" : importSourceValue}
              </div>
            )}

            <div className="flex items-center justify-between gap-2">
              <div className="flex items-center gap-2">
                <Checkbox
                  checked={
                    importCandidates.length > 0 &&
                    selectedImportIds.size === importCandidates.length
                  }
                  indeterminate={
                    selectedImportIds.size > 0 && selectedImportIds.size < importCandidates.length
                  }
                  onChange={toggleSelectAllImportCandidates}
                  aria-label="Select all import candidates"
                />
                <span className="text-sm text-muted-foreground">
                  {selectedImportIds.size} of {importCandidates.length} selected
                </span>
              </div>

              <Select
                value={importConflictMode}
                onChange={(value) => setImportConflictMode(value as ImportConflictMode)}
                options={[
                  { value: "rename", label: "Conflicts: Rename" },
                  { value: "skip", label: "Conflicts: Skip" },
                  { value: "replace", label: "Conflicts: Replace" },
                ]}
                className="w-44"
                aria-label="Conflict mode"
              />
            </div>

            {importCandidates.length === 0 ? (
              <div className="rounded-md border p-3 text-sm text-muted-foreground">
                No import candidates found for this source.
              </div>
            ) : (
              <ul className="space-y-2">
                {importCandidates.map((candidate) => (
                  <li key={candidate.id} className="rounded-md border p-3">
                    <div className="flex items-start gap-3">
                      <Checkbox
                        checked={selectedImportIds.has(candidate.id)}
                        onChange={(checked) => toggleImportCandidate(candidate.id, checked)}
                        aria-label={`Select candidate ${candidate.proposedName}`}
                      />
                      <div className="min-w-0 flex-1">
                        <div className="flex items-center gap-2">
                          <span className="font-medium truncate">{candidate.proposedName}</span>
                          <Badge variant="outline">{candidate.sourceLabel}</Badge>
                          <Badge variant={candidate.scope === "global" ? "default" : "secondary"}>
                            {candidate.scope}
                          </Badge>
                        </div>
                        <p className="text-xs text-muted-foreground truncate mt-1">
                          {candidate.sourcePath}
                        </p>
                      </div>
                    </div>
                  </li>
                ))}
              </ul>
            )}

            {importScanErrors.length > 0 && (
              <div className="rounded-md border border-destructive/40 bg-destructive/5 p-3 text-xs text-destructive">
                <p className="font-medium mb-1">Scan warnings</p>
                {importScanErrors.slice(0, 3).map((err) => (
                  <p key={err}>{err}</p>
                ))}
              </div>
            )}

            {importResult && (
              <div className="rounded-md border p-3 text-sm">
                <p className="font-medium">Latest Import Result</p>
                <p className="text-muted-foreground">
                  {importResult.imported.length} imported, {importResult.skipped.length} skipped,{" "}
                  {importResult.conflicts.length} conflicts, {importResult.errors.length} errors
                </p>
                {importResult.imported.length > 0 && (
                  <p className="text-xs text-muted-foreground mt-2">
                    Imported:{" "}
                    {importResult.imported
                      .slice(0, 3)
                      .map((r) => r.name)
                      .join(", ")}
                    {importResult.imported.length > 3 ? "..." : ""}
                  </p>
                )}
                {importResult.conflicts.length > 0 && (
                  <p className="text-xs text-destructive mt-1">
                    Conflict: {importResult.conflicts[0].candidateName}
                  </p>
                )}
              </div>
            )}

            {importHistory.length > 0 && (
              <div className="rounded-md border p-3 text-xs">
                <p className="font-medium mb-2">Recent Import Runs</p>
                <div className="space-y-1 text-muted-foreground">
                  {importHistory.slice(0, 3).map((entry) => (
                    <p key={entry.id}>
                      {new Date(entry.timestamp * 1000).toLocaleString()} - {entry.sourceType} -{" "}
                      {entry.importedCount} imported
                    </p>
                  ))}
                </div>
              </div>
            )}
          </div>

          <DialogFooter>
            <Button variant="outline" onClick={() => setImportDialogOpen(false)}>
              Close
            </Button>
            <Button
              variant="outline"
              onClick={() => {
                if (importSourceMode === "ai") {
                  void scanAiToolRules();
                } else if (importSourceMode === "file") {
                  void scanImportFromFile();
                } else if (importSourceMode === "directory") {
                  void scanImportFromDirectory();
                } else if (importSourceMode === "url") {
                  setUrlImportDialogOpen(true);
                } else {
                  void scanImportFromClipboard();
                }
              }}
              disabled={isScanningImport}
            >
              Rescan
            </Button>
            <Button onClick={executeImport} disabled={isImporting || selectedImportIds.size === 0}>
              {isImporting ? "Importing..." : "Import Selected"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={urlImportDialogOpen} onOpenChange={setUrlImportDialogOpen}>
        <DialogContent onClose={() => setUrlImportDialogOpen(false)}>
          <DialogHeader>
            <DialogTitle>Import Rules From URL</DialogTitle>
            <DialogDescription>Enter a URL to scan before importing.</DialogDescription>
          </DialogHeader>

          <Input
            value={urlImportValue}
            onChange={(e) => setUrlImportValue(e.target.value)}
            placeholder="https://example.com/rules.md"
            aria-label="Import URL"
          />

          <DialogFooter>
            <Button variant="outline" onClick={() => setUrlImportDialogOpen(false)}>
              Cancel
            </Button>
            <Button onClick={submitUrlImportScan}>Scan URL</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
