import { useState, useCallback, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
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
import { useRegistryStore } from "@/stores/registryStore";
import { toast } from "@/lib/toast-helpers";
import {
  type Scope,
  type AdapterType,
  type ImportCandidate,
  type ImportConflictMode,
  type ImportExecutionOptions,
  type ImportExecutionResult,
  type ImportArtifactType,
} from "@/types/rule";

export type ImportSourceMode = "ai" | "file" | "directory" | "url" | "clipboard";

export interface ImportDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onImportComplete: (result: ImportExecutionResult) => Promise<void>;
  artifactType: ImportArtifactType;
  title?: string;
  initialSourceMode?: ImportSourceMode | null;
}

export function ImportDialog({
  open: isOpen,
  onOpenChange,
  onImportComplete,
  artifactType,
  title: titleProp,
  initialSourceMode,
}: ImportDialogProps) {
  const { tools } = useRegistryStore();
  const { addToast } = useToast();
  const [isImporting, setIsImporting] = useState(false);
  const [isScanningImport, setIsScanningImport] = useState(false);
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
  const [importScopeOverride, setImportScopeOverride] = useState<"source" | Scope>("source");
  const [useAdapterOverride, setUseAdapterOverride] = useState(false);
  const [adapterOverrideSet, setAdapterOverrideSet] = useState<Set<AdapterType>>(new Set());

  const handleImportResult = useCallback(
    async (title: string, result: ImportExecutionResult) => {
      await onImportComplete(result);

      const totalImported =
        (result.importedRules?.length || 0) +
          (result.importedCommands?.length || 0) +
          (result.importedSkills?.length || 0) || result.imported.length;

      addToast({
        title,
        description: `${totalImported} imported, ${result.skipped.length} skipped, ${result.conflicts.length} conflicts`,
        variant: result.errors.length > 0 ? "error" : "success",
      });
      if (result.errors.length > 0) {
        addToast({
          title: "Import Warnings",
          description: result.errors[0],
          variant: "error",
        });
      }
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
      setImportScopeOverride("source");
    },
    []
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

  const scanAiToolArtifacts = useCallback(async () => {
    setIsScanningImport(true);
    try {
      if (artifactType === "command") {
        await api.ruleImport.importAiToolCommands({
          conflictMode: "skip",
          selectedCandidateIds: [],
        });
      } else if (artifactType === "skill") {
        await api.ruleImport.importAiToolSkills({ conflictMode: "skip", selectedCandidateIds: [] });
      }

      // Backend scan_ai_tool_import_candidates now returns EVERYTHING.
      // We should filter it here for the current artifactType.
      const allScan = await api.ruleImport.scanAiToolCandidates();
      const filteredCandidates = allScan.candidates.filter((c) => c.artifactType === artifactType);

      await openImportPreview("ai", "", filteredCandidates, allScan.errors);
    } catch (error) {
      toast.error(addToast, { title: "Scan Failed", error });
    } finally {
      setIsScanningImport(false);
    }
  }, [artifactType, addToast, openImportPreview]);

  const scanImportFromFile = useCallback(async () => {
    const selected = await open({
      multiple: false,
      filters: [{ name: "Artifact Files", extensions: ["md", "txt", "json", "yaml", "yml"] }],
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
  }, [addToast, openImportPreview]);

  const scanImportFromDirectory = useCallback(async () => {
    const selected = await open({ directory: true, multiple: false });
    if (!selected || Array.isArray(selected)) return;

    setIsScanningImport(true);
    try {
      let scan;
      if (artifactType === "command") {
        scan = await api.ruleImport.scanCommandDirectoryImport(selected);
      } else if (artifactType === "skill") {
        scan = await api.ruleImport.scanSkillDirectoryImport(selected);
      } else {
        scan = await api.ruleImport.scanFromDirectory(selected);
      }
      await openImportPreview("directory", selected, scan.candidates, scan.errors);
    } catch (error) {
      toast.error(addToast, { title: "Scan Failed", error });
    } finally {
      setIsScanningImport(false);
    }
  }, [artifactType, addToast, openImportPreview]);

  const scanImportFromUrl = useCallback(
    async (url: string) => {
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
    },
    [addToast, openImportPreview]
  );

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

  const scanImportFromClipboard = useCallback(async () => {
    try {
      const text = await navigator.clipboard.readText();
      if (!text.trim()) {
        toast.error(addToast, {
          title: "Clipboard Empty",
          description: "No text found in clipboard",
        });
        return;
      }

      setIsScanningImport(true);
      try {
        const scan = await api.ruleImport.scanFromClipboard(text);
        await openImportPreview("clipboard", text, scan.candidates, scan.errors);
      } catch (error) {
        toast.error(addToast, { title: "Scan Failed", error });
      } finally {
        setIsScanningImport(false);
      }
    } catch {
      toast.error(addToast, {
        title: "Clipboard Access Denied",
        description: "Please allow clipboard permissions",
      });
    }
  }, [addToast, openImportPreview]);

  useEffect(() => {
    if (initialSourceMode && isOpen) {
      setImportSourceMode(initialSourceMode);
      // Trigger auto-scan for AI, File, or Directory if they are initial
      if (initialSourceMode === "ai") {
        void scanAiToolArtifacts();
      } else if (initialSourceMode === "file") {
        void scanImportFromFile();
      } else if (initialSourceMode === "directory") {
        void scanImportFromDirectory();
      } else if (initialSourceMode === "url") {
        setUrlImportDialogOpen(true);
      }
    }
  }, [initialSourceMode, isOpen, scanAiToolArtifacts, scanImportFromFile, scanImportFromDirectory]);

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

      if (importSourceMode === "directory") {
        if (artifactType === "command") {
          result = await api.ruleImport.importCommandsFromDirectory(importSourceValue, options);
        } else if (artifactType === "skill") {
          result = await api.ruleImport.importSkillsFromDirectory(importSourceValue, options);
        } else {
          result = await api.ruleImport.importFromDirectory(importSourceValue, options);
        }
      } else {
        // Fallback to legacy single-artifact paths which backend handles correctly now for Rules
        // Backend generalized execute_import handles all types if candidates are passed correctly.
        // However, scanFromFile/scanFromUrl currently defaults to Rule in legacy paths.
        if (importSourceMode === "ai") {
          result = await api.ruleImport.importAiToolRules(options);
        } else if (importSourceMode === "file") {
          result = await api.ruleImport.importFromFile(importSourceValue, options);
        } else if (importSourceMode === "url") {
          result = await api.ruleImport.importFromUrl(importSourceValue, options);
        } else {
          result = await api.ruleImport.importFromClipboard(
            importSourceValue,
            clipboardImportName,
            options
          );
        }
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
      void scanAiToolArtifacts();
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

  const artifactLabelPlural =
    artifactType === "rule" ? "Rules" : artifactType === "command" ? "Commands" : "Skills";

  return (
    <>
      <Dialog open={isOpen} onOpenChange={onOpenChange}>
        <DialogContent onClose={() => onOpenChange(false)} className="max-w-2xl">
          <DialogHeader>
            <DialogTitle>
              {titleProp ||
                (importSourceMode === "ai"
                  ? `Import Existing AI Tool ${artifactLabelPlural}`
                  : importSourceMode === "file"
                    ? `Import ${artifactLabelPlural} From File`
                    : importSourceMode === "directory"
                      ? `Import ${artifactLabelPlural} From Folder`
                      : importSourceMode === "url"
                        ? `Import ${artifactLabelPlural} From URL`
                        : `Import ${artifactLabelPlural} From Clipboard`)}
            </DialogTitle>
            <DialogDescription>
              Review discovered candidates, choose conflict handling, and import selected{" "}
              {artifactLabelPlural.toLowerCase()}.
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-4 max-h-[60vh] overflow-y-auto pr-2">
            {importSourceMode !== "ai" && importSourceValue && (
              <div className="rounded-md border p-3 text-xs font-mono text-muted-foreground break-all bg-black/10">
                <span className="font-bold mr-1 uppercase">Source: </span>
                {importSourceMode === "clipboard" ? "Clipboard CONTENT" : importSourceValue}
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
                  aria-label="Select all candidates"
                />
                <span className="text-sm text-muted-foreground font-medium">
                  {selectedImportIds.size} of {importCandidates.length} selected
                </span>
              </div>

              <div className="flex items-center gap-2">
                <span className="text-xs font-bold uppercase text-muted-foreground/60">
                  Conflicts:
                </span>
                <Select
                  value={importConflictMode}
                  onChange={(value) => setImportConflictMode(value as ImportConflictMode)}
                  options={[
                    { value: "rename", label: "Rename" },
                    { value: "skip", label: "Skip" },
                    { value: "replace", label: "Replace" },
                  ]}
                  className="w-32"
                  aria-label="Conflict mode"
                />
              </div>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-3 rounded-xl border border-white/5 bg-white/5 p-4">
              <div className="space-y-2">
                <label className="text-[10px] font-bold uppercase tracking-widest text-muted-foreground/60">
                  Scope Override
                </label>
                <Select
                  value={importScopeOverride}
                  onChange={(value) => setImportScopeOverride(value as "source" | Scope)}
                  options={[
                    { value: "source", label: "Use source preference" },
                    { value: "global", label: "Force global" },
                    { value: "local", label: "Force local" },
                  ]}
                  aria-label="Scope override"
                />
              </div>

              <div className="space-y-2">
                <label className="text-[10px] font-bold uppercase tracking-widest text-muted-foreground/60">
                  Adapters Override
                </label>
                <div className="flex items-center gap-2 h-10 px-3 rounded-md border bg-black/20">
                  <Checkbox
                    checked={useAdapterOverride}
                    onChange={setUseAdapterOverride}
                    aria-label="Enable adapter override"
                  />
                  <span className="text-xs text-muted-foreground">Override tool adapters</span>
                </div>
              </div>

              {useAdapterOverride && (
                <div className="md:col-span-2 grid grid-cols-3 gap-2 p-3 rounded-lg border border-dashed border-white/10">
                  {tools.map((adapter) => (
                    <label key={adapter.id} className="flex items-center gap-2 text-xs truncate">
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

            <div className="min-h-[200px]">
              <AnimatePresence mode="wait">
                {isScanningImport ? (
                  <motion.div
                    key="loading"
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    exit={{ opacity: 0 }}
                    className="space-y-2"
                  >
                    {[1, 2, 3].map((i) => (
                      <div
                        key={i}
                        className="h-16 rounded-xl border border-white/5 bg-white/5 animate-pulse"
                      />
                    ))}
                  </motion.div>
                ) : importCandidates.length === 0 ? (
                  <motion.div
                    key="empty"
                    initial={{ opacity: 0, y: 10 }}
                    animate={{ opacity: 1, y: 0 }}
                    className="rounded-xl border border-dashed p-12 text-center"
                  >
                    <p className="text-sm text-muted-foreground">
                      No valid candidates found in this source.
                    </p>
                  </motion.div>
                ) : (
                  <motion.ul
                    key="list"
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    className="space-y-2"
                  >
                    {importCandidates.map((candidate) => (
                      <motion.li
                        layout
                        initial={{ opacity: 0, x: -10 }}
                        animate={{ opacity: 1, x: 0 }}
                        key={candidate.id}
                        className="group rounded-xl border border-white/5 bg-white/5 p-3 hover:border-primary/20 transition-all"
                      >
                        <div className="flex items-start gap-4">
                          <Checkbox
                            checked={selectedImportIds.has(candidate.id)}
                            onChange={(checked) => toggleImportCandidate(candidate.id, checked)}
                            aria-label={`Select candidate ${candidate.proposedName}`}
                            className="mt-1"
                          />
                          <div className="min-w-0 flex-1">
                            <div className="flex items-center flex-wrap gap-2">
                              <span className="font-bold truncate text-sm">
                                {candidate.proposedName}
                              </span>
                              <Badge
                                variant="outline"
                                className="text-[9px] uppercase font-black px-1.5 py-0 bg-primary/5 text-primary border-primary/20"
                              >
                                {candidate.sourceLabel}
                              </Badge>
                              <Badge
                                variant={candidate.scope === "global" ? "default" : "secondary"}
                                className="text-[9px] uppercase font-black px-1.5 py-0"
                              >
                                {candidate.scope}
                              </Badge>
                              <Badge
                                variant="outline"
                                className={`text-[9px] uppercase font-black px-1.5 py-0 ${
                                  candidate.artifactType === "rule"
                                    ? "bg-blue-500/10 text-blue-400 border-blue-500/20"
                                    : candidate.artifactType === "command"
                                      ? "bg-purple-500/10 text-purple-400 border-purple-500/20"
                                      : "bg-emerald-500/10 text-emerald-400 border-emerald-500/20"
                                }`}
                              >
                                {candidate.artifactType}
                              </Badge>
                            </div>
                            <p className="text-[10px] text-muted-foreground/60 truncate mt-1.5 font-mono">
                              {candidate.sourcePath}
                            </p>
                          </div>
                        </div>
                      </motion.li>
                    ))}
                  </motion.ul>
                )}
              </AnimatePresence>
            </div>

            {importScanErrors.length > 0 && (
              <div className="rounded-xl border border-destructive/20 bg-destructive/5 p-4">
                <p className="text-[10px] font-bold uppercase tracking-widest text-destructive mb-2">
                  Scan warnings
                </p>
                <div className="space-y-1">
                  {importScanErrors.slice(0, 3).map((err, idx) => (
                    <p key={idx} className="text-xs text-destructive/80">
                      {err}
                    </p>
                  ))}
                  {importScanErrors.length > 3 && (
                    <p className="text-[10px] text-destructive/40 italic">
                      ...and {importScanErrors.length - 3} more
                    </p>
                  )}
                </div>
              </div>
            )}

            {importResult && (
              <div className="rounded-xl border border-primary/20 bg-primary/5 p-4">
                <p className="text-[10px] font-bold uppercase tracking-widest text-primary mb-2">
                  Execution Report
                </p>
                <div className="flex gap-4 mb-3">
                  <div className="text-center flex-1 p-2 rounded-lg bg-black/20">
                    <div className="text-xl font-black text-primary">
                      {importResult.importedRules?.length || importResult.imported.length}
                    </div>
                    <div className="text-[9px] uppercase font-bold text-muted-foreground/60">
                      Imported
                    </div>
                  </div>
                  <div className="text-center flex-1 p-2 rounded-lg bg-black/20">
                    <div className="text-xl font-black">{importResult.skipped.length}</div>
                    <div className="text-[9px] uppercase font-bold text-muted-foreground/60">
                      Skipped
                    </div>
                  </div>
                  <div className="text-center flex-1 p-2 rounded-lg bg-black/20">
                    <div className="text-xl font-black text-amber-500">
                      {importResult.conflicts.length}
                    </div>
                    <div className="text-[9px] uppercase font-bold text-muted-foreground/60">
                      Conflicts
                    </div>
                  </div>
                </div>

                {importResult.conflicts.length > 0 && (
                  <div className="flex items-center justify-between gap-3 pt-3 border-t border-white/5">
                    <p className="text-[10px] text-amber-500 font-medium">
                      Collision detected for "{importResult.conflicts[0].candidateName}"
                    </p>
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => void retryConflictsAsRename()}
                      className="h-7 text-[10px] px-2 uppercase font-black"
                    >
                      Retry as Renames
                    </Button>
                  </div>
                )}
              </div>
            )}
          </div>

          <DialogFooter className="gap-2 sm:gap-0 border-t border-white/5 pt-6 mt-2">
            <Button
              variant="ghost"
              onClick={() => onOpenChange(false)}
              className="text-xs uppercase font-black tracking-widest text-muted-foreground/60"
            >
              Close
            </Button>
            <div className="flex gap-2">
              <Button
                variant="outline"
                onClick={handleRescan}
                disabled={isScanningImport}
                className="text-xs uppercase font-black tracking-widest glass"
              >
                Rescan
              </Button>
              <Button
                onClick={executeImport}
                disabled={isImporting || selectedImportIds.size === 0}
                className="text-xs uppercase font-black tracking-widest glow-primary"
              >
                {isImporting ? "Importing..." : "Process Import"}
              </Button>
            </div>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Helper Dialogs for Specific Sources */}
      <Dialog open={urlImportDialogOpen} onOpenChange={setUrlImportDialogOpen}>
        <DialogContent onClose={() => setUrlImportDialogOpen(false)} className="max-w-md">
          <DialogHeader>
            <DialogTitle>Import from URL</DialogTitle>
            <DialogDescription>Enter a raw file or documentation URL to scan.</DialogDescription>
          </DialogHeader>

          <Input
            value={urlImportValue}
            onChange={(e) => setUrlImportValue(e.target.value)}
            placeholder="https://example.com/artifacts.md"
            aria-label="Import URL"
            className="bg-black/20"
          />

          <DialogFooter>
            <Button variant="ghost" onClick={() => setUrlImportDialogOpen(false)}>
              Cancel
            </Button>
            <Button onClick={submitUrlImportScan} className="glow-primary">
              Scan Remote Source
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}
