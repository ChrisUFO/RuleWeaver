import { CheckCircle, AlertTriangle, XCircle, FileText, ExternalLink } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import type { SyncResult } from "@/types/rule";
import { api } from "@/lib/tauri";

interface SyncResultsDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  result: SyncResult | null;
}

export function SyncResultsDialog({ open, onOpenChange, result }: SyncResultsDialogProps) {
  if (!result) return null;

  const successCount = result.filesWritten.length;
  const conflictCount = result.conflicts.length;
  const errorCount = result.errors.length;

  const handleOpenFolder = async (filePath: string) => {
    const dirPath = filePath.substring(0, filePath.lastIndexOf("/"));
    try {
      await api.app.openInExplorer(dirPath);
    } catch {
      console.error("Failed to open folder");
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-lg" onClose={() => onOpenChange(false)}>
        <DialogHeader>
          <DialogTitle>
            {result.success ? "Sync Complete" : "Sync Completed with Issues"}
          </DialogTitle>
        </DialogHeader>

        <div className="py-4 space-y-4">
          <div className="flex gap-4">
            {successCount > 0 && (
              <div className="flex items-center gap-2 text-success">
                <CheckCircle className="h-5 w-5" />
                <span className="font-medium">{successCount} synced</span>
              </div>
            )}
            {conflictCount > 0 && (
              <div className="flex items-center gap-2 text-warning">
                <AlertTriangle className="h-5 w-5" />
                <span className="font-medium">{conflictCount} conflicts</span>
              </div>
            )}
            {errorCount > 0 && (
              <div className="flex items-center gap-2 text-destructive">
                <XCircle className="h-5 w-5" />
                <span className="font-medium">{errorCount} errors</span>
              </div>
            )}
          </div>

          {result.filesWritten.length > 0 && (
            <div className="space-y-2">
              <h4 className="text-sm font-medium">Files Updated</h4>
              <div className="max-h-40 overflow-auto space-y-1">
                {result.filesWritten.map((filePath) => (
                  <div
                    key={filePath}
                    className="flex items-center justify-between p-2 rounded-md bg-muted/30 text-sm"
                  >
                    <div className="flex items-center gap-2 min-w-0">
                      <FileText className="h-4 w-4 shrink-0 text-muted-foreground" />
                      <span className="truncate">{filePath}</span>
                    </div>
                    <Button
                      variant="ghost"
                      size="icon"
                      className="h-6 w-6 shrink-0"
                      onClick={() => handleOpenFolder(filePath)}
                    >
                      <ExternalLink className="h-3 w-3" />
                    </Button>
                  </div>
                ))}
              </div>
            </div>
          )}

          {result.errors.length > 0 && (
            <div className="space-y-2">
              <h4 className="text-sm font-medium text-destructive">Errors</h4>
              <div className="max-h-32 overflow-auto space-y-1">
                {result.errors.map((error, index) => (
                  <div key={index} className="p-2 rounded-md bg-destructive/10 text-sm">
                    <p className="font-medium">{error.adapterName}</p>
                    <p className="text-muted-foreground">{error.message}</p>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>

        <DialogFooter>
          <Button onClick={() => onOpenChange(false)}>Done</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
