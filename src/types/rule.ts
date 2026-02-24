export type Scope = "global" | "local";

export type AdapterType =
  | "antigravity"
  | "gemini"
  | "opencode"
  | "cline"
  | "claude-code"
  | "codex"
  | "kilo"
  | "cursor"
  | "windsurf"
  | "roocode";

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

export type ImportSourceType = "ai_tool" | "file" | "directory" | "url" | "clipboard";
export type ImportConflictMode = "skip" | "rename" | "replace";

export interface ImportCandidate {
  id: string;
  sourceType: ImportSourceType;
  sourceLabel: string;
  sourcePath: string;
  sourceTool?: AdapterType;
  name: string;
  proposedName: string;
  content: string;
  scope: Scope;
  targetPaths: string[] | null;
  enabledAdapters: AdapterType[];
  contentHash: string;
  fileSize: number;
}

export interface ImportScanResult {
  candidates: ImportCandidate[];
  errors: string[];
}

export interface ImportExecutionOptions {
  conflictMode?: ImportConflictMode;
  defaultScope?: Scope;
  defaultAdapters?: AdapterType[];
  selectedCandidateIds?: string[];
  maxFileSizeBytes?: number;
}

export interface ImportConflict {
  candidateId: string;
  candidateName: string;
  existingRuleId?: string;
  existingRuleName?: string;
  reason: string;
}

export interface ImportSkip {
  candidateId: string;
  name: string;
  reason: string;
}

export interface ImportExecutionResult {
  imported: Rule[];
  skipped: ImportSkip[];
  conflicts: ImportConflict[];
  errors: string[];
}

export interface ImportHistoryEntry {
  id: string;
  timestamp: number;
  sourceType: ImportSourceType;
  importedCount: number;
  skippedCount: number;
  conflictCount: number;
  errorCount: number;
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
    fileName: "GEMINI.md",
    icon: "antigravity",
    description: "Antigravity AI coding assistant",
    globalPath: "~/.gemini/GEMINI.md",
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
    globalPath: "~/.config/opencode/AGENTS.md",
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
    fileName: "AGENTS.md",
    icon: "codex",
    description: "OpenAI Codex assistant",
    globalPath: "~/.codex/AGENTS.md",
    enabled: true,
  },
  {
    id: "kilo",
    name: "Kilo Code",
    fileName: "AGENTS.md",
    icon: "kilo",
    description: "Kilo Code AI assistant",
    globalPath: "~/.kilocode/rules/AGENTS.md",
    enabled: true,
  },
  {
    id: "cursor",
    name: "Cursor",
    fileName: ".cursorrules",
    icon: "cursor",
    description: "Cursor AI code editor",
    globalPath: "~/.cursorrules",
    enabled: true,
  },
  {
    id: "windsurf",
    name: "Windsurf",
    fileName: ".windsurf/rules/rules.md",
    icon: "windsurf",
    description: "Windsurf AI assistant",
    globalPath: "~/.windsurf/rules/rules.md",
    enabled: true,
  },
  {
    id: "roocode",
    name: "Roo Code",
    fileName: ".roo/rules/rules.md",
    icon: "roocode",
    description: "Roo Code AI assistant",
    globalPath: "~/.roo/rules/rules.md",
    enabled: true,
  },
];
