import { CheckCircle, AlertTriangle, XCircle, ExternalLink } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import { Badge } from "@/components/ui/badge";
import type { SyncResult, Conflict, Rule } from "@/types/rule";
import { api } from "@/lib/tauri";
import { ConflictResolutionDialog } from "./ConflictResolutionDialog";

interface SyncPreviewDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  previewResult: SyncResult | null;
  rules: Rule[];
  onConfirm: () => void;
  onCancel: () => void;
  onConflictResolved: () => void;
}

export function SyncPreviewDialog({
  open,
  onOpenChange,
  previewResult,
  rules,
  onConfirm,
  onCancel,
  onConflictResolved,
}: SyncPreviewDialogProps) {
  const [selectedConflict, setSelectedConflict] = useState<Conflict | null>(null);
  const [conflictDialogOpen, setConflictDialogOpen] = useState(false);

  const getFileStatus = (filePath: string) => {
    if (!previewResult) return "new";
    const isConflict = previewResult.conflicts.some((c) => c.filePath === filePath);
    if (isConflict) return "conflict";
    return "modified";
  };

  const handleOpenFolder = async (filePath: string) => {
    const dirPath = filePath.substring(0, filePath.lastIndexOf("/"));
    try {
      await api.app.openInExplorer(dirPath);
    } catch {
      console.error("Failed to open folder");
    }
  };

  const handleConflictClick = (conflict: Conflict) => {
    setSelectedConflict(conflict);
    setConflictDialogOpen(true);
  };

  const handleConflictResolved = () => {
    onConflictResolved();
  };

  if (!previewResult) return null;

  const newFiles = previewResult.filesWritten.filter(
    (f) => !previewResult.conflicts.some((c) => c.filePath === f)
  );
  const conflictFiles = previewResult.conflicts;

  return (
    <>
      <Dialog open={open} onOpenChange={onOpenChange}>
        <DialogContent
          className="max-w-2xl max-h-[80vh] overflow-hidden flex flex-col"
          onClose={onCancel}
        >
          <DialogHeader>
            <DialogTitle>Sync Preview</DialogTitle>
            <DialogDescription>
              Review the files that will be updated before syncing
            </DialogDescription>
          </DialogHeader>

          <div className="flex-1 overflow-auto py-4 space-y-4">
            <div className="flex gap-4 text-sm">
              <div className="flex items-center gap-2 text-success">
                <CheckCircle className="h-4 w-4" />
                <span>{newFiles.length} files to update</span>
              </div>
              {conflictFiles.length > 0 && (
                <div className="flex items-center gap-2 text-warning">
                  <AlertTriangle className="h-4 w-4" />
                  <span>{conflictFiles.length} conflicts</span>
                </div>
              )}
            </div>

            {previewResult.filesWritten.length === 0 ? (
              <p className="text-muted-foreground text-center py-8">
                No files need to be synced. All rules are up to date.
              </p>
            ) : (
              <div className="space-y-2">
                {previewResult.filesWritten.map((filePath) => {
                  const status = getFileStatus(filePath);
                  const conflict = conflictFiles.find((c) => c.filePath === filePath);
                  const fileName = filePath.split("/").pop() || filePath;

                  return (
                    <div
                      key={filePath}
                      className={`flex items-center justify-between p-3 rounded-md border bg-muted/30 ${
                        status === "conflict" ? "cursor-pointer hover:bg-accent/50" : ""
                      }`}
                      onClick={() => conflict && handleConflictClick(conflict)}
                      role={status === "conflict" ? "button" : undefined}
                      tabIndex={status === "conflict" ? 0 : undefined}
                      onKeyDown={(e) => {
                        if (status === "conflict" && (e.key === "Enter" || e.key === " ")) {
                          handleConflictClick(conflict!);
                        }
                      }}
                    >
                      <div className="flex items-center gap-3">
                        {status === "conflict" ? (
                          <AlertTriangle className="h-4 w-4 text-warning" />
                        ) : (
                          <CheckCircle className="h-4 w-4 text-success" />
                        )}
                        <div>
                          <div className="flex items-center gap-2">
                            <span className="font-medium text-sm">{fileName}</span>
                            <Badge variant={status === "conflict" ? "warning" : "outline"}>
                              {status === "conflict" ? "CONFLICT" : "Modified"}
                            </Badge>
                          </div>
                          <p className="text-xs text-muted-foreground truncate max-w-md">
                            {filePath}
                          </p>
                          {conflict && (
                            <p className="text-xs text-warning mt-1">Click to resolve conflict</p>
                          )}
                        </div>
                      </div>
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={(e) => {
                          e.stopPropagation();
                          handleOpenFolder(filePath);
                        }}
                        title="Open in file manager"
                      >
                        <ExternalLink className="h-4 w-4" />
                      </Button>
                    </div>
                  );
                })}
              </div>
            )}

            {previewResult.errors.length > 0 && (
              <div className="space-y-2">
                <h4 className="font-medium text-destructive">Errors</h4>
                {previewResult.errors.map((error, index) => (
                  <div
                    key={index}
                    className="flex items-start gap-2 p-2 rounded-md bg-destructive/10 text-destructive text-sm"
                  >
                    <XCircle className="h-4 w-4 mt-0.5" />
                    <div>
                      <p className="font-medium">{error.adapterName}</p>
                      <p>{error.message}</p>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>

          <DialogFooter>
            <Button variant="outline" onClick={onCancel}>
              Cancel
            </Button>
            <Button onClick={onConfirm} disabled={previewResult.filesWritten.length === 0}>
              {conflictFiles.length > 0
                ? `Sync & Resolve ${conflictFiles.length} Conflicts`
                : "Sync Now"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <ConflictResolutionDialog
        open={conflictDialogOpen}
        onOpenChange={setConflictDialogOpen}
        conflict={selectedConflict}
        localRules={rules}
        onResolved={handleConflictResolved}
      />
    </>
  );
}
