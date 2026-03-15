import { useState } from "react";
import { Trash2, AlertTriangle, CheckCircle, XCircle } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Select } from "@/components/ui/select";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import { useToast } from "@/components/ui/toast";
import { api } from "@/lib/tauri";
import { useRegistryStore } from "@/stores/registryStore";
import type { AdapterType } from "@/types/rule";
import type { CleanupResult } from "@/types/status";

export function CleanupCard() {
  const { addToast } = useToast();
  const { tools } = useRegistryStore();
  const [selectedAdapter, setSelectedAdapter] = useState<AdapterType | "all">("all");
  const [isConfirmOpen, setIsConfirmOpen] = useState(false);
  const [isCleaning, setIsCleaning] = useState(false);
  const [result, setResult] = useState<CleanupResult | null>(null);
  const [isResultOpen, setIsResultOpen] = useState(false);

  const adapterOptions = [
    { value: "all", label: "All Tools" },
    ...tools.map((tool) => ({ value: tool.id, label: tool.name })),
  ];

  const handleCleanup = async () => {
    setIsCleaning(true);
    setIsConfirmOpen(false);

    try {
      const filter = selectedAdapter === "all" ? {} : { adapter: selectedAdapter };
      const cleanupResult = await api.registry.cleanupSyncedFiles(filter);
      setResult(cleanupResult);
      setIsResultOpen(true);

      if (cleanupResult.errors.length > 0) {
        addToast({
          title: "Cleanup completed with errors",
          description: `${cleanupResult.filesRemoved} files removed, ${cleanupResult.errors.length} errors`,
          variant: "error",
        });
      } else if (cleanupResult.filesRemoved > 0) {
        addToast({
          title: "Cleanup successful",
          description: `${cleanupResult.filesRemoved} synced files removed`,
        });
      } else {
        addToast({
          title: "No files to clean",
          description: "No synced files were found matching the criteria",
        });
      }
    } catch (error) {
      addToast({
        title: "Cleanup failed",
        description: String(error),
        variant: "error",
      });
    } finally {
      setIsCleaning(false);
    }
  };

  return (
    <>
      <Card className="glass-card premium-shadow border-none overflow-hidden">
        <CardHeader className="bg-white/5 pb-4">
          <CardTitle className="text-sm font-semibold tracking-wide uppercase text-muted-foreground/80">
            Clean Synced Files
          </CardTitle>
          <CardDescription>
            Remove files that were synced by RuleWeaver from AI tool directories
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4 pt-6">
          <div className="flex items-center gap-4">
            <Select
              options={adapterOptions}
              value={selectedAdapter}
              onChange={(v) => setSelectedAdapter(v as AdapterType | "all")}
              className="w-48"
            />

            <Button
              variant="outline"
              className="glass"
              onClick={() => setIsConfirmOpen(true)}
              disabled={isCleaning}
            >
              <Trash2 className="mr-2 h-4 w-4" />
              {isCleaning ? "Cleaning..." : "Clean Up"}
            </Button>
          </div>

          <div className="rounded-xl border border-amber-500/20 bg-amber-500/5 p-3 text-xs text-muted-foreground">
            <div className="flex items-start gap-2">
              <AlertTriangle className="h-4 w-4 text-amber-500 mt-0.5 shrink-0" />
              <div>
                <strong className="text-amber-500">Warning:</strong> This will permanently delete
                synced files (like GEMINI.md, CLAUDE.md, etc.) from the selected tool directories.
                Your rules in RuleWeaver database will not be affected.
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      <Dialog open={isConfirmOpen} onOpenChange={setIsConfirmOpen}>
        <DialogContent className="glass max-w-md">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <AlertTriangle className="h-5 w-5 text-amber-500" />
              Confirm Cleanup
            </DialogTitle>
            <DialogDescription>
              Are you sure you want to remove synced files for{" "}
              {selectedAdapter === "all"
                ? "all tools"
                : tools.find((t) => t.id === selectedAdapter)?.name || selectedAdapter}
              ? This action cannot be undone.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" className="glass" onClick={() => setIsConfirmOpen(false)}>
              Cancel
            </Button>
            <Button variant="destructive" onClick={handleCleanup} disabled={isCleaning}>
              <Trash2 className="mr-2 h-4 w-4" />
              Confirm Cleanup
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={isResultOpen} onOpenChange={setIsResultOpen}>
        <DialogContent className="glass max-w-md">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              {result && result.errors.length === 0 ? (
                <CheckCircle className="h-5 w-5 text-green-500" />
              ) : (
                <XCircle className="h-5 w-5 text-amber-500" />
              )}
              Cleanup Results
            </DialogTitle>
          </DialogHeader>
          {result && (
            <div className="space-y-3 py-4">
              <div className="grid grid-cols-2 gap-3">
                <div className="rounded-xl border border-white/5 bg-white/5 p-3 text-center">
                  <div className="text-xl font-black text-green-500">{result.filesRemoved}</div>
                  <div className="text-[9px] uppercase font-black text-muted-foreground/60">
                    Files Removed
                  </div>
                </div>
                <div className="rounded-xl border border-white/5 bg-white/5 p-3 text-center">
                  <div className="text-xl font-black text-muted-foreground">
                    {result.filesSkipped}
                  </div>
                  <div className="text-[9px] uppercase font-black text-muted-foreground/60">
                    Files Skipped
                  </div>
                </div>
              </div>

              {result.removedPaths.length > 0 && (
                <div className="rounded-xl border border-white/5 bg-white/5 p-3">
                  <div className="text-xs font-bold mb-2">Removed Paths:</div>
                  <div className="max-h-32 overflow-y-auto space-y-1">
                    {result.removedPaths.map((path, i) => (
                      <div key={i} className="text-xs font-mono text-muted-foreground truncate">
                        {path}
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {result.errors.length > 0 && (
                <div className="rounded-xl border border-red-500/20 bg-red-500/5 p-3">
                  <div className="text-xs font-bold text-red-500 mb-2">Errors:</div>
                  <div className="max-h-24 overflow-y-auto space-y-1">
                    {result.errors.map((error, i) => (
                      <div key={i} className="text-xs text-red-400">
                        {error}
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
          )}
          <DialogFooter>
            <Button onClick={() => setIsResultOpen(false)}>Close</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}
