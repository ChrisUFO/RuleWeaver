import { describe, it, expect, vi, beforeEach } from "vitest";
import { useRulesStore } from "@/stores/rulesStore";
import { api } from "@/lib/tauri";
import type { Rule, AdapterType } from "@/types/rule";

vi.mock("@/lib/tauri", () => ({
  api: {
    rules: {
      getAll: vi.fn(),
      getById: vi.fn(),
      create: vi.fn(),
      update: vi.fn(),
      delete: vi.fn(),
      toggle: vi.fn(),
    },
  },
}));

function createMockRule(overrides?: Partial<Rule>): Rule {
  return {
    id: "1",
    name: "Test Rule",
    content: "Test content",
    scope: "global",
    targetPaths: null,
    enabledAdapters: ["gemini"] as AdapterType[],
    enabled: true,
    createdAt: Date.now(),
    updatedAt: Date.now(),
    ...overrides,
  };
}

describe("rulesStore", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useRulesStore.setState({
      rules: [],
      selectedRule: null,
      isLoading: false,
      error: null,
    });
  });

  it("starts with empty rules", () => {
    const state = useRulesStore.getState();
    expect(state.rules).toEqual([]);
    expect(state.selectedRule).toBeNull();
    expect(state.isLoading).toBe(false);
  });

  it("fetches rules successfully", async () => {
    const mockRules = [createMockRule()];
    vi.mocked(api.rules.getAll).mockResolvedValue(mockRules);

    const { fetchRules } = useRulesStore.getState();
    await fetchRules();

    const state = useRulesStore.getState();
    expect(state.rules).toEqual(mockRules);
    expect(state.isLoading).toBe(false);
  });

  it("handles fetch errors", async () => {
    vi.mocked(api.rules.getAll).mockRejectedValue(new Error("Fetch failed"));

    const { fetchRules } = useRulesStore.getState();
    await fetchRules();

    const state = useRulesStore.getState();
    expect(state.error).toBe("Fetch failed");
    expect(state.isLoading).toBe(false);
  });

  it("creates a rule successfully", async () => {
    const newRule = createMockRule({
      id: "2",
      name: "New Rule",
      content: "New content",
      scope: "local",
      targetPaths: ["/path/to/repo"],
      enabledAdapters: ["opencode"] as AdapterType[],
    });
    vi.mocked(api.rules.create).mockResolvedValue(newRule);

    const { createRule } = useRulesStore.getState();
    const result = await createRule({
      name: "New Rule",
      content: "New content",
      scope: "local",
      targetPaths: ["/path/to/repo"],
      enabledAdapters: ["opencode"],
    });

    expect(result).toEqual(newRule);
    const state = useRulesStore.getState();
    expect(state.rules).toContainEqual(newRule);
  });

  it("deletes a rule successfully", async () => {
    const existingRule = createMockRule();
    useRulesStore.setState({ rules: [existingRule] });
    vi.mocked(api.rules.delete).mockResolvedValue(undefined);

    const { deleteRule } = useRulesStore.getState();
    await deleteRule("1");

    const state = useRulesStore.getState();
    expect(state.rules).not.toContainEqual(existingRule);
  });

  it("selects a rule", () => {
    const rule = createMockRule();

    const { selectRule } = useRulesStore.getState();
    selectRule(rule);

    const state = useRulesStore.getState();
    expect(state.selectedRule).toEqual(rule);
  });

  it("clears error", () => {
    useRulesStore.setState({ error: "Some error" });

    const { clearError } = useRulesStore.getState();
    clearError();

    const state = useRulesStore.getState();
    expect(state.error).toBeNull();
  });
});
