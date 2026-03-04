import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act } from "@testing-library/react";

import { useRuleEditorState } from "@/hooks/useRuleEditorState";
import type { ToolEntry, Rule } from "@/types/rule";

const mockCreateRule = vi.fn();
const mockUpdateRule = vi.fn();
const mockDuplicateRule = vi.fn();
const mockAddToast = vi.fn();
const mockSettingsGet = vi.fn();

let mockTools: ToolEntry[] = [];

vi.mock("@/stores/rulesStore", () => ({
  useRulesStore: () => ({
    createRule: mockCreateRule,
    updateRule: mockUpdateRule,
    duplicateRule: mockDuplicateRule,
  }),
}));

vi.mock("@/stores/registryStore", () => ({
  useRegistryStore: () => ({ tools: mockTools }),
}));

vi.mock("@/components/ui/toast", () => ({
  useToast: () => ({ addToast: mockAddToast }),
}));

vi.mock("@/hooks/useRepositoryRoots", () => ({
  useRepositoryRoots: () => ({ roots: ["/repo/a"] }),
}));

vi.mock("@/lib/tauri", () => ({
  api: {
    settings: {
      get: (...args: unknown[]) => mockSettingsGet(...args),
    },
    app: {
      openInExplorer: vi.fn().mockResolvedValue(undefined),
    },
  },
}));

const makeTool = (tool: Partial<ToolEntry> & Pick<ToolEntry, "id" | "name">): ToolEntry => ({
  id: tool.id,
  name: tool.name,
  description: tool.description ?? "",
  icon: tool.icon ?? "",
  capabilities:
    tool.capabilities ??
    ({
      supportsRules: true,
      supportsCommandStubs: false,
      supportsSlashCommands: false,
      supportsSkills: false,
      supportsGlobalScope: true,
      supportsLocalScope: true,
    } as ToolEntry["capabilities"]),
  paths: tool.paths ?? {
    globalPath: "",
    localPathTemplate: "",
  },
  ruleFileModel: tool.ruleFileModel,
  fileFormat: tool.fileFormat ?? "markdown",
});

const baseProps = {
  rule: null as Rule | null,
  isNew: true,
  onBack: vi.fn(),
  onSelectRule: vi.fn(),
};

