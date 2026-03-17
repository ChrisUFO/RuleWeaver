import { useState, useEffect, useMemo, useCallback, useRef } from "react";
import { Sparkles, Loader2, Check, X, RefreshCw, AlertTriangle, Send } from "lucide-react";
import ReactDiffViewer, { DiffMethod } from "react-diff-viewer-continued";
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

export function AiImproveRuleDialog({
  open,
  onOpenChange,
  ruleContent,
  ruleName,
  onApply,
}: AiImproveRuleDialogProps) {
  const { addToast } = useToast();
  const [viewMode, setViewMode] = useState<"diff" | "original" | "improved">("diff");
  const [showRegenerateInput, setShowRegenerateInput] = useState(false);
  const [regenerateInstructions, setRegenerateInstructions] = useState("");

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
      setShowRegenerateInput(false);
      setRegenerateInstructions("");
    }
  }, [open, ruleContent, ruleName, isContentTooLarge, improve, clearResult]);

  const hasChanges = improvedContent && improvedContent !== ruleContent;

  const handleRegenerateClick = useCallback(() => {
    setShowRegenerateInput(true);
  }, []);

  const handleRegenerateWithInstructions = useCallback(() => {
    clearResult();
    improve(ruleContent, ruleName, regenerateInstructions || undefined);
    setShowRegenerateInput(false);
    setRegenerateInstructions("");
  }, [clearResult, improve, ruleContent, ruleName, regenerateInstructions]);

  const handleCancelRegenerate = useCallback(() => {
    setShowRegenerateInput(false);
    setRegenerateInstructions("");
  }, []);

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
                ? "Found improvements to apply"
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

              <div className="flex-1 overflow-auto rounded-lg border border-white/10 bg-black/20 font-mono text-sm">
                {viewMode === "diff" && (
                  <ReactDiffViewer
                    oldValue={ruleContent}
                    newValue={improvedContent || ""}
                    splitView={true}
                    compareMethod={DiffMethod.WORDS}
                    useDarkTheme={true}
                    styles={{
                      variables: {
                        dark: {
                          diffViewerBackground: "transparent",
                          diffViewerTitleBackground: "transparent",
                          addedBackground: "rgba(34, 197, 94, 0.1)",
                          addedColor: "#4ade80",
                          removedBackground: "rgba(239, 68, 68, 0.1)",
                          removedColor: "#f87171",
                          wordAddedBackground: "rgba(34, 197, 94, 0.3)",
                          wordRemovedBackground: "rgba(239, 68, 68, 0.3)",
                          addedGutterBackground: "rgba(34, 197, 94, 0.05)",
                          removedGutterBackground: "rgba(239, 68, 68, 0.05)",
                          emptyLineBackground: "transparent",
                        },
                      },
                      line: {
                        fontFamily: "inherit",
                      },
                    }}
                  />
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
          {improvedContent && !isImproving && !showRegenerateInput && (
            <Button
              variant="outline"
              onClick={handleRegenerateClick}
              disabled={isImproving}
              title="Regenerate with optional instructions"
            >
              <RefreshCw className="mr-2 h-4 w-4" />
              Regenerate
            </Button>
          )}
          {showRegenerateInput && (
            <div className="flex items-center gap-2 flex-1 mr-auto">
              <input
                type="text"
                placeholder="e.g., 'Make it more concise' or 'Add error handling'"
                value={regenerateInstructions}
                onChange={(e) => setRegenerateInstructions(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") {
                    handleRegenerateWithInstructions();
                  } else if (e.key === "Escape") {
                    handleCancelRegenerate();
                  }
                }}
                className="flex-1 px-3 py-1.5 text-sm bg-background border border-border rounded-md focus:outline-none focus:ring-2 focus:ring-primary"
                autoFocus
              />
              <Button
                size="sm"
                onClick={handleRegenerateWithInstructions}
                title="Send instructions"
              >
                <Send className="h-4 w-4" />
              </Button>
              <Button size="sm" variant="ghost" onClick={handleCancelRegenerate} title="Cancel">
                <X className="h-4 w-4" />
              </Button>
            </div>
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
