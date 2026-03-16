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
