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
  uptime_seconds: number;
}

export interface McpConnectionInstructions {
  claude_code_json: string;
  opencode_json: string;
  standalone_command: string;
}

export interface ExecutionLog {
  id: string;
  command_id: string;
  command_name: string;
  arguments: string;
  stdout: string;
  stderr: string;
  exit_code: number;
  duration_ms: number;
  executed_at: number;
  triggered_by: string;
}
