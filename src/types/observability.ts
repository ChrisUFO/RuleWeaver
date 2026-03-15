export type ObservabilityEventType = "commandRun" | "skillRun";

export type ObservabilityEventStatus =
  | "info"
  | "started"
  | "success"
  | "warning"
  | "error"
  | "stopped";

export interface ObservabilityEvent {
  id: string;
  timestamp: number;
  eventType: ObservabilityEventType;
  status: ObservabilityEventStatus;
  source: string;
  entityKind?: string | null;
  entityId?: string | null;
  entityName?: string | null;
  summary: string;
  metadata?: string | null;
  stdoutExcerpt?: string | null;
  stderrExcerpt?: string | null;
  durationMs?: number | null;
  exitCode?: number | null;
  failureClass?: string | null;
  attemptNumber?: number | null;
  isRedacted: boolean;
  workspacePath?: string | null;
}

export interface ObservabilityEventFilter {
  eventType?: ObservabilityEventType;
  status?: ObservabilityEventStatus;
  source?: string;
  entityName?: string;
  workspacePath?: string;
  metadata?: Record<string, unknown>;
  fromTimestamp?: number;
  toTimestamp?: number;
  limit?: number;
}
