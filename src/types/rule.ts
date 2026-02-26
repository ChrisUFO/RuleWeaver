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
export type ImportArtifactType = "rule" | "command" | "skill" | "other" | "unknown";
export type ImportConflictMode = "skip" | "rename" | "replace";

export interface ImportCandidate {
  id: string;
  sourceType: ImportSourceType;
  sourceLabel: string;
  sourcePath: string;
  sourceTool?: AdapterType;
  artifactType: ImportArtifactType;
  name: string;
  proposedName: string;
  content: string;
  scope: Scope;
  targetPaths: string[] | null;
  enabledAdapters: AdapterType[];
  contentHash: string;
  fileSize: number;
  metadata?: string;
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
  importedRules: Rule[];
  importedCommands: unknown[]; // Avoid circular dependency
  importedSkills: unknown[];
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
