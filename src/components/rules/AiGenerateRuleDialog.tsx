import { useState, useEffect } from "react";
import { Sparkles, Loader2, Check, X, Wand2 } from "lucide-react";
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
import { useToast } from "@/components/ui/toast";
import { useAiGeneration } from "@/hooks/useAiGeneration";

interface AiGenerateRuleDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onApply: (ruleContent: string, ruleName?: string) => void;
}

export function AiGenerateRuleDialog({ open, onOpenChange, onApply }: AiGenerateRuleDialogProps) {
  const { addToast } = useToast();
  const [description, setDescription] = useState("");
  const [ruleName, setRuleName] = useState("");
  const [context, setContext] = useState("");

  const { isGenerating, generatedContent, suggestedName, modelUsed, error, generate, clearResult } =
    useAiGeneration({
      onError: (err) => {
        addToast({
          title: "AI Generation Failed",
          description: err instanceof Error ? err.message : "Unknown error",
          variant: "error",
        });
      },
    });

  useEffect(() => {
    if (!open) {
      setDescription("");
      setRuleName("");
      setContext("");
      clearResult();
    }
  }, [open, clearResult]);

  useEffect(() => {
    if (suggestedName && !ruleName) {
      setRuleName(suggestedName);
    }
  }, [suggestedName, ruleName]);

  const handleGenerate = async () => {
    if (!description.trim()) {
      addToast({
        title: "Description Required",
        description: "Please describe the rule you want to create",
        variant: "error",
      });
      return;
    }

    await generate(description.trim(), ruleName.trim() || undefined, context.trim() || undefined);
  };

  const handleApply = () => {
    if (generatedContent) {
      onApply(generatedContent, ruleName.trim() || suggestedName || undefined);
      addToast({
        title: "Rule Generated",
        description: "The generated rule has been applied",
        variant: "success",
      });
      onOpenChange(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-3xl max-h-[85vh] flex flex-col">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Wand2 className="h-5 w-5 text-primary" />
            Generate Rule with AI
          </DialogTitle>
          <DialogDescription>
            Describe the rule you want to create and AI will generate it for you
          </DialogDescription>
        </DialogHeader>

        <div className="flex-1 min-h-0 overflow-hidden flex flex-col gap-4">
          {error && (
            <div className="rounded-lg border border-destructive/40 bg-destructive/10 p-3 text-sm text-destructive">
              {error}
            </div>
          )}

          <div className="space-y-2">
            <label className="text-xs font-bold uppercase tracking-wider text-muted-foreground">
              Rule Name (optional)
            </label>
            <Input
              value={ruleName}
              onChange={(e) => setRuleName(e.target.value)}
              placeholder="e.g., TypeScript Best Practices"
              disabled={isGenerating}
            />
          </div>

          <div className="space-y-2">
            <label className="text-xs font-bold uppercase tracking-wider text-muted-foreground">
              Description <span className="text-amber-500">*</span>
            </label>
            <textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="Describe the rule you want to create... Example: 'Enforce that all API endpoints return proper error responses with consistent structure'"
              rows={3}
              disabled={isGenerating}
              className="w-full min-h-[80px] rounded-lg border border-white/10 bg-white/5 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary/50 resize-none"
            />
          </div>

          <div className="space-y-2">
            <label className="text-xs font-bold uppercase tracking-wider text-muted-foreground">
              Additional Context (optional)
            </label>
            <textarea
              value={context}
              onChange={(e) => setContext(e.target.value)}
              placeholder="Any additional context like tech stack, frameworks, coding standards, etc."
              rows={2}
              disabled={isGenerating}
              className="w-full min-h-[48px] rounded-lg border border-white/10 bg-white/5 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary/50 resize-none"
            />
          </div>

          {isGenerating && (
            <div className="flex items-center justify-center py-4">
              <div className="flex flex-col items-center gap-3">
                <Loader2 className="h-8 w-8 animate-spin text-primary" />
                <span className="text-muted-foreground">Generating your rule...</span>
              </div>
            </div>
          )}

          {generatedContent && !isGenerating && (
            <div className="flex-1 min-h-0 overflow-hidden flex flex-col">
              <div className="flex items-center justify-between mb-2">
                <span className="text-xs font-bold uppercase tracking-wider text-muted-foreground">
                  Generated Rule
                </span>
                {modelUsed && (
                  <span className="text-xs text-muted-foreground">via {modelUsed}</span>
                )}
              </div>
              <div className="flex-1 overflow-auto rounded-lg border border-white/10 bg-black/20 p-3 font-mono text-xs whitespace-pre-wrap">
                {generatedContent}
              </div>
            </div>
          )}
        </div>

        <DialogFooter>
          {!generatedContent ? (
            <>
              <Button variant="outline" onClick={() => onOpenChange(false)}>
                <X className="mr-2 h-4 w-4" />
                Cancel
              </Button>
              <Button onClick={handleGenerate} disabled={isGenerating || !description.trim()}>
                {isGenerating ? (
                  <>
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                    Generating...
                  </>
                ) : (
                  <>
                    <Sparkles className="mr-2 h-4 w-4" />
                    Generate
                  </>
                )}
              </Button>
            </>
          ) : (
            <>
              <Button variant="outline" onClick={() => clearResult()}>
                Regenerate
              </Button>
              <Button variant="outline" onClick={() => onOpenChange(false)}>
                Cancel
              </Button>
              <Button onClick={handleApply}>
                <Check className="mr-2 h-4 w-4" />
                Use This Rule
              </Button>
            </>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
