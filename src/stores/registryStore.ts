import { create } from "zustand";
import { api } from "@/lib/tauri";
import type { ToolEntry } from "@/types/rule";

interface RegistryState {
  tools: ToolEntry[];
  isLoading: boolean;
  error: string | null;
  fetchTools: () => Promise<void>;
}

export const useRegistryStore = create<RegistryState>((set) => ({
  tools: [],
  isLoading: true,
  error: null,
  fetchTools: async () => {
    set({ isLoading: true, error: null });
    try {
      const tools = await api.registry.getTools();
      set({ tools, isLoading: false });
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : "Failed to fetch tools registry",
        isLoading: false,
      });
    }
  },
}));
