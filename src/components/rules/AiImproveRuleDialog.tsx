import { useState, useEffect, useMemo, useCallback, useRef } from "react";
import { Sparkles, Loader2, Check, X, RefreshCw, AlertTriangle } from "lucide-react";
import * as Diff from "diff";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { useToast } from "@/components/ui/toast";
import { useAiImprovement } from "@/hooks/useAiImprovement";
import { useKeyboardShortcuts, SHORTCUTS } from "@/hooks/useKeyboardShortcuts";
import { AI_VALIDATION } from "@/types/ai";

const getRuleContentSizeWarning = (contentLength: number): string | null => {
  if (contentLength > AI_VALIDATION.MAX_RULE_CONTENT_LENGTH) {
    return `Rule content exceeds ${Math.round(AI_VALIDATION.MAX_RULE_CONTENT_LENGTH / 1000)}k characters and may be too long for some AI models.`;
  }
  if (contentLength > AI_VALIDATION.LARGE_CONTENT_WARNING_THRESHOLD) {
    return `Rule content is large (${Math.round(contentLength / 1000)}k chars). Processing may take longer.`;
  }
  return null;
};

interface AiImproveRuleDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  ruleContent: string;
  ruleName?: string;
  onApply: (improvedContent: string) => void;
}

type DiffPart = {
  type: "context" | "removed" | "added";
  value: string;
  added?: boolean;
  removed?: boolean;
};

type DiffLine = {
  type: "context" | "removed" | "added";
  content: string;
  origLineNum?: number;
  newLineNum?: number;
  charDiff?: DiffPart[];
};

function computeDiffLines(original: string, improved: string): DiffLine[] {
  const changes = Diff.diffLines(original, improved);
  const result: DiffLine[] = [];
  let origLineNum = 0;
  let newLineNum = 0;

  for (let i = 0; i < changes.length; i++) {
    const change = changes[i];
    const nextChange = changes[i + 1];
    const lines = change.value.replace(/\n$/, "").split("\n");

    if (change.added) {
      for (const line of lines) {
        newLineNum++;
        result.push({
          type: "added",
          content: line,
          newLineNum,
        });
      }
    } else if (change.removed) {
      if (nextChange?.added) {
        // Here we have a combined edit block: some lines removed, some added.
        // Instead of zipping them line-by-line, we output ALL removed lines first,
        // then ALL added lines. We still want word-level diffing if possible.

        const removedLines = lines;
        const addedLines = nextChange.value.replace(/\n$/, "").split("\n");

        // 1. Output all removed lines
        for (let j = 0; j < removedLines.length; j++) {
          origLineNum++;
          const remLine = removedLines[j];
          // Try to find a corresponding added line for word-diffing
          const addLine = addedLines[j];
          const charDiff = addLine !== undefined ? Diff.diffWords(remLine, addLine) : undefined;

          result.push({
            type: "removed",
            content: remLine,
            origLineNum,
            charDiff: charDiff?.map((part) => ({
              type: part.added ? "added" : part.removed ? "removed" : "context",
              value: part.value,
              added: part.added,
              removed: part.removed,
            })),
          });
        }

        // 2. Output all added lines
        for (let j = 0; j < addedLines.length; j++) {
          newLineNum++;
          const addLine = addedLines[j];
          // Try to find a corresponding removed line for word-diffing
          const remLine = removedLines[j];
          const charDiff = remLine !== undefined ? Diff.diffWords(remLine, addLine) : undefined;

          result.push({
            type: "added",
            content: addLine,
            newLineNum,
            charDiff: charDiff?.map((part) => ({
              type: part.added ? "added" : part.removed ? "removed" : "context",
              value: part.value,
              added: part.added,
              removed: part.removed,
            })),
          });
        }

        i++; // Skip the next 'added' change since we processed it here
      } else {
        // Just removed lines
        for (const line of lines) {
          origLineNum++;
          result.push({
            type: "removed",
            content: line,
            origLineNum,
          });
        }
      }
    } else {
      // Unchanged lines (context)
      for (const line of lines) {
        origLineNum++;
        newLineNum++;
        result.push({
          type: "context",
          content: line,
          origLineNum,
          newLineNum,
        });
      }
    }
  }

  return result;
}

