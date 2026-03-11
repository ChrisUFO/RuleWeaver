import { Download, RefreshCw, Search, X } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Select } from "@/components/ui/select";
import { useToast } from "@/components/ui/toast";
import { useObservabilityState } from "@/hooks/useObservabilityState";
import type { ObservabilityEventStatus, ObservabilityEventType } from "@/types/observability";

const EVENT_TYPE_OPTIONS = [
  { value: "", label: "All event types" },
  { value: "mcpLifecycle", label: "MCP lifecycle" },
  { value: "mcpClient", label: "MCP client" },
  { value: "commandRun", label: "Command runs" },
  { value: "skillRun", label: "Skill runs" },
];

const STATUS_OPTIONS = [
  { value: "", label: "All statuses" },
  { value: "info", label: "Info" },
  { value: "started", label: "Started" },
  { value: "success", label: "Success" },
  { value: "warning", label: "Warning" },
  { value: "error", label: "Error" },
  { value: "stopped", label: "Stopped" },
];

function badgeVariantForStatus(status: ObservabilityEventStatus) {
  switch (status) {
    case "success":
      return "success" as const;
    case "warning":
      return "warning" as const;
    case "error":
      return "destructive" as const;
    default:
      return "outline" as const;
  }
}

function formatMetadata(metadata?: string | null) {
  if (!metadata) return null;
  try {
    return JSON.stringify(JSON.parse(metadata), null, 2);
  } catch {
    return metadata;
  }
}