describe("useRuleEditorState adapter path + preview", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockSettingsGet.mockResolvedValue(JSON.stringify(["opencode"]));
    mockTools = [];
  });

  it("returns per-rule global target path for per_rule_dir adapters", () => {
    mockTools = [
      makeTool({
        id: "opencode",
        name: "OpenCode",
        paths: {
          globalPath: "~/.config/opencode/AGENTS.md",
          localPathTemplate: ".opencode/AGENTS.md",
          globalRulesDir: "~/.config/opencode/rules",
          localRulesDirTemplate: ".opencode/rules",
          globalRuleFileModel: "per_rule_dir",
          localRuleFileModel: "per_rule_dir",
        },
        ruleFileModel: "per_rule_dir",
      }),
    ];

    const { result } = renderHook(() => useRuleEditorState(baseProps));

    act(() => {
      result.current.setName("My Rule!!!");
      result.current.setScope("global");
    });

    expect(result.current.getAdapterPath("opencode")).toBe("~/.config/opencode/rules/my-rule.md");
  });

  it("returns per-rule local target path for per_rule_dir adapters", () => {
    mockTools = [
      makeTool({
        id: "opencode",
        name: "OpenCode",
        paths: {
          globalPath: "~/.config/opencode/AGENTS.md",
          localPathTemplate: ".opencode/AGENTS.md",
          globalRulesDir: "~/.config/opencode/rules",
          localRulesDirTemplate: ".opencode/rules",
          globalRuleFileModel: "per_rule_dir",
          localRuleFileModel: "per_rule_dir",
        },
        ruleFileModel: "per_rule_dir",
      }),
    ];

    const { result } = renderHook(() => useRuleEditorState(baseProps));

    act(() => {
      result.current.setName("Repo Rule");
      result.current.setScope("local");
      result.current.toggleTargetPath("/repo/a", true);
    });

    expect(result.current.getAdapterPath("opencode")).toBe("/repo/a/.opencode/rules/repo-rule.md");
  });

  it("returns single-file path for single_file adapters", () => {
    mockTools = [
      makeTool({
        id: "claude-code",
        name: "Claude Code",
        paths: {
          globalPath: "~/.claude/CLAUDE.md",
          localPathTemplate: ".claude/CLAUDE.md",
          globalRuleFileModel: "single_file",
          localRuleFileModel: "single_file",
        },
        ruleFileModel: "single_file",
      }),
    ];

    const { result } = renderHook(() => useRuleEditorState(baseProps));
    expect(result.current.getAdapterPath("claude-code")).toBe("~/.claude/CLAUDE.md");
  });

  it("uses rule-id fallback slug when sanitized rule name is empty", () => {
    const existingRule = {
      id: "rule-123",
      name: "Legacy",
      description: "",
      content: "x",
      scope: "global",
      targetPaths: null,
      enabledAdapters: ["opencode"],
      enabled: true,
      createdAt: 1,
      updatedAt: 1,
    } as Rule;

    mockTools = [
      makeTool({
        id: "opencode",
        name: "OpenCode",
        paths: {
          globalPath: "~/.config/opencode/AGENTS.md",
          localPathTemplate: ".opencode/AGENTS.md",
          globalRulesDir: "~/.config/opencode/rules",
          localRulesDirTemplate: ".opencode/rules",
          globalRuleFileModel: "per_rule_dir",
          localRuleFileModel: "per_rule_dir",
        },
        ruleFileModel: "per_rule_dir",
      }),
    ];

    const { result } = renderHook(() =>
      useRuleEditorState({ ...baseProps, isNew: false, rule: existingRule })
    );

    act(() => {
      result.current.setName("!!!");
      result.current.setScope("global");
    });

    expect(result.current.getAdapterPath("opencode")).toBe(
      "~/.config/opencode/rules/rule-rule-123.md"
    );
  });

  it("generates marker + rule header preview for both file models", () => {
    mockTools = [
      makeTool({
        id: "opencode",
        name: "OpenCode",
        paths: {
          globalPath: "~/.config/opencode/AGENTS.md",
          localPathTemplate: ".opencode/AGENTS.md",
          globalRulesDir: "~/.config/opencode/rules",
          localRulesDirTemplate: ".opencode/rules",
          globalRuleFileModel: "per_rule_dir",
          localRuleFileModel: "per_rule_dir",
        },
        ruleFileModel: "per_rule_dir",
      }),
      makeTool({
        id: "claude-code",
        name: "Claude Code",
        paths: {
          globalPath: "~/.claude/CLAUDE.md",
          localPathTemplate: ".claude/CLAUDE.md",
          globalRuleFileModel: "single_file",
          localRuleFileModel: "single_file",
        },
        ruleFileModel: "single_file",
      }),
    ];

    const { result } = renderHook(() => useRuleEditorState(baseProps));

    act(() => {
      result.current.setName("Preview Rule");
      result.current.setContent("Always test your code.");
      result.current.setPreviewAdapter("opencode");
    });
    const perRulePreview = result.current.generatePreview();
    expect(perRulePreview).toContain("Generated by RuleWeaver - Do not edit manually");
    expect(perRulePreview).toContain("## Preview Rule");
    expect(perRulePreview).toContain("Always test your code.");

    act(() => {
      result.current.setPreviewAdapter("claude-code");
    });
    const singleFilePreview = result.current.generatePreview();
    expect(singleFilePreview).toContain("Generated by RuleWeaver - Do not edit manually");
    expect(singleFilePreview).toContain("## Preview Rule");
    expect(singleFilePreview).toContain("Always test your code.");
    expect(singleFilePreview).not.toContain("<!-- Rule:");
  });
});
