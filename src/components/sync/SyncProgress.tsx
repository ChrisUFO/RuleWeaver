import { CheckCircle, XCircle, Loader2 } from "lucide-react";
import { cn } from "@/lib/utils";

interface SyncProgressProps {
  isSyncing: boolean;
  currentFile: string;
  currentFileIndex: number;
  totalFiles: number;
  completedFiles: { path: string; success: boolean }[];
}

export function SyncProgress({
  isSyncing,
  currentFile,
  currentFileIndex,
  totalFiles,
  completedFiles,
}: SyncProgressProps) {
  if (!isSyncing) return null;

  const progress = totalFiles > 0 ? (currentFileIndex / totalFiles) * 100 : 0;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div className="bg-card rounded-lg shadow-lg p-6 w-full max-w-md mx-4">
        <div className="flex items-center gap-3 mb-4">
          <Loader2 className="h-5 w-5 animate-spin text-primary" />
          <h3 className="font-semibold">Syncing Rules...</h3>
        </div>

        <div className="space-y-4">
          <div>
            <div className="flex justify-between text-sm mb-2">
              <span className="text-muted-foreground">Progress</span>
              <span className="font-medium">
                {currentFileIndex} of {totalFiles} files
              </span>
            </div>
            <div className="h-2 bg-muted rounded-full overflow-hidden">
              <div
                className="h-full bg-primary transition-all duration-300"
                style={{ width: `${progress}%` }}
              />
            </div>
          </div>

          {currentFile && (
            <div className="text-sm">
              <span className="text-muted-foreground">Current: </span>
              <span className="font-mono text-xs bg-muted px-2 py-0.5 rounded">
                {currentFile.split("/").pop()}
              </span>
            </div>
          )}

          {completedFiles.length > 0 && (
            <div className="max-h-32 overflow-auto space-y-1">
              {completedFiles.map((file, index) => (
                <div
                  key={index}
                  className={cn(
                    "flex items-center gap-2 text-xs",
                    file.success ? "text-success" : "text-destructive"
                  )}
                >
                  {file.success ? (
                    <CheckCircle className="h-3 w-3" />
                  ) : (
                    <XCircle className="h-3 w-3" />
                  )}
                  <span className="truncate">{file.path.split("/").pop()}</span>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
