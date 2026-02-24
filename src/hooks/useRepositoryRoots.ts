import { useCallback, useEffect, useState } from "react";
import { api } from "@/lib/tauri";

export function parseRepositoryRoots(raw: string | null | undefined): string[] {
  if (!raw) return [];
  try {
    const parsed = JSON.parse(raw) as string[];
    return Array.isArray(parsed) ? parsed : [];
  } catch (error) {
    console.error("Failed to parse repository roots", { raw, error });
    return [];
  }
}

export function useRepositoryRoots(autoLoad = true) {
  const [roots, setRoots] = useState<string[]>([]);
  const [isLoading, setIsLoading] = useState(autoLoad);

  const refresh = useCallback(async () => {
    setIsLoading(true);
    try {
      const raw = await api.settings.get("local_rule_paths");
      setRoots(parseRepositoryRoots(raw));
    } catch (error) {
      console.error("Failed to refresh repository roots", { error });
      setRoots([]);
    } finally {
      setIsLoading(false);
    }
  }, []);

  const save = useCallback(async (nextRoots: string[]) => {
    await api.settings.set("local_rule_paths", JSON.stringify(nextRoots));
    setRoots(nextRoots);
  }, []);

  useEffect(() => {
    if (!autoLoad) return;
    void refresh();
  }, [autoLoad, refresh]);

  return { roots, setRoots, isLoading, refresh, save };
}
