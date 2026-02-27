import { useState, useEffect, useCallback } from "react";
import { api } from "@/lib/tauri";
import { listen } from "@tauri-apps/api/event";
import type { McpStatus } from "@/types/command";

/**
 * Shared hook for polling MCP status and listening for background refresh events.
 */
export function useMcpWatcher(onRefresh?: () => void) {
  const [mcpStatus, setMcpStatus] = useState<McpStatus | null>(null);
  const [mcpJustRefreshed, setMcpJustRefreshed] = useState(false);

  const loadMcpStatus = useCallback(async () => {
    try {
      const status = await api.mcp.getStatus();
      setMcpStatus(status);
    } catch {
      setMcpStatus(null);
    }
  }, []);

  useEffect(() => {
    loadMcpStatus();
    const timer = setInterval(loadMcpStatus, 5000);

    let unlisten: (() => void) | undefined;
    listen("mcp-artifacts-refreshed", () => {
      loadMcpStatus();
      if (onRefresh) {
        onRefresh();
      }
      setMcpJustRefreshed(true);
      setTimeout(() => setMcpJustRefreshed(false), 2000);
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      clearInterval(timer);
      if (unlisten) unlisten();
    };
  }, [loadMcpStatus, onRefresh]);

  return {
    mcpStatus,
    mcpJustRefreshed,
    refreshMcpStatus: loadMcpStatus,
  };
}
