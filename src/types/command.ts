export interface CommandArgument {
  name: string;
  description: string;
  required: boolean;
  default_value?: string;
}

export interface CommandModel {
  id: string;
  name: string;
  description: string;
  script: string;
  arguments: CommandArgument[];
  expose_via_mcp: boolean;
  created_at: number;
  updated_at: number;
}

export interface CreateCommandInput {
  name: string;
  description: string;
  script: string;
  arguments?: CommandArgument[];
  expose_via_mcp?: boolean;
}

export interface UpdateCommandInput {
  name?: string;
  description?: string;
  script?: string;
  arguments?: CommandArgument[];
  expose_via_mcp?: boolean;
}

export interface TestCommandResult {
  success: boolean;
  stdout: string;
  stderr: string;
  exit_code: number;
  duration_ms: number;
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