export function AiImproveRuleDialog({
  open,
  onOpenChange,
  ruleContent,
  ruleName,
  onApply,
}: AiImproveRuleDialogProps) {
  const { addToast } = useToast();
  const [viewMode, setViewMode] = useState<"diff" | "original" | "improved">("diff");
  const hasRequestedRef = useRef(false);

  const contentSizeWarning = useMemo(
    () => getRuleContentSizeWarning(ruleContent.length),
    [ruleContent]
  );
  const isContentTooLarge = ruleContent.length > AI_VALIDATION.MAX_RULE_CONTENT_LENGTH;

  const { isImproving, improvedContent, modelUsed, error, improve, clearResult } = useAiImprovement(
    {
      onError: (err) => {
        addToast({
          title: "AI Improvement Failed",
          description: err instanceof Error ? err.message : "Unknown error",
          variant: "error",
        });
      },
    }
  );

  useEffect(() => {
    if (open && ruleContent && !isContentTooLarge && !hasRequestedRef.current) {
      hasRequestedRef.current = true;
      improve(ruleContent, ruleName);
    }
    if (!open) {
      hasRequestedRef.current = false;
      clearResult();
      setViewMode("diff");
    }
  }, [open, ruleContent, ruleName, isContentTooLarge, improve, clearResult]);

  const diffLines = useMemo(() => {
    if (!improvedContent) return [];
    return computeDiffLines(ruleContent, improvedContent);
  }, [ruleContent, improvedContent]);

  const hasChanges = improvedContent && improvedContent !== ruleContent;
  const changeCount = diffLines.filter((l) => l.type === "added" || l.type === "removed").length;

  const handleRegenerate = useCallback(() => {
    clearResult();
    improve(ruleContent, ruleName);
  }, [clearResult, improve, ruleContent, ruleName]);

  const handleReject = useCallback(() => {
    onOpenChange(false);
  }, [onOpenChange]);

  const handleAccept = useCallback(() => {
    if (improvedContent) {
      onApply(improvedContent);
      addToast({
        title: "Improvement Applied",
        description: "The improved content has been applied to the rule",
        variant: "success",
      });
      onOpenChange(false);
    }
  }, [improvedContent, onApply, addToast, onOpenChange]);

  useKeyboardShortcuts({
    shortcuts: [
      { ...SHORTCUTS.ESCAPE, action: handleReject },
      ...(hasChanges && !isImproving ? [{ ...SHORTCUTS.SAVE, action: handleAccept }] : []),
    ],
    enabled: open,
  });

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-6xl w-[90vw] max-h-[90vh] flex flex-col">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Sparkles className="h-5 w-5 text-primary" />
            Improve Rule with AI
          </DialogTitle>
          <DialogDescription>
            {isImproving
              ? "Analyzing and improving your rule..."
              : hasChanges
                ? `Found ${changeCount} changes to apply`
                : "No improvements found"}
            {modelUsed && !isImproving && (
              <span className="ml-2 text-xs text-muted-foreground">via {modelUsed}</span>
            )}
          </DialogDescription>
        </DialogHeader>

        <div className="flex-1 min-h-0 overflow-hidden flex flex-col">
          {error && (
            <div className="rounded-lg border border-destructive/40 bg-destructive/10 p-3 mb-3 text-sm text-destructive">
              {error}
            </div>
          )}

          {contentSizeWarning && !error && !improvedContent && (
            <div className="rounded-lg border border-amber-500/30 bg-amber-500/10 p-3 mb-3 text-sm text-amber-200 flex items-start gap-2">
              <AlertTriangle className="h-4 w-4 mt-0.5 flex-shrink-0" />
              <span>{contentSizeWarning}</span>
            </div>
          )}

          {isImproving && (
            <div className="flex-1 flex items-center justify-center">
              <div className="flex flex-col items-center gap-3">
                <Loader2 className="h-8 w-8 animate-spin text-primary" />
                <span className="text-muted-foreground">Analyzing your rule...</span>
              </div>
            </div>
          )}

          {!isImproving && improvedContent && (
            <>
              <div className="flex items-center gap-2 mb-3">
                <Button
                  variant={viewMode === "diff" ? "default" : "outline"}
                  size="sm"
                  onClick={() => setViewMode("diff")}
                >
                  Diff
                </Button>
                <Button
                  variant={viewMode === "original" ? "default" : "outline"}
                  size="sm"
                  onClick={() => setViewMode("original")}
                >
                  Original
                </Button>
                <Button
                  variant={viewMode === "improved" ? "default" : "outline"}
                  size="sm"
                  onClick={() => setViewMode("improved")}
                >
                  Improved
                </Button>
              </div>

              <div className="flex-1 overflow-auto rounded-lg border border-white/10 bg-black/20 font-mono text-xs">
                {viewMode === "diff" && (
                  <div className="divide-y divide-white/5">
                    {diffLines.map((line, idx) => (
                      <div
                        key={idx}
                        className={`flex ${
                          line.type === "removed"
                            ? "bg-red-500/10"
                            : line.type === "added"
                              ? "bg-green-500/10"
                              : ""
                        }`}
                      >
                        <span className="w-10 shrink-0 px-2 py-0.5 text-right text-muted-foreground/50 border-r border-white/5 select-none">
                          {line.origLineNum ?? ""}
                        </span>
                        <span className="w-10 shrink-0 px-2 py-0.5 text-right text-muted-foreground/50 border-r border-white/5 select-none">
                          {line.newLineNum ?? ""}
                        </span>
                        <span
                          className={`px-2 py-0.5 whitespace-pre-wrap break-words ${
                            line.type === "removed"
                              ? "text-red-400"
                              : line.type === "added"
                                ? "text-green-400"
                                : "text-muted-foreground"
                          }`}
                        >
                          {line.type === "removed" ? "- " : line.type === "added" ? "+ " : "  "}
                          {line.charDiff
                            ? line.charDiff.map((part, pidx) => (
                                <span
                                  key={pidx}
                                  className={
                                    part.added
                                      ? "bg-green-500/30 text-green-300"
                                      : part.removed
                                        ? "bg-red-500/30 text-red-300 line-through"
                                        : ""
                                  }
                                >
                                  {part.value}
                                </span>
                              ))
                            : line.content}
                        </span>
                      </div>
                    ))}
                  </div>
                )}

                {viewMode === "original" && (
                  <pre className="p-3 whitespace-pre-wrap text-muted-foreground">{ruleContent}</pre>
                )}

                {viewMode === "improved" && (
                  <pre className="p-3 whitespace-pre-wrap text-foreground">{improvedContent}</pre>
                )}
              </div>
            </>
          )}

          {!isImproving && !improvedContent && !error && (
            <div className="flex-1 flex items-center justify-center">
              <span className="text-muted-foreground">Waiting for AI response...</span>
            </div>
          )}
        </div>

        <DialogFooter>
          {improvedContent && !isImproving && (
            <Button
              variant="outline"
              onClick={handleRegenerate}
              disabled={isImproving}
              title="Try again with the same input"
            >
              <RefreshCw className="mr-2 h-4 w-4" />
              Regenerate
            </Button>
          )}
          <Button variant="outline" onClick={handleReject}>
            <X className="mr-2 h-4 w-4" />
            Cancel
          </Button>
          <Button onClick={handleAccept} disabled={!hasChanges || isImproving}>
            <Check className="mr-2 h-4 w-4" />
            Apply Changes
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
