import { useCallback, useEffect, useState } from "react";
import { api } from "@/lib/tauri";
import type { ToolEntry, AdapterType } from "@/types/rule";

export function useToolRegistry(autoLoad = true) {
  const [tools, setTools] = useState<ToolEntry[]>([]);
  const [isLoading, setIsLoading] = useState(autoLoad);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const registry = await api.registry.getTools();
      setTools(registry);
    } catch (err) {
      console.error("Failed to load tool registry", { error: err });
      setError(err instanceof Error ? err.message : "Failed to load tool registry");
      setTools([]);
    } finally {
      setIsLoading(false);
    }
  }, []);

  const getTool = useCallback(
    (adapterId: AdapterType): ToolEntry | undefined => {
      return tools.find((t) => t.id === adapterId);
    },
    [tools]
  );

  const getToolsByCapability = useCallback(
    (capability: keyof ToolEntry["capabilities"]): ToolEntry[] => {
      return tools.filter((t) => t.capabilities[capability] === true);
    },
    [tools]
  );

  const getAdapterFilterOptions = useCallback(() => {
    return [
      { value: "all", label: "All Adapters" },
      ...tools.map((t) => ({ value: t.id, label: t.name })),
    ];
  }, [tools]);

  useEffect(() => {
    if (!autoLoad) return;
    void refresh();
  }, [autoLoad, refresh]);

  return {
    tools,
    isLoading,
    error,
    refresh,
    getTool,
    getToolsByCapability,
    getAdapterFilterOptions,
  };
}
