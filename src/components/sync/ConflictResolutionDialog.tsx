import { useState, useEffect } from "react";
import { AlertTriangle, RefreshCw, FileText, ChevronDown, ChevronUp } from "lucide-react";
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
import { useToast } from "@/components/ui/toast";
import { api } from "@/lib/tauri";
import type { Conflict, Rule, AdapterType } from "@/types/rule";

const DIFF_PREVIEW_LINES = 50;

const ADAPTER_NAME_TO_ID: Record<string, AdapterType> = {
  antigravity: "antigravity",
  "gemini cli": "gemini",
  opencode: "opencode",
  cline: "cline",
  "claude code": "claude-code",
  codex: "codex",
};

interface ConflictResolutionDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  conflict: Conflict | null;
  localRules: Rule[];
  onResolved: () => void;
}

interface DiffLine {
  type: "context" | "added" | "removed";
  content: string;
  lineNumber: { local?: number; remote?: number };
}

function computeDiff(localContent: string, remoteContent: string): DiffLine[] {
  const localLines = localContent.split("\n");
  const remoteLines = remoteContent.split("\n");
  const diff: DiffLine[] = [];

  const maxLines = Math.max(localLines.length, remoteLines.length);
  let localIdx = 0;
  let remoteIdx = 0;

  for (let i = 0; i < maxLines * 2; i++) {
    if (localIdx < localLines.length && remoteIdx < remoteLines.length) {
      if (localLines[localIdx] === remoteLines[remoteIdx]) {
        diff.push({
          type: "context",
          content: localLines[localIdx],
          lineNumber: { local: localIdx + 1, remote: remoteIdx + 1 },
        });
        localIdx++;
        remoteIdx++;
      } else {
        const localRemaining = localLines.length - localIdx;
        const remoteRemaining = remoteLines.length - remoteIdx;

        if (localRemaining > remoteRemaining) {
          diff.push({
            type: "removed",
            content: localLines[localIdx],
            lineNumber: { local: localIdx + 1 },
          });
          localIdx++;
        } else if (remoteRemaining > localRemaining) {
          diff.push({
            type: "added",
            content: remoteLines[remoteIdx],
            lineNumber: { remote: remoteIdx + 1 },
          });
          remoteIdx++;
        } else {
          diff.push({
            type: "removed",
            content: localLines[localIdx],
            lineNumber: { local: localIdx + 1 },
          });
          localIdx++;
          diff.push({
            type: "added",
            content: remoteLines[remoteIdx],
            lineNumber: { remote: remoteIdx + 1 },
          });
          remoteIdx++;
        }
      }
    } else if (localIdx < localLines.length) {
      diff.push({
        type: "removed",
        content: localLines[localIdx],
        lineNumber: { local: localIdx + 1 },
      });
      localIdx++;
    } else if (remoteIdx < remoteLines.length) {
      diff.push({
        type: "added",
        content: remoteLines[remoteIdx],
        lineNumber: { remote: remoteIdx + 1 },
      });
      remoteIdx++;
    }
  }

  return diff;
}

