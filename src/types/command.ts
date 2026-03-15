export interface CommandArgument {
  name: string;
  description: string;
  required: boolean;
  defaultValue?: string;
}

export interface CommandModel {
  id: string;
  name: string;
  description: string;
  script: string;
  arguments: CommandArgument[];
  isPlaceholder: boolean;
  generateSlashCommands?: boolean;
  slashCommandAdapters?: string[];
  targetPaths?: string[];
  basePath?: string | null;
  timeoutMs?: number;
  maxRetries?: number;
  createdAt: number;
  updatedAt: number;
}

export interface CreateCommandInput {
  id?: string;
  name: string;
  description: string;
  script: string;
  isPlaceholder: boolean;
  arguments?: CommandArgument[];
  targetPaths?: string[];
  basePath?: string | null;
  timeoutMs?: number;
  maxRetries?: number;
}

export interface UpdateCommandInput {
  name?: string;
  description?: string;
  script?: string;
  isPlaceholder?: boolean;
  arguments?: CommandArgument[];
  generateSlashCommands?: boolean;
  slashCommandAdapters?: string[];
  targetPaths?: string[];
  basePath?: string | null;
  timeoutMs?: number;
  maxRetries?: number;
}

export interface TestCommandResult {
  success: boolean;
  stdout: string;
  stderr: string;
  exitCode: number;
  durationMs: number;
}

export interface ExecutionLog {
  id: string;
  commandId: string;
  commandName: string;
  arguments: string;
  stdout: string;
  stderr: string;
  exitCode: number;
  durationMs: number;
  executedAt: number;
  triggeredBy: string;
  failureClass?: string;
  adapterContext?: string;
  isRedacted?: boolean;
  attemptNumber?: number;
}
export interface TemplateCommand {
  templateId: string;
  theme: string;
  metadata: CreateCommandInput;
}
