import { useEffect, useState, useMemo, useRef } from "react";
import type { DragEvent } from "react";
import { cn } from "@/lib/utils";
import { Plus, Upload, FileText, FolderUp, Link, Clipboard } from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Checkbox } from "@/components/ui/checkbox";
import { Input } from "@/components/ui/input";
import { Select } from "@/components/ui/select";
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
import { RulesFilterBar } from "@/components/rules/RulesFilterBar";
import { parseSortValue } from "@/components/rules/filter-utils";
import { RuleCard } from "@/components/rules/RuleCard";
import { toast } from "@/lib/toast-helpers";
import {
  ADAPTERS,
  type Rule,
  type AdapterType,
  type ImportCandidate,
  type ImportConflictMode,
  type ImportExecutionOptions,
  type ImportExecutionResult,
} from "@/types/rule";
import { api } from "@/lib/tauri";
import { RuleTemplateBrowser } from "@/components/rules/RuleTemplateBrowser";

type ImportSourceMode = "ai" | "file" | "directory" | "url" | "clipboard";

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
  const [isImporting, setIsImporting] = useState(false);
  const [isScanningImport, setIsScanningImport] = useState(false);
  const [importDialogOpen, setImportDialogOpen] = useState(false);
  const [importCandidates, setImportCandidates] = useState<ImportCandidate[]>([]);
  const [selectedImportIds, setSelectedImportIds] = useState<Set<string>>(new Set());
  const [importScanErrors, setImportScanErrors] = useState<string[]>([]);
  const [importConflictMode, setImportConflictMode] = useState<ImportConflictMode>("rename");
  const [importResult, setImportResult] = useState<ImportExecutionResult | null>(null);
  const [importSourceMode, setImportSourceMode] = useState<ImportSourceMode>("ai");
  const [importSourceValue, setImportSourceValue] = useState("");
  const [clipboardImportName, setClipboardImportName] = useState<string | undefined>(undefined);
  const [urlImportDialogOpen, setUrlImportDialogOpen] = useState(false);
  const [urlImportValue, setUrlImportValue] = useState("");
  const [clipboardNameDialogOpen, setClipboardNameDialogOpen] = useState(false);
  const [clipboardPendingContent, setClipboardPendingContent] = useState("");
  const [clipboardNameInput, setClipboardNameInput] = useState("");
  const [importScopeOverride, setImportScopeOverride] = useState<"source" | "global" | "local">(
    "source"
  );
  const [useAdapterOverride, setUseAdapterOverride] = useState(false);
  const [adapterOverrideSet, setAdapterOverrideSet] = useState<Set<AdapterType>>(new Set());
  const [isDragImportActive, setIsDragImportActive] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);

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
      await duplicateRule(rule);
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
    setImportScopeOverride("source");
    setUseAdapterOverride(false);
    setAdapterOverrideSet(new Set());
  };

  const getImportExecutionOptions = (): ImportExecutionOptions => ({
    conflictMode: importConflictMode,
    selectedCandidateIds: Array.from(selectedImportIds),
    defaultScope: importScopeOverride === "source" ? undefined : importScopeOverride,
    defaultAdapters: useAdapterOverride ? Array.from(adapterOverrideSet) : undefined,
  });

  const toggleAdapterOverride = (adapter: AdapterType, checked: boolean) => {
    setAdapterOverrideSet((prev) => {
      const next = new Set(prev);
      if (checked) {
        next.add(adapter);
      } else {
        next.delete(adapter);
      }
      return next;
    });
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

    setIsScanningImport(true);
    try {
      const scan = await api.ruleImport.scanFromFile(filePath);
      await openImportPreview("file", filePath, scan.candidates, scan.errors);
    } catch (error) {
      toast.error(addToast, { title: "Scan Failed", error });
    } finally {
      setIsScanningImport(false);
    }
  };

  const scanAiToolRules = async () => {
    setIsScanningImport(true);
    try {
      const scan = await api.ruleImport.scanAiToolCandidates();
      await openImportPreview("ai", "", scan.candidates, scan.errors);
    } catch (error) {
      toast.error(addToast, { title: "Scan Failed", error });
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
      toast.error(addToast, { title: "Scan Failed", error });
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
      toast.error(addToast, { title: "Scan Failed", error });
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
      toast.error(addToast, { title: "Scan Failed", error });
    } finally {
      setIsScanningImport(false);
    }
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

      setClipboardPendingContent(text);
      setClipboardNameInput(clipboardImportName ?? "");
      setClipboardNameDialogOpen(true);
    } catch (error) {
      toast.error(addToast, { title: "Scan Failed", error });
    }
  };

  const submitClipboardImportScan = async () => {
    if (!clipboardPendingContent.trim()) {
      setClipboardNameDialogOpen(false);
      return;
    }

    setClipboardNameDialogOpen(false);
    setIsScanningImport(true);
    try {
      const name = clipboardNameInput.trim() || undefined;
      setClipboardImportName(name);
      const scan = await api.ruleImport.scanFromClipboard(clipboardPendingContent, name);
      await openImportPreview("clipboard", clipboardPendingContent, scan.candidates, scan.errors);
    } catch (error) {
      toast.error(addToast, { title: "Scan Failed", error });
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
      const options = getImportExecutionOptions();

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
      toast.error(addToast, { title: "Import Failed", error });
    } finally {
      setIsImporting(false);
    }
  };

  const retryConflictsAsRename = async () => {
    if (!importResult || importResult.conflicts.length === 0) {
      return;
    }
    setSelectedImportIds(new Set(importResult.conflicts.map((c) => c.candidateId)));
    setImportConflictMode("rename");
    await executeImport();
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
            onClick={() => setUrlImportDialogOpen(true)}
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

            <div className="grid grid-cols-1 gap-2 rounded-md border p-3">
              <Select
                value={importScopeOverride}
                onChange={(value) => setImportScopeOverride(value as "source" | "global" | "local")}
                options={[
                  { value: "source", label: "Scope: Use source" },
                  { value: "global", label: "Scope: Force global" },
                  { value: "local", label: "Scope: Force local" },
                ]}
                aria-label="Scope override"
              />

              <div className="flex items-center gap-2">
                <Checkbox
                  checked={useAdapterOverride}
                  onChange={setUseAdapterOverride}
                  aria-label="Enable adapter override"
                />
                <span className="text-sm text-muted-foreground">Override adapters on import</span>
              </div>

              {useAdapterOverride && (
                <div className="grid grid-cols-2 gap-2">
                  {ADAPTERS.map((adapter) => (
                    <label key={adapter.id} className="flex items-center gap-2 text-sm">
                      <Checkbox
                        checked={adapterOverrideSet.has(adapter.id)}
                        onChange={(checked) => toggleAdapterOverride(adapter.id, checked)}
                        aria-label={`Use adapter ${adapter.name}`}
                      />
                      <span>{adapter.name}</span>
                    </label>
                  ))}
                </div>
              )}
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
                          <span className="text-xs px-2 py-0.5 rounded-full border">
                            {candidate.sourceLabel}
                          </span>
                          <span
                            className={`text-xs px-2 py-0.5 rounded-full ${candidate.scope === "global" ? "bg-primary/20 text-primary" : "bg-secondary"}`}
                          >
                            {candidate.scope}
                          </span>
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
                {importResult.conflicts.length > 0 && (
                  <div className="mt-2 flex items-center justify-between gap-2">
                    <p className="text-xs text-destructive">
                      Conflict: {importResult.conflicts[0].candidateName}
                    </p>
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => void retryConflictsAsRename()}
                    >
                      Retry Conflicts
                    </Button>
                  </div>
                )}
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
            type="url"
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

      <Dialog open={clipboardNameDialogOpen} onOpenChange={setClipboardNameDialogOpen}>
        <DialogContent onClose={() => setClipboardNameDialogOpen(false)}>
          <DialogHeader>
            <DialogTitle>Clipboard Import Name</DialogTitle>
            <DialogDescription>
              Optionally provide a name used for preview and import.
            </DialogDescription>
          </DialogHeader>

          <Input
            type="text"
            value={clipboardNameInput}
            onChange={(e) => setClipboardNameInput(e.target.value)}
            placeholder="clipboard-import"
            aria-label="Clipboard import name"
          />

          <DialogFooter>
            <Button variant="outline" onClick={() => setClipboardNameDialogOpen(false)}>
              Cancel
            </Button>
            <Button onClick={submitClipboardImportScan}>Scan Clipboard</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