export function ConflictResolutionDialog({
  open,
  onOpenChange,
  conflict,
  localRules,
  onResolved,
}: ConflictResolutionDialogProps) {
  const { addToast } = useToast();
  const [remoteContent, setRemoteContent] = useState<string>("");
  const [localContent, setLocalContent] = useState<string>("");
  const [isLoading, setIsLoading] = useState(true);
  const [resolution, setResolution] = useState<"overwrite" | "keep-remote" | null>(null);
  const [isResolving, setIsResolving] = useState(false);
  const [showFullDiff, setShowFullDiff] = useState(false);

  useEffect(() => {
    if (!conflict) return;

    const loadContent = async () => {
      setIsLoading(true);
      try {
        const remote = await api.sync.readFileContent(conflict.filePath);
        setRemoteContent(remote);

        const adapterName = conflict.adapterName.toLowerCase();
        const adapterId =
          ADAPTER_NAME_TO_ID[adapterName] || (adapterName.replace(" ", "-") as AdapterType);
        const adapterRules = localRules.filter((r) => r.enabledAdapters.includes(adapterId));

        const globalRules = adapterRules.filter((r) => r.scope === "global" && r.enabled);

        const local = generateLocalContent(globalRules);
        setLocalContent(local);
      } catch (error) {
        addToast({
          title: "Error Loading Content",
          description: error instanceof Error ? error.message : "Unknown error",
          variant: "error",
        });
      } finally {
        setIsLoading(false);
      }
    };

    loadContent();
  }, [conflict, localRules, addToast]);

  const generateLocalContent = (rules: Rule[]): string => {
    const maxTimestamp =
      rules.length > 0
        ? new Date(Math.max(...rules.map((r) => r.updatedAt * 1000))).toISOString()
        : "New Rule";
    const ruleNames = rules.map((r) => r.name).join(", ");

    let content = `<!-- Generated by RuleWeaver - Do not edit manually -->\n<!-- Last synced: ${maxTimestamp} -->\n<!-- Rules: ${ruleNames} -->\n\n`;

    for (const rule of rules) {
      content += `## ${rule.name}\n${rule.content}\n\n`;
    }

    return content;
  };

  const diff = computeDiff(localContent, remoteContent);
  const displayDiff = showFullDiff ? diff : diff.slice(0, DIFF_PREVIEW_LINES);
  const hasMore = diff.length > DIFF_PREVIEW_LINES;

  const handleResolve = async () => {
    if (!conflict || !resolution) return;

    setIsResolving(true);
    try {
      await api.sync.resolveConflict(conflict, resolution);
      addToast({
        title: "Conflict Resolved",
        description:
          resolution === "overwrite"
            ? "Remote file overwritten with local rules"
            : "Kept remote file changes",
        variant: "success",
      });
      onResolved();
      onOpenChange(false);
    } catch (error) {
      addToast({
        title: "Resolution Failed",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "error",
      });
    } finally {
      setIsResolving(false);
    }
  };

  if (!conflict) return null;

  const fileName = conflict.filePath.split(/[/\\]/).pop() || conflict.filePath;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent
        className="max-w-4xl max-h-[85vh] overflow-hidden flex flex-col"
        onClose={() => onOpenChange(false)}
      >
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <AlertTriangle className="h-5 w-5 text-warning" />
            Resolve Conflict
          </DialogTitle>
          <DialogDescription>
            The file "{fileName}" has been modified externally. Choose how to resolve this conflict.
          </DialogDescription>
        </DialogHeader>

        <div className="flex-1 overflow-auto py-4 space-y-4">
          {isLoading ? (
            <div className="flex items-center justify-center py-8">
              <RefreshCw className="h-6 w-6 animate-spin text-muted-foreground" />
            </div>
          ) : (
            <>
              <div className="flex items-center gap-4 text-sm">
                <div className="flex items-center gap-2">
                  <div className="w-3 h-3 rounded bg-destructive/50" />
                  <span>Local (will be written)</span>
                </div>
                <div className="flex items-center gap-2">
                  <div className="w-3 h-3 rounded bg-success/50" />
                  <span>Remote (current file)</span>
                </div>
              </div>

              <div className="rounded-md border overflow-hidden">
                <div className="bg-muted/50 px-3 py-2 text-sm font-medium flex items-center gap-2 border-b">
                  <FileText className="h-4 w-4" />
                  <span className="truncate">{fileName}</span>
                  <Badge variant="outline" className="ml-auto">
                    {diff.filter((d) => d.type !== "context").length} changes
                  </Badge>
                </div>
                <div className="overflow-auto max-h-80 font-mono text-xs">
                  {displayDiff.map((line, idx) => (
                    <div
                      key={idx}
                      className={`flex ${
                        line.type === "removed"
                          ? "bg-destructive/20 text-destructive"
                          : line.type === "added"
                            ? "bg-success/20 text-success"
                            : "bg-background"
                      }`}
                    >
                      <span className="w-10 text-right pr-2 border-r text-muted-foreground select-none">
                        {line.lineNumber.local || ""}
                      </span>
                      <span className="w-10 text-right pr-2 border-r text-muted-foreground select-none">
                        {line.lineNumber.remote || ""}
                      </span>
                      <span className="flex-1 pl-2 whitespace-pre-wrap">
                        {line.type === "removed" ? "-" : line.type === "added" ? "+" : " "}
                        {line.content}
                      </span>
                    </div>
                  ))}
                </div>
                {hasMore && !showFullDiff && (
                  <button
                    className="w-full py-2 text-sm text-muted-foreground hover:bg-accent flex items-center justify-center gap-1"
                    onClick={() => setShowFullDiff(true)}
                  >
                    <ChevronDown className="h-4 w-4" />
                    Show {diff.length - DIFF_PREVIEW_LINES} more lines
                  </button>
                )}
                {showFullDiff && hasMore && (
                  <button
                    className="w-full py-2 text-sm text-muted-foreground hover:bg-accent flex items-center justify-center gap-1"
                    onClick={() => setShowFullDiff(false)}
                  >
                    <ChevronUp className="h-4 w-4" />
                    Show less
                  </button>
                )}
              </div>

              <div className="space-y-3">
                <p className="text-sm font-medium">Choose resolution:</p>
                <div className="grid gap-3">
                  <button
                    className={`flex items-start gap-3 p-3 rounded-md border text-left transition-colors ${
                      resolution === "overwrite"
                        ? "border-primary bg-primary/10"
                        : "hover:bg-accent"
                    }`}
                    onClick={() => setResolution("overwrite")}
                  >
                    <div
                      className={`w-4 h-4 rounded-full border-2 mt-0.5 ${
                        resolution === "overwrite"
                          ? "border-primary bg-primary"
                          : "border-muted-foreground"
                      }`}
                    />
                    <div>
                      <p className="font-medium">Overwrite with Local</p>
                      <p className="text-sm text-muted-foreground">
                        Replace the remote file with your local rules. External changes will be
                        lost.
                      </p>
                    </div>
                  </button>
                  <button
                    className={`flex items-start gap-3 p-3 rounded-md border text-left transition-colors ${
                      resolution === "keep-remote"
                        ? "border-primary bg-primary/10"
                        : "hover:bg-accent"
                    }`}
                    onClick={() => setResolution("keep-remote")}
                  >
                    <div
                      className={`w-4 h-4 rounded-full border-2 mt-0.5 ${
                        resolution === "keep-remote"
                          ? "border-primary bg-primary"
                          : "border-muted-foreground"
                      }`}
                    />
                    <div>
                      <p className="font-medium">Keep Remote Changes</p>
                      <p className="text-sm text-muted-foreground">
                        Accept the external changes. Your local rules for this adapter will not be
                        synced to this file.
                      </p>
                    </div>
                  </button>
                </div>
              </div>
            </>
          )}
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button onClick={handleResolve} disabled={!resolution || isLoading || isResolving}>
            {isResolving ? (
              <>
                <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
                Resolving...
              </>
            ) : (
              "Resolve Conflict"
            )}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
