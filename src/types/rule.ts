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

export const ADAPTERS: {
  id: AdapterType;
  name: string;
  description: string;
  icon: string;
  enabled: boolean;
  fileName: string;
  globalPath: string;
}[] = [
  {
    id: "antigravity",
    name: "Antigravity",
    description: "Antigravity AI coding assistant",
    icon: "antigravity",
    enabled: true,
    fileName: "GEMINI.md",
    globalPath: "~/.gemini/GEMINI.md",
  },
  {
    id: "gemini",
    name: "Gemini",
    description: "Google's Gemini AI coding assistant",
    icon: "gemini",
    enabled: true,
    fileName: "GEMINI.md",
    globalPath: "~/.gemini/GEMINI.md",
  },
  {
    id: "opencode",
    name: "OpenCode",
    description: "OpenCode AI coding assistant",
    icon: "opencode",
    enabled: true,
    fileName: "AGENTS.md",
    globalPath: "~/.config/opencode/AGENTS.md",
  },
  {
    id: "cline",
    name: "Cline",
    description: "Cline VS Code extension",
    icon: "cline",
    enabled: true,
    fileName: ".clinerules",
    globalPath: "~/.clinerules",
  },
  {
    id: "claude-code",
    name: "Claude Code",
    description: "Anthropic's Claude Code assistant",
    icon: "claude",
    enabled: true,
    fileName: "CLAUDE.md",
    globalPath: "~/.claude/CLAUDE.md",
  },
  {
    id: "codex",
    name: "Codex",
    description: "OpenAI Codex assistant",
    icon: "codex",
    enabled: true,
    fileName: "AGENTS.md",
    globalPath: "~/.codex/AGENTS.md",
  },
  {
    id: "kilo",
    name: "Kilo Code",
    description: "Kilo Code AI assistant",
    icon: "kilo",
    enabled: true,
    fileName: "AGENTS.md",
    globalPath: "~/.kilocode/rules/AGENTS.md",
  },
  {
    id: "cursor",
    name: "Cursor",
    description: "Cursor AI code editor",
    icon: "cursor",
    enabled: true,
    fileName: ".cursorrules",
    globalPath: "~/.cursorrules",
  },
  {
    id: "windsurf",
    name: "Windsurf",
    description: "Windsurf AI assistant",
    icon: "windsurf",
    enabled: true,
    fileName: "rules.md",
    globalPath: "~/.windsurf/rules/rules.md",
  },
  {
    id: "roocode",
    name: "Roo Code",
    description: "Roo Code AI assistant",
    icon: "roocode",
    enabled: true,
    fileName: "rules.md",
    globalPath: "~/.roo/rules/rules.md",
  },
];

export interface Rule {
  id: string;
  name: string;
  description: string;
  content: string;
  scope: Scope;
  targetPaths: string[] | null;
  enabledAdapters: AdapterType[];
  enabled: boolean;
  createdAt: number;
  updatedAt: number;
}

export interface CreateRuleInput {
  id?: string;
  name: string;
  description: string;
  content: string;
  scope: Scope;
  targetPaths?: string[];
  enabledAdapters: AdapterType[];
  enabled?: boolean;
}

export interface UpdateRuleInput {
  name?: string;
  description?: string;
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

export interface ToolCapabilities {
  supportsRules: boolean;
  supportsCommandStubs: boolean;
  supportsSlashCommands: boolean;
  supportsSkills: boolean;
  supportsGlobalScope: boolean;
  supportsLocalScope: boolean;
}

export interface PathTemplates {
  globalPath: string;
  localPathTemplate: string;
}

export interface ToolEntry {
  id: AdapterType;
  name: string;
  description: string;
  icon: string;
  capabilities: ToolCapabilities;
  paths: PathTemplates;
  fileFormat: string;
}

export interface TemplateRule {
  templateId: string;
  theme: string;
  metadata: CreateRuleInput;
}
