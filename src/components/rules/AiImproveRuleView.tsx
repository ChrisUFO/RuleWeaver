import { useState, useEffect, useMemo, useCallback, useRef } from "react";
import { Sparkles, Loader2, Check, X, RefreshCw, AlertTriangle, Send, Layout } from "lucide-react";
import ReactDiffViewer, { DiffMethod } from "react-diff-viewer-continued";
import { Button } from "@/components/ui/button";
import { useToast } from "@/components/ui/toast";
import { useAiImprovement } from "@/hooks/useAiImprovement";
import { useKeyboardShortcuts, SHORTCUTS } from "@/hooks/useKeyboardShortcuts";
import { AI_VALIDATION, getRuleContentSizeWarning } from "@/types/ai";
import { cn } from "@/lib/utils";

interface AiImproveRuleViewProps {
  ruleContent: string;
  ruleName?: string;
  onApply: (improvedContent: string) => void;
  onCancel: () => void;
}

export function AiImproveRuleView({
  ruleContent,
  ruleName,
  onApply,
  onCancel,
}: AiImproveRuleViewProps) {
  const { addToast } = useToast();
  const [viewMode, setViewMode] = useState<"diff" | "original" | "improved">("diff");
  const [splitView, setSplitView] = useState(true);
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
    if (ruleContent && !isContentTooLarge && !hasRequestedRef.current) {
      hasRequestedRef.current = true;
      improve(ruleContent, ruleName);
    }
  }, [ruleContent, ruleName, isContentTooLarge, improve]);

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

  const handleAccept = useCallback(() => {
    if (improvedContent) {
      onApply(improvedContent);
      addToast({
        title: "Improvement Applied",
        description: "The improved content has been applied to the rule",
        variant: "success",
      });
    }
  }, [improvedContent, onApply, addToast]);

  useKeyboardShortcuts({
    shortcuts: [
      { ...SHORTCUTS.ESCAPE, action: onCancel },
      ...(hasChanges && !isImproving ? [{ ...SHORTCUTS.SAVE, action: handleAccept }] : []),
    ],
  });

  return (
    <div className="fixed inset-0 z-[150] bg-background/95 backdrop-blur-md flex flex-col animate-in fade-in zoom-in-95 duration-300">
      {/* Header */}
      <header className="flex items-center justify-between px-6 py-4 border-b border-white/10 bg-muted/30">
        <div className="flex items-center gap-4">
          <div className="p-2 bg-primary/10 rounded-lg">
            <Sparkles className="h-5 w-5 text-primary" />
          </div>
          <div>
            <h1 className="text-lg font-semibold flex items-center gap-2">
              Improve Rule with AI
              {ruleName && <span className="text-muted-foreground font-normal">/ {ruleName}</span>}
            </h1>
            <p className="text-xs text-muted-foreground">
              {isImproving
                ? "Analyzing and improving your rule..."
                : hasChanges
                  ? "Review changes and apply to your rule"
                  : "Scanning for potential improvements..."}
              {modelUsed && !isImproving && (
                <span className="ml-2 text-primary/60">via {modelUsed}</span>
              )}
            </p>
          </div>
        </div>

        <div className="flex items-center gap-3">
          {!isImproving && improvedContent && (
            <div className="flex items-center gap-1 p-1 glass border border-white/5 rounded-md mr-4">
              <Button
                variant={viewMode === "diff" ? "default" : "ghost"}
                size="sm"
                onClick={() => setViewMode("diff")}
                className="h-8 px-3 text-xs"
              >
                Diff
              </Button>
              <Button
                variant={viewMode === "original" ? "default" : "ghost"}
                size="sm"
                onClick={() => setViewMode("original")}
                className="h-8 px-3 text-xs"
              >
                Original
              </Button>
              <Button
                variant={viewMode === "improved" ? "default" : "ghost"}
                size="sm"
                onClick={() => setViewMode("improved")}
                className="h-8 px-3 text-xs"
              >
                Improved
              </Button>
            </div>
          )}

          <Button variant="ghost" onClick={onCancel} className="hover:bg-white/5">
            <X className="mr-2 h-4 w-4" />
            Cancel
          </Button>
          <Button
            onClick={handleAccept}
            disabled={!hasChanges || isImproving}
            className="glow-primary px-6"
          >
            <Check className="mr-2 h-4 w-4" />
            Apply Changes
          </Button>
        </div>
      </header>

      {/* Content Area */}
      <main className="flex-1 relative min-h-0 flex flex-col bg-muted/10">
        {error && (
          <div className="m-6 p-4 rounded-lg border border-destructive/40 bg-destructive/10 text-sm text-destructive flex items-center gap-3">
            <AlertTriangle className="h-5 w-5 flex-shrink-0" />
            {error}
          </div>
        )}

        {contentSizeWarning && !error && !improvedContent && (
          <div className="m-6 p-4 rounded-lg border border-amber-500/30 bg-amber-500/10 text-sm text-amber-200 flex items-start gap-3">
            <AlertTriangle className="h-5 w-5 mt-0.5 flex-shrink-0 text-amber-500" />
            <span>{contentSizeWarning}</span>
          </div>
        )}

        {isImproving && (
          <div className="flex-1 flex flex-col items-center justify-center animate-pulse">
            <div className="relative mb-6">
              <div className="absolute inset-x-0 bottom-0 top-0 bg-primary/20 blur-2xl rounded-full" />
              <Loader2 className="h-12 w-12 animate-spin text-primary relative z-10" />
            </div>
            <span className="text-lg font-medium text-muted-foreground">Thinking...</span>
            <p className="text-sm text-muted-foreground/60 mt-2">
              Updating your rule with AI magic
            </p>
          </div>
        )}

        {!isImproving && improvedContent && (
          <div className="flex-1 flex flex-col min-h-0">
            {viewMode === "diff" && (
              <div className="flex-1 overflow-auto bg-black/40 font-mono text-sm relative">
                <div className="absolute top-4 right-10 z-10">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setSplitView(!splitView)}
                    className="glass border-white/10 hover:bg-white/20 h-8 gap-2 premium-shadow transition-all"
                  >
                    <Layout className="h-3.5 w-3.5" />
                    {splitView ? "Unified View" : "Split View"}
                  </Button>
                </div>
                <ReactDiffViewer
                  oldValue={ruleContent}
                  newValue={improvedContent || ""}
                  splitView={splitView}
                  compareMethod={DiffMethod.WORDS}
                  useDarkTheme={true}
                  styles={{
                    variables: {
                      dark: {
                        diffViewerBackground: "transparent",
                        diffViewerTitleBackground: "transparent",
                        addedBackground: "rgba(34, 197, 94, 0.15)",
                        addedColor: "#4ade80",
                        removedBackground: "rgba(239, 68, 68, 0.15)",
                        removedColor: "#f87171",
                        wordAddedBackground: "rgba(34, 197, 94, 0.4)",
                        wordRemovedBackground: "rgba(239, 68, 68, 0.4)",
                        addedGutterBackground: "rgba(34, 197, 94, 0.08)",
                        removedGutterBackground: "rgba(239, 68, 68, 0.08)",
                        emptyLineBackground: "transparent",
                        gutterColor: "rgba(255, 255, 255, 0.2)",
                        codeFoldBackground: "rgba(255, 255, 255, 0.05)",
                        codeFoldContentColor: "rgba(255, 255, 255, 0.4)",
                      },
                    },
                    line: {
                      fontFamily: "ui-monospace, monospace",
                      fontSize: "13px",
                      lineHeight: "1.6",
                    },
                    contentText: {
                      wordBreak: "break-all",
                    },
                  }}
                />
              </div>
            )}

            {viewMode === "original" && (
              <div className="flex-1 overflow-auto p-8">
                <div className="max-w-4xl mx-auto glass-card p-6">
                  <pre className="whitespace-pre-wrap text-muted-foreground font-mono text-sm leading-relaxed">
                    {ruleContent}
                  </pre>
                </div>
              </div>
            )}

            {viewMode === "improved" && (
              <div className="flex-1 overflow-auto p-8">
                <div className="max-w-4xl mx-auto glass-card p-6">
                  <pre className="whitespace-pre-wrap text-foreground font-mono text-sm leading-relaxed">
                    {improvedContent}
                  </pre>
                </div>
              </div>
            )}
          </div>
        )}

        {!isImproving && !improvedContent && !error && (
          <div className="flex-1 flex items-center justify-center">
            <span className="text-muted-foreground">Waiting for AI response...</span>
          </div>
        )}
      </main>

      {/* Footer / AI Toolbar */}
      <footer className="px-6 py-4 border-t border-white/10 bg-muted/30">
        <div className="max-w-3xl mx-auto w-full flex items-center justify-center">
          {improvedContent && !isImproving && !showRegenerateInput && (
            <Button
              variant="outline"
              onClick={handleRegenerateClick}
              disabled={isImproving}
              className="glass border-white/10 hover:bg-white/20 hover:border-primary/50 h-12 w-full rounded-full transition-all premium-shadow group relative overflow-hidden"
            >
              <div className="absolute inset-0 bg-primary/5 opacity-0 group-hover:opacity-100 transition-opacity" />
              <RefreshCw className="mr-2 h-4 w-4 transition-transform group-hover:rotate-180 duration-700" />
              <span className="relative z-10 font-medium">
                Not quite right? Regenerate with specific instructions...
              </span>
            </Button>
          )}

          {showRegenerateInput && (
            <div className="flex items-center gap-3 w-full animate-in slide-in-from-bottom-2 duration-300">
              <div className="relative flex-1">
                <input
                  type="text"
                  placeholder="e.g., 'Make it more professional', 'Add a section about variable naming', 'Be more concise'..."
                  value={regenerateInstructions}
                  onChange={(e) => setRegenerateInstructions(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter") {
                      handleRegenerateWithInstructions();
                    } else if (e.key === "Escape") {
                      handleCancelRegenerate();
                    }
                  }}
                  className="w-full pl-4 pr-12 py-3 bg-black/40 border border-white/10 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary/50 text-sm transition-all"
                  autoFocus
                />
                <Button
                  size="icon"
                  variant="ghost"
                  onClick={handleRegenerateWithInstructions}
                  className={cn(
                    "absolute right-1.5 top-1.5 h-8 w-8 rounded-lg transition-colors",
                    regenerateInstructions.trim()
                      ? "text-primary hover:bg-primary/20"
                      : "text-muted-foreground"
                  )}
                  title="Send instructions"
                >
                  <Send className="h-4 w-4" />
                </Button>
              </div>
              <Button
                variant="ghost"
                size="sm"
                onClick={handleCancelRegenerate}
                className="text-muted-foreground hover:text-foreground h-10 px-4 rounded-xl"
              >
                Cancel
              </Button>
            </div>
          )}
        </div>
      </footer>
    </div>
  );
}
