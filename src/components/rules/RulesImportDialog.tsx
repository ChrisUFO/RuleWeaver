import { useState, useCallback } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Checkbox } from "@/components/ui/checkbox";
import { Badge } from "@/components/ui/badge";
import { Select } from "@/components/ui/select";
import { useToast } from "@/components/ui/toast";
import { api } from "@/lib/tauri";
import { toast } from "@/lib/toast-helpers";
import {
  ADAPTERS,
  type Scope,
  type AdapterType,
  type ImportCandidate,
  type ImportConflictMode,
  type ImportExecutionOptions,
  type ImportExecutionResult,
  type ImportHistoryEntry,
} from "@/types/rule";

type ImportSourceMode = "ai" | "file" | "directory" | "url" | "clipboard";

interface RulesImportDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onImportComplete: () => Promise<void>;
}

export function RulesImportDialog({
  open: onOpenProp,
  onOpenChange,
  onImportComplete,
}: RulesImportDialogProps) {
  const { addToast } = useToast();
  const [isImporting, setIsImporting] = useState(false);
  const [isScanningImport, setIsScanningImport] = useState(false);
  const [importDialogOpen, setImportDialogOpen] = useState(false);
  const [importCandidates, setImportCandidates] = useState<ImportCandidate[]>([]);
  const [selectedImportIds, setSelectedImportIds] = useState<Set<string>>(new Set());
  const [importScanErrors, setImportScanErrors] = useState<string[]>([]);
  const [importConflictMode, setImportConflictMode] = useState<ImportConflictMode>("rename");
  const [importResult, setImportResult] = useState<ImportExecutionResult | null>(null);
  const [importHistory, setImportHistory] = useState<ImportHistoryEntry[]>([]);
  const [importHistoryFilter, setImportHistoryFilter] = useState<
    "all" | "ai_tool" | "file" | "directory" | "url" | "clipboard"
  >("all");
  const [importSourceMode, setImportSourceMode] = useState<ImportSourceMode>("ai");
  const [importSourceValue, setImportSourceValue] = useState("");
  const [clipboardImportName, setClipboardImportName] = useState<string | undefined>(undefined);
  const [urlImportDialogOpen, setUrlImportDialogOpen] = useState(false);
  const [urlImportValue, setUrlImportValue] = useState("");
  const [clipboardNameDialogOpen, setClipboardNameDialogOpen] = useState(false);
  const [clipboardPendingContent, setClipboardPendingContent] = useState("");
  const [clipboardNameInput, setClipboardNameInput] = useState("");
  const [importScopeOverride, setImportScopeOverride] = useState<"source" | Scope>("source");
  const [useAdapterOverride, setUseAdapterOverride] = useState(false);
  const [adapterOverrideSet, setAdapterOverrideSet] = useState<Set<AdapterType>>(new Set());

  const isOpen = onOpenProp ?? importDialogOpen;
  const setIsOpen = onOpenChange ?? setImportDialogOpen;

  const handleImportResult = useCallback(
    async (title: string, result: ImportExecutionResult) => {
      await onImportComplete();
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
    },
    [onImportComplete, addToast]
  );

  const openImportPreview = useCallback(
    async (
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
      setIsOpen(true);
      setImportScopeOverride("source");
      setUseAdapterOverride(false);
      setAdapterOverrideSet(new Set());
      const history = await api.ruleImport.getHistory();
      setImportHistory(history);
    },
    [setIsOpen]
  );

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
      toast.error(addToast, {
        title: "URL Required",
        description: "Enter a URL to scan for import",
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
        toast.error(addToast, {
          title: "Clipboard Empty",
          description: "No text found in clipboard",
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
      toast.error(addToast, {
        title: "No Candidates Selected",
        description: "Select at least one candidate to import",
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

  const handleRescan = () => {
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
  };

  return (
    <>
      <Dialog open={isOpen} onOpenChange={setIsOpen}>
        <DialogContent onClose={() => setIsOpen(false)}>
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
                onChange={(value) => setImportScopeOverride(value as "source" | Scope)}
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

            {importHistory.length > 0 && (
              <div className="rounded-md border p-3 text-xs">
                <div className="mb-2 flex items-center justify-between gap-2">
                  <p className="font-medium">Recent Import Runs</p>
                  <Select
                    value={importHistoryFilter}
                    onChange={(value) =>
                      setImportHistoryFilter(
                        value as "all" | "ai_tool" | "file" | "directory" | "url" | "clipboard"
                      )
                    }
                    options={[
                      { value: "all", label: "All sources" },
                      { value: "ai_tool", label: "AI Tool" },
                      { value: "file", label: "File" },
                      { value: "directory", label: "Directory" },
                      { value: "url", label: "URL" },
                      { value: "clipboard", label: "Clipboard" },
                    ]}
                    className="w-32"
                    aria-label="Import history source filter"
                  />
                </div>
                <div className="space-y-1 text-muted-foreground">
                  {importHistory
                    .filter((entry) =>
                      importHistoryFilter === "all"
                        ? true
                        : entry.sourceType === importHistoryFilter
                    )
                    .slice(0, 3)
                    .map((entry) => (
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
            <Button variant="outline" onClick={() => setIsOpen(false)}>
              Close
            </Button>
            <Button variant="outline" onClick={handleRescan} disabled={isScanningImport}>
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

      <Dialog open={clipboardNameDialogOpen} onOpenChange={setClipboardNameDialogOpen}>
        <DialogContent onClose={() => setClipboardNameDialogOpen(false)}>
          <DialogHeader>
            <DialogTitle>Clipboard Import Name</DialogTitle>
            <DialogDescription>
              Optionally provide a name used for preview and import.
            </DialogDescription>
          </DialogHeader>

          <Input
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
    </>
  );
}