export function Logs() {
  const { addToast } = useToast();
  const {
    events,
    selectedIds,
    query,
    entityName,
    source,
    eventType,
    status,
    fromTimestamp,
    toTimestamp,
    isLoading,
    isExporting,
    error,
    hasMore,
    handlers,
  } = useObservabilityState(addToast);

  return (
    <div className="space-y-6 p-6">
      <div className="flex flex-col gap-3 lg:flex-row lg:items-end lg:justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Logs</h1>
          <p className="text-sm text-muted-foreground">
            Unified diagnostics for MCP lifecycle, client activity, command runs, and skill runs.
          </p>
        </div>
        <div className="flex flex-wrap gap-2">
          <Button variant="outline" onClick={() => void handlers.refresh()} disabled={isLoading}>
            <RefreshCw className={`mr-2 h-4 w-4 ${isLoading ? "animate-spin" : ""}`} />
            Refresh
          </Button>
          <Button onClick={() => void handlers.exportLogs()} disabled={isExporting || isLoading}>
            <Download className="mr-2 h-4 w-4" />
            {selectedIds.length > 0 ? `Export Selected (${selectedIds.length})` : "Export Filtered"}
          </Button>
        </div>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Filters</CardTitle>
          <CardDescription>
            Search across timestamps, sources, tools, and redacted excerpts.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-4">
            <label className="space-y-2 text-sm">
              <span>Text search</span>
              <div className="relative">
                <Search className="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                <Input
                  value={query}
                  onChange={(e) => handlers.setQuery(e.target.value)}
                  className="pl-9"
                />
              </div>
            </label>
            <label className="space-y-2 text-sm">
              <span>Tool or skill</span>
              <Input value={entityName} onChange={(e) => handlers.setEntityName(e.target.value)} />
            </label>
            <label className="space-y-2 text-sm">
              <span>Source</span>
              <Input value={source} onChange={(e) => handlers.setSource(e.target.value)} />
            </label>
            <label className="space-y-2 text-sm">
              <span>Event type</span>
              <Select
                options={EVENT_TYPE_OPTIONS}
                value={eventType}
                onChange={(value) => handlers.setEventType(value as ObservabilityEventType | "")}
              />
            </label>
            <label className="space-y-2 text-sm">
              <span>Status</span>
              <Select
                options={STATUS_OPTIONS}
                value={status}
                onChange={(value) => handlers.setStatus(value as ObservabilityEventStatus | "")}
              />
            </label>
            <label className="space-y-2 text-sm">
              <span>From</span>
              <Input
                type="datetime-local"
                value={fromTimestamp}
                onChange={(e) => handlers.setFromTimestamp(e.target.value)}
              />
            </label>
            <label className="space-y-2 text-sm">
              <span>To</span>
              <Input
                type="datetime-local"
                value={toTimestamp}
                onChange={(e) => handlers.setToTimestamp(e.target.value)}
              />
            </label>
            <div className="flex items-end">
              <Button variant="ghost" onClick={handlers.clearFilters} className="w-full">
                <X className="mr-2 h-4 w-4" />
                Clear filters
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Entries</CardTitle>
          <CardDescription>
            Showing {events.length} entries
            {selectedIds.length > 0 ? ` • ${selectedIds.length} selected` : ""}
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          {error ? (
            <div className="rounded-md border border-destructive/30 bg-destructive/10 p-4 text-sm">
              {error}
            </div>
          ) : isLoading ? (
            <p className="text-sm text-muted-foreground">Loading logs…</p>
          ) : events.length === 0 ? (
            <p className="text-sm text-muted-foreground">
              No log entries match the current filters.
            </p>
          ) : (
            events.map((event) => (
              <label
                key={event.id}
                className="block rounded-lg border p-4 transition hover:border-primary/40"
              >
                <div className="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
                  <div className="flex gap-3">
                    <input
                      aria-label={`Select log ${event.summary}`}
                      type="checkbox"
                      className="mt-1 h-4 w-4 rounded border-input"
                      checked={selectedIds.includes(event.id)}
                      onChange={() => handlers.toggleSelected(event.id)}
                    />
                    <div className="space-y-2">
                      <div className="flex flex-wrap items-center gap-2">
                        <span className="font-semibold">{event.summary}</span>
                        <Badge variant={badgeVariantForStatus(event.status)}>{event.status}</Badge>
                        <Badge variant="outline">{event.eventType}</Badge>
                        {event.entityName ? (
                          <Badge variant="secondary">{event.entityName}</Badge>
                        ) : null}
                      </div>
                      <div className="flex flex-wrap gap-x-4 gap-y-1 text-xs text-muted-foreground">
                        <span>{new Date(event.timestamp * 1000).toLocaleString()}</span>
                        <span>source: {event.source}</span>
                        {event.durationMs != null ? <span>{event.durationMs}ms</span> : null}
                        {event.exitCode != null ? <span>exit: {event.exitCode}</span> : null}
                        {event.failureClass ? <span>failure: {event.failureClass}</span> : null}
                        {event.attemptNumber != null ? (
                          <span>attempt: {event.attemptNumber}</span>
                        ) : null}
                      </div>
                    </div>
                  </div>
                  {event.isRedacted ? (
                    <Badge variant="outline" className="self-start">
                      redacted
                    </Badge>
                  ) : null}
                </div>

                {formatMetadata(event.metadata) ? (
                  <details className="mt-3 rounded-md bg-muted/40 p-3 text-xs">
                    <summary className="cursor-pointer font-medium">Metadata</summary>
                    <pre className="mt-2 whitespace-pre-wrap break-all text-muted-foreground">
                      {formatMetadata(event.metadata)}
                    </pre>
                  </details>
                ) : null}

                {event.stdoutExcerpt || event.stderrExcerpt ? (
                  <details className="mt-3 rounded-md bg-muted/40 p-3 text-xs">
                    <summary className="cursor-pointer font-medium">Output excerpts</summary>
                    {event.stdoutExcerpt ? (
                      <div className="mt-2 space-y-1">
                        <div className="font-medium">stdout</div>
                        <pre className="whitespace-pre-wrap break-all text-muted-foreground">
                          {event.stdoutExcerpt}
                        </pre>
                      </div>
                    ) : null}
                    {event.stderrExcerpt ? (
                      <div className="mt-2 space-y-1">
                        <div className="font-medium">stderr</div>
                        <pre className="whitespace-pre-wrap break-all text-muted-foreground">
                          {event.stderrExcerpt}
                        </pre>
                      </div>
                    ) : null}
                  </details>
                ) : null}
              </label>
            ))
          )}
          {!error && !isLoading && hasMore && events.length > 0 ? (
            <div className="flex justify-center pt-2">
              <Button variant="outline" onClick={handlers.loadMore}>
                Load more
              </Button>
            </div>
          ) : null}
        </CardContent>
      </Card>
    </div>
  );
}
