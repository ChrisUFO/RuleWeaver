import { useState, useCallback } from "react";
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

  const improve = useCallback(
    async (ruleContent: string, ruleName?: string): Promise<ImproveRuleOutput | null> => {
      setIsImproving(true);
      setError(null);
      setImprovedContent(null);
      setModelUsed(null);

      try {
        const result = await api.ai.improveRule({
          ruleContent,
          ruleName: ruleName || null,
        });

        setImprovedContent(result.improvedContent);
        setModelUsed(result.modelUsed);
        options.onSuccess?.(result);
        return result;
      } catch (err) {
        const errorMessage = getAiErrorMessage(err);
        setError(errorMessage);
        options.onError?.(err);
        return null;
      } finally {
        setIsImproving(false);
      }
    },
    [options]
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
