import { useState, useEffect, useMemo } from "react";
import { Sparkles, Loader2, Check, X } from "lucide-react";
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

interface AiImproveRuleDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  ruleContent: string;
  ruleName?: string;
  onApply: (improvedContent: string) => void;
}

function computeDiffLines(original: string, improved: string) {
  const originalLines = original.split("\n");
  const improvedLines = improved.split("\n");

  const diffLines: Array<{
    type: "context" | "removed" | "added";
    original?: string;
    improved?: string;
    lineNum: number;
  }> = [];

  let origIdx = 0;
  let impIdx = 0;

  while (origIdx < originalLines.length || impIdx < improvedLines.length) {
    const origLine = originalLines[origIdx];
    const impLine = improvedLines[impIdx];

    if (origIdx >= originalLines.length) {
      diffLines.push({ type: "added", improved: impLine, lineNum: impIdx + 1 });
      impIdx++;
    } else if (impIdx >= improvedLines.length) {
      diffLines.push({ type: "removed", original: origLine, lineNum: origIdx + 1 });
      origIdx++;
    } else if (origLine === impLine) {
      diffLines.push({
        type: "context",
        original: origLine,
        improved: impLine,
        lineNum: origIdx + 1,
      });
      origIdx++;
      impIdx++;
    } else {
      const origInFuture = improvedLines.slice(impIdx + 1).includes(origLine);
      const impInFuture = originalLines.slice(origIdx + 1).includes(impLine);

      if (!origInFuture && impInFuture) {
        diffLines.push({ type: "added", improved: impLine, lineNum: impIdx + 1 });
        impIdx++;
      } else if (origInFuture && !impInFuture) {
        diffLines.push({ type: "removed", original: origLine, lineNum: origIdx + 1 });
        origIdx++;
      } else {
        diffLines.push({ type: "removed", original: origLine, lineNum: origIdx + 1 });
        diffLines.push({ type: "added", improved: impLine, lineNum: impIdx + 1 });
        origIdx++;
        impIdx++;
      }
    }
  }

  return diffLines;
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
    if (open && ruleContent) {
      improve(ruleContent, ruleName);
    }
    if (!open) {
      clearResult();
      setViewMode("diff");
    }
  }, [open, ruleContent, ruleName, improve, clearResult]);

  const diffLines = useMemo(() => {
    if (!improvedContent) return [];
    return computeDiffLines(ruleContent, improvedContent);
  }, [ruleContent, improvedContent]);

  const handleApply = () => {
    if (improvedContent) {
      onApply(improvedContent);
      addToast({
        title: "Improvement Applied",
        description: "The improved content has been applied to the rule",
        variant: "success",
      });
      onOpenChange(false);
    }
  };

  const hasChanges = improvedContent && improvedContent !== ruleContent;
  const changeCount = diffLines.filter((l) => l.type === "added" || l.type === "removed").length;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-4xl max-h-[85vh] flex flex-col">
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
                        <span className="w-12 px-2 py-0.5 text-right text-muted-foreground/50 border-r border-white/5 select-none">
                          {line.type !== "added" ? line.lineNum : ""}
                        </span>
                        <span className="w-12 px-2 py-0.5 text-right text-muted-foreground/50 border-r border-white/5 select-none">
                          {line.type !== "removed" ? line.lineNum : ""}
                        </span>
                        <span
                          className={`px-2 py-0.5 whitespace-pre ${
                            line.type === "removed"
                              ? "text-red-400"
                              : line.type === "added"
                                ? "text-green-400"
                                : "text-muted-foreground"
                          }`}
                        >
                          {line.type === "removed"
                            ? `- ${line.original}`
                            : line.type === "added"
                              ? `+ ${line.improved}`
                              : `  ${line.original}`}
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
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            <X className="mr-2 h-4 w-4" />
            Cancel
          </Button>
          <Button onClick={handleApply} disabled={!hasChanges || isImproving}>
            <Check className="mr-2 h-4 w-4" />
            Apply Changes
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
