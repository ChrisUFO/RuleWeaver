import { create } from "zustand";
import type { McpStatus } from "@/types/command";
import { api } from "@/lib/tauri";

interface McpState {
  mcpStatus: McpStatus | null;
  isMcpLoading: boolean;
  error: string | null;
  refreshMcpStatus: () => Promise<void>;
  setMcpStatus: (status: McpStatus | null) => void;
}

export const useMcpStore = create<McpState>((set) => ({
  mcpStatus: null,
  isMcpLoading: false,
  error: null,
  refreshMcpStatus: async () => {
    set({ isMcpLoading: true });
    try {
      const status = await api.mcp.getStatus();
      set({ mcpStatus: status, error: null });
    } catch (err) {
      set({ error: err instanceof Error ? err.message : "Unknown error" });
    } finally {
      set({ isMcpLoading: false });
    }
  },
  setMcpStatus: (status) => set({ mcpStatus: status }),
}));
