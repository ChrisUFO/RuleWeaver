import { create } from "zustand";
import type { Rule, CreateRuleInput, UpdateRuleInput } from "@/types/rule";
import { api } from "@/lib/tauri";

interface RulesState {
  rules: Rule[];
  selectedRule: Rule | null;
  isLoading: boolean;
  error: string | null;
  recentlyDeleted: Rule | null;

  fetchRules: () => Promise<void>;
  createRule: (input: CreateRuleInput) => Promise<Rule>;
  updateRule: (id: string, input: UpdateRuleInput) => Promise<Rule>;
  deleteRule: (id: string) => Promise<void>;
  duplicateRule: (rule: Rule) => Promise<Rule>;
  restoreRecentlyDeleted: () => Promise<void>;
  toggleRule: (id: string, enabled: boolean) => Promise<void>;
  selectRule: (rule: Rule | null) => void;
  setSelectedRuleContent: (content: string) => void;
  clearError: () => void;
  clearRecentlyDeleted: () => void;
}

export const useRulesStore = create<RulesState>((set, get) => ({
  rules: [],
  selectedRule: null,
  isLoading: false,
  error: null,
  recentlyDeleted: null,

  fetchRules: async () => {
    set({ isLoading: true, error: null });
    try {
      const rules = await api.rules.getAll();
      set({ rules, isLoading: false });
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : "Failed to fetch rules",
        isLoading: false,
      });
    }
  },

  createRule: async (input: CreateRuleInput) => {
    set({ isLoading: true, error: null });
    try {
      const rule = await api.rules.create(input);
      set((state) => ({
        rules: [...state.rules, rule],
        isLoading: false,
      }));
      return rule;
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : "Failed to create rule",
        isLoading: false,
      });
      throw error;
    }
  },

  updateRule: async (id: string, input: UpdateRuleInput) => {
    set({ isLoading: true, error: null });
    try {
      const updatedRule = await api.rules.update(id, input);
      set((state) => ({
        rules: state.rules.map((r) => (r.id === id ? updatedRule : r)),
        selectedRule: state.selectedRule?.id === id ? updatedRule : state.selectedRule,
        isLoading: false,
      }));
      return updatedRule;
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : "Failed to update rule",
        isLoading: false,
      });
      throw error;
    }
  },

  deleteRule: async (id: string) => {
    set({ isLoading: true, error: null });
    try {
      const ruleToDelete = get().rules.find((r) => r.id === id);
      await api.rules.delete(id);
      set((state) => ({
        rules: state.rules.filter((r) => r.id !== id),
        selectedRule: state.selectedRule?.id === id ? null : state.selectedRule,
        recentlyDeleted: ruleToDelete || null,
        isLoading: false,
      }));
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : "Failed to delete rule",
        isLoading: false,
      });
      throw error;
    }
  },

  duplicateRule: async (rule: Rule) => {
    set({ isLoading: true, error: null });
    try {
      const newRule = await api.rules.create({
        name: `${rule.name} (Copy)`,
        content: rule.content,
        scope: rule.scope,
        targetPaths: rule.targetPaths ?? undefined,
        enabledAdapters: rule.enabledAdapters,
      });
      set((state) => ({
        rules: [...state.rules, newRule],
        isLoading: false,
      }));
      return newRule;
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : "Failed to duplicate rule",
        isLoading: false,
      });
      throw error;
    }
  },

  restoreRecentlyDeleted: async () => {
    const { recentlyDeleted } = get();
    if (!recentlyDeleted) return;

    try {
      const restoredRule = await api.rules.create({
        name: recentlyDeleted.name,
        content: recentlyDeleted.content,
        scope: recentlyDeleted.scope,
        targetPaths: recentlyDeleted.targetPaths ?? undefined,
        enabledAdapters: recentlyDeleted.enabledAdapters,
      });
      set((state) => ({
        rules: [...state.rules, restoredRule],
        recentlyDeleted: null,
      }));
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : "Failed to restore rule",
      });
    }
  },

  toggleRule: async (id: string, enabled: boolean) => {
    try {
      const updatedRule = await api.rules.toggle(id, enabled);
      set((state) => ({
        rules: state.rules.map((r) => (r.id === id ? updatedRule : r)),
        selectedRule: state.selectedRule?.id === id ? updatedRule : state.selectedRule,
      }));
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : "Failed to toggle rule",
      });
    }
  },

  selectRule: (rule: Rule | null) => {
    set({ selectedRule: rule });
  },

  setSelectedRuleContent: (content: string) => {
    set((state) => {
      if (!state.selectedRule) return state;
      return {
        selectedRule: { ...state.selectedRule, content },
      };
    });
  },

  clearError: () => {
    set({ error: null });
  },

  clearRecentlyDeleted: () => {
    set({ recentlyDeleted: null });
  },
}));
