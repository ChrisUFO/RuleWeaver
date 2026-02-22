export type Scope = "global" | "local";

export type AdapterType = "antigravity" | "gemini" | "opencode" | "cline" | "claude-code" | "codex";

export interface Rule {
  id: string;
  name: string;
  content: string;
  scope: Scope;
  targetPaths: string[] | null;
  enabledAdapters: AdapterType[];
  enabled: boolean;
  createdAt: number;
  updatedAt: number;
}

export interface CreateRuleInput {
  name: string;
  content: string;
  scope: Scope;
  targetPaths?: string[];
  enabledAdapters: AdapterType[];
  enabled?: boolean;
}

export interface UpdateRuleInput {
  name?: string;
  content?: string;
  scope?: Scope;
  targetPaths?: string[];
  enabledAdapters?: AdapterType[];
  enabled?: boolean;
}

export interface SyncResult {
  success: boolean;
  filesWritten: string[];
  errors: SyncError[];
  conflicts: Conflict[];
}

export interface SyncError {
  filePath: string;
  adapterName: string;
  message: string;
}

export interface Conflict {
  id: string;
  filePath: string;
  adapterName: string;
  adapterId?: AdapterType;
  localHash: string;
  currentHash: string;
}

export interface SyncHistoryEntry {
  id: string;
  timestamp: number;
  filesWritten: number;
  status: "success" | "partial" | "failed";
  triggeredBy: "manual" | "auto";
}

export interface AdapterInfo {
  id: AdapterType;
  name: string;
  fileName: string;
  icon: string;
  description: string;
  globalPath: string;
  enabled: boolean;
}

export const ADAPTERS: AdapterInfo[] = [
  {
    id: "antigravity",
    name: "Antigravity",
    fileName: "ANTIGRAVITY.md",
    icon: "antigravity",
    description: "Antigravity AI coding assistant",
    globalPath: "~/.antigravity/ANTIGRAVITY.md",
    enabled: true,
  },
  {
    id: "gemini",
    name: "Gemini CLI",
    fileName: "GEMINI.md",
    icon: "gemini",
    description: "Google's Gemini AI coding assistant",
    globalPath: "~/.gemini/GEMINI.md",
    enabled: true,
  },
  {
    id: "opencode",
    name: "OpenCode",
    fileName: "AGENTS.md",
    icon: "opencode",
    description: "OpenCode AI coding assistant",
    globalPath: "~/.opencode/AGENTS.md",
    enabled: true,
  },
  {
    id: "cline",
    name: "Cline",
    fileName: ".clinerules",
    icon: "cline",
    description: "Cline VS Code extension",
    globalPath: "~/.clinerules",
    enabled: true,
  },
  {
    id: "claude-code",
    name: "Claude Code",
    fileName: "CLAUDE.md",
    icon: "claude",
    description: "Anthropic's Claude Code assistant",
    globalPath: "~/.claude/CLAUDE.md",
    enabled: true,
  },
  {
    id: "codex",
    name: "Codex",
    fileName: "CODEX.md",
    icon: "codex",
    description: "OpenAI Codex assistant",
    globalPath: "~/.codex/CODEX.md",
    enabled: true,
  },
];
