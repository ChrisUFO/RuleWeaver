import { useCallback, useEffect, useMemo, useState } from "react";
import { save } from "@tauri-apps/plugin-dialog";
import { api } from "@/lib/tauri";
import type { useToast } from "@/components/ui/toast";
import type {
  ObservabilityEvent,
  ObservabilityEventFilter,
  ObservabilityEventStatus,
  ObservabilityEventType,
} from "@/types/observability";

const DEFAULT_PAGE_SIZE = 250;

function toUnixTimestamp(value: string): number | undefined {
  if (!value) return undefined;
  const parsed = new Date(value);
  return Number.isNaN(parsed.getTime()) ? undefined : Math.floor(parsed.getTime() / 1000);
}

export interface UseObservabilityStateReturn {
  events: ObservabilityEvent[];
  selectedIds: string[];
  workspacePath: string;
  isLoading: boolean;
  isExporting: boolean;
  error: string | null;
  hasMore: boolean;
  handlers: {
    setQuery: (value: string) => void;
    setEntityName: (value: string) => void;
    setSource: (value: string) => void;
    setWorkspacePath: (value: string) => void;
    setEventType: (value: ObservabilityEventType | "") => void;
    setStatus: (value: ObservabilityEventStatus | "") => void;
    setFromTimestamp: (value: string) => void;
    setToTimestamp: (value: string) => void;
    refresh: () => Promise<void>;
    clearFilters: () => void;
    toggleSelected: (id: string) => void;
    exportLogs: () => Promise<void>;
    loadMore: () => void;
  };
  query: string;
  source: string;
  entityName: string;
  eventType: ObservabilityEventType | "";
  status: ObservabilityEventStatus | "";
  fromTimestamp: string;
  toTimestamp: string;
}

export function useObservabilityState(
  addToast: ReturnType<typeof useToast>["addToast"]
): UseObservabilityStateReturn {
  const [events, setEvents] = useState<ObservabilityEvent[]>([]);
  const [selectedIds, setSelectedIds] = useState<string[]>([]);
  const [workspacePath, setWorkspacePath] = useState("");
  const [limit, setLimit] = useState(DEFAULT_PAGE_SIZE);
  const [query, setQuery] = useState("");
  const [entityName, setEntityName] = useState("");
  const [source, setSource] = useState("");
  const [eventType, setEventType] = useState<ObservabilityEventType | "">("");
  const [status, setStatus] = useState<ObservabilityEventStatus | "">("");
  const [fromTimestamp, setFromTimestamp] = useState("");
  const [toTimestamp, setToTimestamp] = useState("");
  const [isLoading, setIsLoading] = useState(true);
  const [isExporting, setIsExporting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Hardening: Debounced query to prevent excessive backend load
  const [debouncedQuery, setDebouncedQuery] = useState(query);
  useEffect(() => {
    const timer = setTimeout(() => setDebouncedQuery(query), 300);
    return () => clearTimeout(timer);
  }, [query]);

  const filter = useMemo<ObservabilityEventFilter>(
    () => ({
      eventType: eventType || undefined,
      status: status || undefined,
      source: source.trim() || undefined,
      entityName: entityName.trim() || undefined,
      workspacePath: workspacePath.trim() || undefined,
      query: debouncedQuery.trim() || undefined,
      fromTimestamp: toUnixTimestamp(fromTimestamp),
      toTimestamp: toUnixTimestamp(toTimestamp),
      limit,
    }),
    [
      debouncedQuery,
      entityName,
      eventType,
      fromTimestamp,
      limit,
      source,
      status,
      toTimestamp,
      workspacePath,
    ]
  );

  const refresh = useCallback(
    async (signal?: AbortSignal) => {
      setIsLoading(true);
      setError(null);
      try {
        const nextEvents = await api.observability.list(filter);
        if (signal?.aborted) return;
        setEvents(nextEvents);
        setSelectedIds((current) =>
          current.filter((id) => nextEvents.some((event) => event.id === id))
        );
      } catch (cause: unknown) {
        if (signal?.aborted) return;
        const message = cause instanceof Error ? cause.message : "Failed to load logs";
        setError(message);
        addToast({ title: "Logs Unavailable", description: message, variant: "error" });
      } finally {
        if (!signal?.aborted) setIsLoading(false);
      }
    },
    [addToast, filter]
  );

  useEffect(() => {
    const controller = new AbortController();
    void refresh(controller.signal);
    return () => controller.abort();
  }, [refresh]);

  const clearFilters = useCallback(() => {
    setQuery("");
    setEntityName("");
    setSource("");
    setWorkspacePath("");
    setEventType("");
    setStatus("");
    setFromTimestamp("");
    setToTimestamp("");
    setLimit(DEFAULT_PAGE_SIZE);
  }, []);

  const toggleSelected = useCallback((id: string) => {
    setSelectedIds((current) =>
      current.includes(id) ? current.filter((value) => value !== id) : [...current, id]
    );
  }, []);

  const exportLogs = useCallback(async () => {
    const path = await save({
      defaultPath: `ruleweaver-logs-${new Date().toISOString().replace(/[:.]/g, "-")}.json`,
      filters: [{ name: "JSON", extensions: ["json"] }],
    });
    if (!path) return;

    setIsExporting(true);
    try {
      const count = await api.observability.export(
        path,
        filter,
        selectedIds.length > 0 ? selectedIds : undefined
      );
      addToast({
        title: "Logs Exported",
        description:
          selectedIds.length > 0
            ? `Exported ${count} selected log entries.`
            : `Exported ${count} filtered log entries.`,
        variant: "success",
      });
    } catch (cause: unknown) {
      addToast({
        title: "Export Failed",
        description: cause instanceof Error ? cause.message : "Failed to export logs",
        variant: "error",
      });
    } finally {
      setIsExporting(false);
    }
  }, [addToast, filter, selectedIds]);

  return {
    events,
    selectedIds,
    query,
    entityName,
    source,
    eventType,
    status,
    fromTimestamp,
    toTimestamp,
    workspacePath,
    isLoading,
    isExporting,
    error,
    hasMore: events.length >= limit,
    handlers: {
      setQuery: (value) => {
        setLimit(DEFAULT_PAGE_SIZE);
        setQuery(value);
      },
      setEntityName: (value) => {
        setLimit(DEFAULT_PAGE_SIZE);
        setEntityName(value);
      },
      setSource: (value) => {
        setLimit(DEFAULT_PAGE_SIZE);
        setSource(value);
      },
      setWorkspacePath: (value) => {
        setLimit(DEFAULT_PAGE_SIZE);
        setWorkspacePath(value);
      },
      setEventType: (value) => {
        setLimit(DEFAULT_PAGE_SIZE);
        setEventType(value);
      },
      setStatus: (value) => {
        setLimit(DEFAULT_PAGE_SIZE);
        setStatus(value);
      },
      setFromTimestamp: (value: string) => {
        setLimit(DEFAULT_PAGE_SIZE);
        setFromTimestamp(value);
      },
      setToTimestamp: (value: string) => {
        setLimit(DEFAULT_PAGE_SIZE);
        setToTimestamp(value);
      },
      refresh,
      clearFilters,
      toggleSelected,
      exportLogs,
      loadMore: () => setLimit((current) => current + DEFAULT_PAGE_SIZE),
    },
  };
}
