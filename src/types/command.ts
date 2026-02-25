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
  exposeViaMcp: boolean;
  generateSlashCommands?: boolean;
  slashCommandAdapters?: string[];
  targetPaths?: string[];
  createdAt: number;
  updatedAt: number;
}

export interface CreateCommandInput {
  name: string;
  description: string;
  script: string;
  arguments?: CommandArgument[];
  exposeViaMcp?: boolean;
  targetPaths?: string[];
}

export interface UpdateCommandInput {
  name?: string;
  description?: string;
  script?: string;
  arguments?: CommandArgument[];
  exposeViaMcp?: boolean;
  generateSlashCommands?: boolean;
  slashCommandAdapters?: string[];
  targetPaths?: string[];
}

export interface TestCommandResult {
  success: boolean;
  stdout: string;
  stderr: string;
  exitCode: number;
  durationMs: number;
}

export interface McpStatus {
  running: boolean;
  port: number;
  uptimeSeconds: number;
}

export interface McpConnectionInstructions {
  claudeCodeJson: string;
  opencodeJson: string;
  standaloneCommand: string;
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
}
export interface TemplateCommand {
  templateId: string;
  theme: string;
  metadata: CreateCommandInput;
}
