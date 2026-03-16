import { useState, useCallback } from "react";
import { api } from "@/lib/tauri";
import type { GenerateRuleOutput } from "@/types/ai";

interface UseAiGenerationOptions {
  onSuccess?: (result: GenerateRuleOutput) => void;
  onError?: (error: unknown) => void;
}

interface UseAiGenerationReturn {
  isGenerating: boolean;
  generatedContent: string | null;
  suggestedName: string | null;
  modelUsed: string | null;
  error: string | null;
  generate: (
    description: string,
    ruleName?: string,
    context?: string
  ) => Promise<GenerateRuleOutput | null>;
  clearResult: () => void;
}

export function useAiGeneration(options: UseAiGenerationOptions = {}): UseAiGenerationReturn {
  const [isGenerating, setIsGenerating] = useState(false);
  const [generatedContent, setGeneratedContent] = useState<string | null>(null);
  const [suggestedName, setSuggestedName] = useState<string | null>(null);
  const [modelUsed, setModelUsed] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const generate = useCallback(
    async (
      description: string,
      ruleName?: string,
      context?: string
    ): Promise<GenerateRuleOutput | null> => {
      setIsGenerating(true);
      setError(null);
      setGeneratedContent(null);
      setSuggestedName(null);
      setModelUsed(null);

      try {
        const result = await api.ai.generateRule({
          description,
          ruleName: ruleName || null,
          context: context || null,
        });

        setGeneratedContent(result.ruleContent);
        setSuggestedName(result.suggestedName);
        setModelUsed(result.modelUsed);
        options.onSuccess?.(result);
        return result;
      } catch (err) {
        const errorMessage =
          err instanceof Error ? err.message : typeof err === "string" ? err : "Unknown error";
        setError(errorMessage);
        options.onError?.(err);
        return null;
      } finally {
        setIsGenerating(false);
      }
    },
    [options]
  );

  const clearResult = useCallback(() => {
    setGeneratedContent(null);
    setSuggestedName(null);
    setModelUsed(null);
    setError(null);
  }, []);

  return {
    isGenerating,
    generatedContent,
    suggestedName,
    modelUsed,
    error,
    generate,
    clearResult,
  };
}
