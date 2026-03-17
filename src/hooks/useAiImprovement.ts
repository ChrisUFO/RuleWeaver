import { useState, useCallback, useRef } from "react";
import { api } from "@/lib/tauri";
import type { ImproveRuleOutput } from "@/types/ai";
import { getAiErrorMessage } from "@/types/ai";

interface UseAiImprovementOptions {
  onSuccess?: (result: ImproveRuleOutput) => void;
  onError?: (error: unknown) => void;
}

interface UseAiImprovementReturn {
  isImproving: boolean;
  improvedContent: string | null;
  modelUsed: string | null;
  error: string | null;
  improve: (ruleContent: string, ruleName?: string) => Promise<ImproveRuleOutput | null>;
  clearResult: () => void;
}

export function useAiImprovement(options: UseAiImprovementOptions = {}): UseAiImprovementReturn {
  const [isImproving, setIsImproving] = useState(false);
  const [improvedContent, setImprovedContent] = useState<string | null>(null);
  const [modelUsed, setModelUsed] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const inProgressRef = useRef(false);

  const optionsRef = useRef(options);
  optionsRef.current = options;

  const improve = useCallback(
    async (ruleContent: string, ruleName?: string): Promise<ImproveRuleOutput | null> => {
      if (inProgressRef.current) {
        console.log("[useAiImprovement] Already improving, skipping duplicate request");
        return null;
      }

      inProgressRef.current = true;
      setIsImproving(true);
      setError(null);
      setImprovedContent(null);
      setModelUsed(null);

      console.log("[useAiImprovement] Starting AI improvement request", {
        contentLength: ruleContent.length,
        ruleName,
      });

      try {
        const result = await api.ai.improveRule({
          ruleContent,
          ruleName: ruleName || null,
        });

        console.log("[useAiImprovement] AI improvement succeeded", {
          modelUsed: result.modelUsed,
          contentLength: result.improvedContent.length,
        });

        setImprovedContent(result.improvedContent);
        setModelUsed(result.modelUsed);
        optionsRef.current.onSuccess?.(result);
        return result;
      } catch (err) {
        const errorMessage = getAiErrorMessage(err);
        console.error("[useAiImprovement] AI improvement failed", {
          error: err,
          errorMessage,
          errorType: err?.constructor?.name,
          errorString: String(err),
        });
        setError(errorMessage);
        optionsRef.current.onError?.(err);
        return null;
      } finally {
        inProgressRef.current = false;
        setIsImproving(false);
      }
    },
    []
  );

  const clearResult = useCallback(() => {
    setImprovedContent(null);
    setModelUsed(null);
    setError(null);
  }, []);

  return {
    isImproving,
    improvedContent,
    modelUsed,
    error,
    improve,
    clearResult,
  };
}
