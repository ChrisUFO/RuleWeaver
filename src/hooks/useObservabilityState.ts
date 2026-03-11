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
  query: string;
  entityName: string;
  source: string;
  eventType: ObservabilityEventType | "";
  status: ObservabilityEventStatus | "";
  fromTimestamp: string;
  toTimestamp: string;
  isLoading: boolean;
  isExporting: boolean;
  error: string | null;
  hasMore: boolean;
  handlers: {
    setQuery: (value: string) => void;
    setEntityName: (value: string) => void;
    setSource: (value: string) => void;
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
}

export function useObservabilityState(
  addToast: ReturnType<typeof useToast>["addToast"]
): UseObservabilityStateReturn {
  const [events, setEvents] = useState<ObservabilityEvent[]>([]);
  const [selectedIds, setSelectedIds] = useState<string[]>([]);
  const [query, setQuery] = useState("");
  const [entityName, setEntityName] = useState("");
  const [source, setSource] = useState("");
  const [eventType, setEventType] = useState<ObservabilityEventType | "">("");
  const [status, setStatus] = useState<ObservabilityEventStatus | "">("");
  const [fromTimestamp, setFromTimestamp] = useState("");
  const [toTimestamp, setToTimestamp] = useState("");
  const [limit, setLimit] = useState(DEFAULT_PAGE_SIZE);
  const [isLoading, setIsLoading] = useState(true);
  const [isExporting, setIsExporting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const filter = useMemo<ObservabilityEventFilter>(
    () => ({
      eventType: eventType || undefined,
      status: status || undefined,
      source: source.trim() || undefined,
      entityName: entityName.trim() || undefined,
      query: query.trim() || undefined,
      fromTimestamp: toUnixTimestamp(fromTimestamp),
      toTimestamp: toUnixTimestamp(toTimestamp),
      limit,
    }),
    [entityName, eventType, fromTimestamp, limit, query, source, status, toTimestamp]
  );

  const refresh = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const nextEvents = await api.observability.list(filter);
      setEvents(nextEvents);
      setSelectedIds((current) =>
        current.filter((id) => nextEvents.some((event) => event.id === id))
      );
    } catch (cause) {
      const message = cause instanceof Error ? cause.message : "Failed to load logs";
      setError(message);
      addToast({ title: "Logs Unavailable", description: message, variant: "error" });
    } finally {
      setIsLoading(false);
    }
  }, [addToast, filter]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const clearFilters = useCallback(() => {
    setQuery("");
    setEntityName("");
    setSource("");
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
    } catch (cause) {
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
      setEventType: (value) => {
        setLimit(DEFAULT_PAGE_SIZE);
        setEventType(value);
      },
      setStatus: (value) => {
        setLimit(DEFAULT_PAGE_SIZE);
        setStatus(value);
      },
      setFromTimestamp: (value) => {
        setLimit(DEFAULT_PAGE_SIZE);
        setFromTimestamp(value);
      },
      setToTimestamp: (value) => {
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
