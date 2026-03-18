export type AiProvider =
  | "openai"
  | "anthropic"
  | "google-ai-studio"
  | "openrouter"
  | "deepseek"
  | "together-ai"
  | "minimax-api"
  | "minimax-coding-plan"
  | "zai-general-api"
  | "zai-coding-plan"
  | "openai-compatible";

export interface AiSettings {
  provider: AiProvider;
  baseUrl: string | null;
  model: string;
  hasApiKey: boolean;
  improvementPrompt: string | null;
  generationPrompt: string | null;
  enabled: boolean;
}

export interface SaveAiSettingsInput {
  provider: AiProvider;
  baseUrl?: string | null;
  model: string;
  apiKey?: string | null;
  improvementPrompt?: string | null;
  generationPrompt?: string | null;
  enabled: boolean;
}

export interface ModelInfo {
  id: string;
  name: string | null;
  contextLength: number | null;
}

export interface TestConnectionOutput {
  success: boolean;
  modelAvailable: boolean;
  error: string | null;
}

export interface ImproveRuleInput {
  ruleContent: string;
  ruleName?: string | null;
  additionalInstructions?: string | null;
}

export interface ImproveRuleOutput {
  improvedContent: string;
  modelUsed: string;
  tokensUsed: number | null;
}

export interface GenerateRuleInput {
  description: string;
  ruleName?: string | null;
  context?: string | null;
}

export interface GenerateRuleOutput {
  ruleContent: string;
  suggestedName: string | null;
  modelUsed: string;
  tokensUsed: number | null;
}

export const AI_PROVIDER_INFO: Record<
  AiProvider,
  { name: string; description: string; requiresBaseUrl: boolean }
> = {
  openai: {
    name: "OpenAI",
    description: "GPT-4 and GPT-3.5 models",
    requiresBaseUrl: false,
  },
  anthropic: {
    name: "Anthropic",
    description: "Claude models",
    requiresBaseUrl: false,
  },
  "google-ai-studio": {
    name: "Google AI Studio",
    description: "Gemini models via OpenAI-compatible API",
    requiresBaseUrl: false,
  },
  openrouter: {
    name: "OpenRouter",
    description: "Access to multiple AI providers",
    requiresBaseUrl: false,
  },
  deepseek: {
    name: "DeepSeek",
    description: "DeepSeek models",
    requiresBaseUrl: false,
  },
  "together-ai": {
    name: "Together AI",
    description: "Open-source model hosting",
    requiresBaseUrl: false,
  },
  "minimax-api": {
    name: "MiniMax API",
    description: "MiniMax models (general API)",
    requiresBaseUrl: false,
  },
  "minimax-coding-plan": {
    name: "MiniMax Coding Plan",
    description: "MiniMax models (coding plan)",
    requiresBaseUrl: false,
  },
  "zai-general-api": {
    name: "Z.ai General API",
    description: "Z.ai models (general API)",
    requiresBaseUrl: false,
  },
  "zai-coding-plan": {
    name: "Z.ai Coding Plan",
    description: "Z.ai models (coding plan)",
    requiresBaseUrl: false,
  },
  "openai-compatible": {
    name: "OpenAI-Compatible",
    description: "Any OpenAI-compatible API endpoint",
    requiresBaseUrl: true,
  },
};

export const AI_VALIDATION = {
  MAX_DESCRIPTION_LENGTH: 2000,
  MAX_CONTEXT_LENGTH: 2000,
  MAX_RULE_CONTENT_LENGTH: 50000,
  LARGE_CONTENT_WARNING_THRESHOLD: 30000,
} as const;

export function getRuleContentSizeWarning(contentLength: number): string | null {
  if (contentLength > AI_VALIDATION.MAX_RULE_CONTENT_LENGTH) {
    return `Rule content exceeds ${Math.round(AI_VALIDATION.MAX_RULE_CONTENT_LENGTH / 1000)}k characters and may be too long for some AI models.`;
  }
  if (contentLength > AI_VALIDATION.LARGE_CONTENT_WARNING_THRESHOLD) {
    return `Rule content is large (${Math.round(contentLength / 1000)}k chars). Processing may take longer.`;
  }
  return null;
}

export function getAiErrorMessage(error: unknown): string {
  const message = error instanceof Error ? error.message : String(error);

  if (message.includes("API key not configured") || message.includes("ApiKeyNotSet")) {
    return "AI API key is not configured. Please set up your API key in Settings.";
  }
  if (message.includes("Invalid API key") || message.includes("InvalidApiKey")) {
    return "The API key is invalid. Please check your API key in Settings.";
  }
  if (message.includes("Rate limited") || message.includes("RateLimited")) {
    return "The AI service is rate limiting requests. Please wait a moment and try again.";
  }
  if (message.includes("Context too long") || message.includes("ContextTooLong")) {
    return "The content is too long for the AI model to process. Please try with shorter content.";
  }
  if (message.includes("Model not available") || message.includes("ModelNotAvailable")) {
    return "The selected AI model is not available. Please choose a different model in Settings.";
  }
  if (message.includes("timeout") || message.includes("Timeout")) {
    return "The AI request timed out. Please try again.";
  }
  if (message.includes("Network error") || message.includes("NetworkError")) {
    return "A network error occurred. Please check your internet connection and try again.";
  }
  if (message.includes("Secure storage")) {
    return "Failed to access secure storage. Please try restarting the application.";
  }
  if (message.includes("Base URL is required")) {
    return "Base URL is required for custom providers. Please configure it in Settings.";
  }
  if (message.includes("Model name is required")) {
    return "Please select or enter a model name in Settings.";
  }
  if (message.includes("AI feature not enabled")) {
    return "AI features are not enabled. Please enable them in Settings.";
  }

  return message || "An unexpected error occurred. Please try again.";
}
