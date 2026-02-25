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
    description: "Test description",
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
      description: "New description",
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

  it("updates a rule successfully", async () => {
    const existingRule = createMockRule();
    useRulesStore.setState({ rules: [existingRule] });

    const updatedRule = { ...existingRule, name: "Updated Rule", content: "Updated content" };
    vi.mocked(api.rules.update).mockResolvedValue(updatedRule);

    const { updateRule } = useRulesStore.getState();
    const result = await updateRule("1", { name: "Updated Rule", content: "Updated content" });

    expect(result).toEqual(updatedRule);
    const state = useRulesStore.getState();
    expect(state.rules[0].name).toBe("Updated Rule");
  });

  it("updates rule error and throws", async () => {
    const existingRule = createMockRule();
    useRulesStore.setState({ rules: [existingRule] });
    vi.mocked(api.rules.update).mockRejectedValue(new Error("Update failed"));

    const { updateRule } = useRulesStore.getState();
    await expect(updateRule("1", { name: "Fail" })).rejects.toThrow("Update failed");

    const state = useRulesStore.getState();
    expect(state.error).toBe("Update failed");
  });

  it("toggles rule enabled state", async () => {
    const existingRule = createMockRule({ enabled: true });
    useRulesStore.setState({ rules: [existingRule] });

    const toggledRule = { ...existingRule, enabled: false };
    vi.mocked(api.rules.toggle).mockResolvedValue(toggledRule);

    const { toggleRule } = useRulesStore.getState();
    await toggleRule("1", false);

    const state = useRulesStore.getState();
    expect(state.rules[0].enabled).toBe(false);
  });

  it("toggle rule handles error gracefully", async () => {
    const existingRule = createMockRule({ enabled: true });
    useRulesStore.setState({ rules: [existingRule] });
    vi.mocked(api.rules.toggle).mockRejectedValue(new Error("Toggle failed"));

    const { toggleRule } = useRulesStore.getState();
    await toggleRule("1", false);

    const state = useRulesStore.getState();
    expect(state.error).toBe("Toggle failed");
  });

  it("duplicates a rule successfully", async () => {
    const existingRule = createMockRule({ name: "Original", scope: "global" });
    useRulesStore.setState({ rules: [existingRule] });

    const duplicatedRule = {
      ...existingRule,
      id: "2",
      name: "Original (Copy)",
      createdAt: Date.now(),
      updatedAt: Date.now(),
    };
    vi.mocked(api.rules.create).mockResolvedValue(duplicatedRule);

    const { duplicateRule } = useRulesStore.getState();
    const result = await duplicateRule(existingRule);

    expect(result.name).toBe("Original (Copy)");
    const state = useRulesStore.getState();
    expect(state.rules).toHaveLength(2);
  });

  it("duplicate rule handles error gracefully", async () => {
    const existingRule = createMockRule();
    useRulesStore.setState({ rules: [existingRule] });
    vi.mocked(api.rules.create).mockRejectedValue(new Error("Duplicate failed"));

    const { duplicateRule } = useRulesStore.getState();
    await expect(duplicateRule(existingRule)).rejects.toThrow();

    const state = useRulesStore.getState();
    expect(state.error).toBe("Duplicate failed");
  });

  it("delete rule handles error gracefully", async () => {
    const existingRule = createMockRule();
    useRulesStore.setState({ rules: [existingRule] });
    vi.mocked(api.rules.delete).mockRejectedValue(new Error("Delete failed"));

    const { deleteRule } = useRulesStore.getState();
    await expect(deleteRule("1")).rejects.toThrow();

    const state = useRulesStore.getState();
    expect(state.error).toBe("Delete failed");
  });

  it("stores recently deleted rule for restoration", async () => {
    const existingRule = createMockRule();
    useRulesStore.setState({ rules: [existingRule] });
    vi.mocked(api.rules.delete).mockResolvedValue(undefined);

    const { deleteRule } = useRulesStore.getState();
    await deleteRule("1");

    const state = useRulesStore.getState();
    expect(state.recentlyDeleted).toEqual(existingRule);
  });

  it("restores recently deleted rule", async () => {
    const deletedRule = createMockRule({ enabled: false });
    useRulesStore.setState({ recentlyDeleted: deletedRule });

    const restoredRule = {
      ...deletedRule,
      id: "new-id",
      createdAt: Date.now(),
      updatedAt: Date.now(),
    };
    vi.mocked(api.rules.create).mockResolvedValue(restoredRule);
    vi.mocked(api.rules.toggle).mockResolvedValue({ ...restoredRule, enabled: false });

    const { restoreRecentlyDeleted } = useRulesStore.getState();
    await restoreRecentlyDeleted();

    const state = useRulesStore.getState();
    expect(state.rules).toHaveLength(1);
    expect(state.recentlyDeleted).toBeNull();
  });

  it("restoreRecentlyDeleted does nothing when nothing deleted", async () => {
    useRulesStore.setState({ recentlyDeleted: null });

    const { restoreRecentlyDeleted } = useRulesStore.getState();
    await restoreRecentlyDeleted();

    const state = useRulesStore.getState();
    expect(state.rules).toHaveLength(0);
  });

  it("clears recently deleted", () => {
    const deletedRule = createMockRule();
    useRulesStore.setState({ recentlyDeleted: deletedRule });

    const { clearRecentlyDeleted } = useRulesStore.getState();
    clearRecentlyDeleted();

    const state = useRulesStore.getState();
    expect(state.recentlyDeleted).toBeNull();
  });

  it("setSelectedRuleContent updates content of selected rule", () => {
    const rule = createMockRule({ content: "Original" });
    useRulesStore.setState({ selectedRule: rule });

    const { setSelectedRuleContent } = useRulesStore.getState();
    setSelectedRuleContent("New content");

    const state = useRulesStore.getState();
    expect(state.selectedRule?.content).toBe("New content");
  });

  it("setSelectedRuleContent does nothing when no rule selected", () => {
    useRulesStore.setState({ selectedRule: null });

    const { setSelectedRuleContent } = useRulesStore.getState();
    setSelectedRuleContent("New content");

    const state = useRulesStore.getState();
    expect(state.selectedRule).toBeNull();
  });

  it("createRule handles error and sets error state", async () => {
    vi.mocked(api.rules.create).mockRejectedValue(new Error("Create failed"));

    const { createRule } = useRulesStore.getState();
    await expect(
      createRule({
        name: "Test",
        description: "Test",
        content: "Content",
        scope: "global",
        enabledAdapters: ["gemini"],
      })
    ).rejects.toThrow();

    const state = useRulesStore.getState();
    expect(state.error).toBe("Create failed");
  });
});
