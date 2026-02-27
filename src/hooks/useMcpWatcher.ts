import { useState, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { useMcpStore } from "@/stores/mcpStore";
import { MCP_TIMING } from "@/constants/timing";

/**
 * Shared hook for listening for background refresh events and accessing MCP status.
 * Polling is managed globally via the store if needed, or locally but sharing state.
 */
export function useMcpWatcher(onRefresh?: () => void) {
  const { mcpStatus, refreshMcpStatus } = useMcpStore();
  const [mcpJustRefreshed, setMcpJustRefreshed] = useState(false);

  useEffect(() => {
    // Initial load
    refreshMcpStatus();

    // Setup interval for polling
    const timer = setInterval(refreshMcpStatus, MCP_TIMING.POLL_INTERVAL_MS);

    let unlisten: (() => void) | undefined;

    const setupListener = async () => {
      try {
        const fn = await listen("mcp-artifacts-refreshed", () => {
          refreshMcpStatus();
          if (onRefresh) {
            onRefresh();
          }
          setMcpJustRefreshed(true);
          setTimeout(() => setMcpJustRefreshed(false), MCP_TIMING.REFRESH_ANIMATION_DURATION_MS);
        });
        unlisten = fn;
      } catch (err) {
        console.error("Failed to setup MCP refresh listener", err);
      }
    };

    setupListener();

    return () => {
      clearInterval(timer);
      if (unlisten) unlisten();
    };
  }, [refreshMcpStatus, onRefresh]);

  return {
    mcpStatus,
    mcpJustRefreshed,
    refreshMcpStatus,
  };
}
