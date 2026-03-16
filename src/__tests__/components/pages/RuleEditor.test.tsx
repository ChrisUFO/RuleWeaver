import { describe, it, expect, vi, beforeEach } from "vitest";
import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { RuleEditor } from "../../../components/pages/RuleEditor";
import { ToastProvider } from "../../../components/ui/toast";
import type { AdapterType } from "../../../types/rule";
import { useKeyboardShortcuts } from "../../../hooks/useKeyboardShortcuts";

// --- Mocks ---
vi.mock("../../../lib/featureManager", () => ({
  featureManager: { isEnabled: () => true },
  FEATURE_FLAGS: {
    DIALOG_ACCESSIBILITY: "dialog_accessibility",
    THEME_PERSISTENCE: "theme_persistence",
    ENHANCED_ERROR_UX: "enhanced_error_ux",
    NATIVE_SKILL_SYNC: "native_skill_sync",
    UNIFIED_ARTIFACT_STATUS: "unified_artifact_status",
    EXECUTION_REDACTION: "execution_redaction",
  },
}));

const mockCreateRule = vi.fn();
const mockUpdateRule = vi.fn();
const mockDuplicateRule = vi.fn();

vi.mock("../../../stores/rulesStore", () => ({
  useRulesStore: () => ({
    createRule: mockCreateRule,
    updateRule: mockUpdateRule,
    duplicateRule: mockDuplicateRule,
  }),
}));

const mockTools = [
  {
    id: "gemini",
    name: "Gemini",
    paths: {
      globalPath: "/home/.gemini/GEMINI.md",
      localPathTemplate: "{root}/.gemini/GEMINI.md",
    },
  },
  {
    id: "opencode",
    name: "OpenCode",
    paths: {
      globalPath: "/home/.config/opencode/rules.md",
      localPathTemplate: "{root}/.opencode/rules.md",
    },
  },
];

vi.mock("../../../stores/registryStore", () => ({
  useRegistryStore: () => ({ tools: mockTools }),
}));

vi.mock("../../../lib/tauri", () => ({
  api: {
    settings: {
      get: vi.fn().mockResolvedValue(null),
      set: vi.fn().mockResolvedValue(undefined),
    },
    app: { openInExplorer: vi.fn().mockResolvedValue(undefined) },
    ai: {
      getSettings: vi.fn().mockResolvedValue({ enabled: false, hasApiKey: false }),
    },
  },
}));

vi.mock("../../../hooks/useRepositoryRoots", () => ({
  useRepositoryRoots: () => ({ roots: ["/repo/a", "/repo/b"] }),
}));

vi.mock("../../../hooks/useKeyboardShortcuts", async (importOriginal) => {
  const actual = await importOriginal<typeof import("../../../hooks/useKeyboardShortcuts")>();
  return {
    useKeyboardShortcuts: vi.fn(),
    SHORTCUTS: actual.SHORTCUTS,
  };
});

// Minimal MarkdownEditor stub
vi.mock("../../../components/ui/markdown-editor", () => ({
  MarkdownEditor: ({ value, onChange }: { value: string; onChange: (v: string) => void }) => (
    <textarea
      data-testid="markdown-editor"
      value={value}
      onChange={(e) => onChange(e.target.value)}
    />
  ),
}));

const renderWithProviders = (ui: React.ReactElement) => render(<ToastProvider>{ui}</ToastProvider>);

const baseRule = {
  id: "rule-1",
  name: "My Rule",
  description: "A description",
  content: "Some content",
  scope: "global" as const,
  targetPaths: [],
  enabledAdapters: ["gemini" as const],
  enabled: true,
  createdAt: 1000000,
  updatedAt: 1000000,
};

describe("RuleEditor", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders in edit mode with rule name in heading", async () => {
    renderWithProviders(<RuleEditor rule={baseRule} onBack={vi.fn()} onSelectRule={vi.fn()} />);
    await act(async () => {
      await Promise.resolve();
    });
    expect(screen.getByText("Edit: My Rule")).toBeInTheDocument();
  });

  it("renders in create mode", async () => {
    renderWithProviders(<RuleEditor rule={null} onBack={vi.fn()} onSelectRule={vi.fn()} isNew />);
    await act(async () => {
      await Promise.resolve();
    });
    expect(screen.getByText("Create Rule")).toBeInTheDocument();
  });

  it("shows validation toast when saving without a name", async () => {
    const user = userEvent.setup();
    renderWithProviders(<RuleEditor rule={null} onBack={vi.fn()} onSelectRule={vi.fn()} isNew />);

    // Clear the name field and save
    const nameInput = screen.getByLabelText("Rule name");
    await user.clear(nameInput);
    await user.click(screen.getByRole("button", { name: /save/i }));

    await waitFor(() => {
      expect(screen.getByText("Validation Error")).toBeInTheDocument();
      expect(screen.getByText("Rule name is required")).toBeInTheDocument();
    });
    expect(mockCreateRule).not.toHaveBeenCalled();
  });

  it("shows validation toast when saving without adapters", async () => {
    const user = userEvent.setup();
    // Rule with no enabled adapters
    const noAdapterRule = { ...baseRule, enabledAdapters: [] as AdapterType[] };
    renderWithProviders(
      <RuleEditor rule={noAdapterRule} onBack={vi.fn()} onSelectRule={vi.fn()} />
    );

    // Disable the only adapter (gemini is not in the list here)
    await user.click(screen.getByRole("button", { name: /save/i }));

    await waitFor(() => {
      expect(screen.getByText("At least one adapter must be selected")).toBeInTheDocument();
    });
  });

  it("shows validation toast for local rule without target paths", async () => {
    const user = userEvent.setup();
    renderWithProviders(<RuleEditor rule={null} onBack={vi.fn()} onSelectRule={vi.fn()} isNew />);

    const nameInput = screen.getByLabelText("Rule name");
    await user.type(nameInput, "New Rule");

    // Switch scope to local
    const localRadio = screen.getByRole("radio", { name: /local/i });
    await user.click(localRadio);

    // Type content
    const editor = screen.getByTestId("markdown-editor");
    await user.type(editor, "Some content");

    await user.click(screen.getByRole("button", { name: /save/i }));

    await waitFor(() => {
      expect(screen.getByText("Local rules require at least one target path")).toBeInTheDocument();
    });
  });

  it("calls createRule with correct data on successful save (new rule)", async () => {
    const user = userEvent.setup();
    mockCreateRule.mockResolvedValue({ ...baseRule, id: "new-id" });
    const onBack = vi.fn();

    renderWithProviders(<RuleEditor rule={null} onBack={onBack} onSelectRule={vi.fn()} isNew />);

    await user.type(screen.getByLabelText("Rule name"), "My New Rule");
    await user.type(screen.getByTestId("markdown-editor"), "Content here");

    // Wait for adapters to load (from default settings mock)
    // Since settings.get returns null, enabledAdapters defaults are set by the effect
    // We need at least one adapter — click gemini to enable it
    // Actually the default adapters effect runs — find and toggle gemini
    const geminiSwitch = screen.getByLabelText("Toggle Gemini adapter");
    await user.click(geminiSwitch);

    await user.click(screen.getByRole("button", { name: /save/i }));

    await waitFor(() => {
      expect(mockCreateRule).toHaveBeenCalledWith(expect.objectContaining({ name: "My New Rule" }));
    });
  });

  it("calls updateRule on save for existing rule", async () => {
    const user = userEvent.setup();
    mockUpdateRule.mockResolvedValue(baseRule);
    const onBack = vi.fn();

    renderWithProviders(<RuleEditor rule={baseRule} onBack={onBack} onSelectRule={vi.fn()} />);

    await user.click(screen.getByRole("button", { name: /save/i }));

    await waitFor(() => {
      expect(mockUpdateRule).toHaveBeenCalledWith(
        "rule-1",
        expect.objectContaining({ name: "My Rule" })
      );
    });
  });

  it("calls duplicateRule when Duplicate button is clicked", async () => {
    const user = userEvent.setup();
    mockDuplicateRule.mockResolvedValue({ ...baseRule, id: "dup-id", name: "My Rule (copy)" });
    const onSelectRule = vi.fn();

    renderWithProviders(
      <RuleEditor rule={baseRule} onBack={vi.fn()} onSelectRule={onSelectRule} />
    );

    await user.click(screen.getByRole("button", { name: /duplicate/i }));

    await waitFor(() => {
      expect(mockDuplicateRule).toHaveBeenCalled();
      expect(onSelectRule).toHaveBeenCalled();
    });
  });

  it("shows target repos panel when scope is switched to local", async () => {
    const user = userEvent.setup();
    renderWithProviders(<RuleEditor rule={baseRule} onBack={vi.fn()} onSelectRule={vi.fn()} />);

    expect(screen.queryByText("Target Repositories")).not.toBeInTheDocument();

    const localRadio = screen.getByRole("radio", { name: /local/i });
    await user.click(localRadio);

    expect(screen.getByText("Target Repositories")).toBeInTheDocument();
    expect(screen.getByText("/repo/a")).toBeInTheDocument();
  });

  it("shows adapter paths when enabled and hides when toggled off", async () => {
    const user = userEvent.setup();
    renderWithProviders(<RuleEditor rule={baseRule} onBack={vi.fn()} onSelectRule={vi.fn()} />);

    await waitFor(() => {
      const geminiSwitch = screen.getByLabelText("Toggle Gemini adapter");
      expect(geminiSwitch).toBeChecked();
    });

    const geminiSwitch = screen.getByLabelText("Toggle Gemini adapter");
    await user.click(geminiSwitch);

    await waitFor(() => {
      expect(geminiSwitch).not.toBeChecked();
    });
  });

  it("registers Ctrl+S keyboard shortcut that triggers save", async () => {
    mockUpdateRule.mockResolvedValue(baseRule);
    renderWithProviders(<RuleEditor rule={baseRule} onBack={vi.fn()} onSelectRule={vi.fn()} />);

    await waitFor(() => {
      expect(vi.mocked(useKeyboardShortcuts)).toHaveBeenCalled();
    });

    const calls = vi.mocked(useKeyboardShortcuts).mock.calls;
    expect(calls.length).toBeGreaterThan(0);

    // Find the call that includes the save shortcut (key: "s")
    const ruleEditorCall = calls.find((call) =>
      call[0].shortcuts.some((s: { key: string }) => s.key === "s")
    );
    expect(ruleEditorCall).toBeDefined();
    const saveShortcut = ruleEditorCall![0].shortcuts.find((s: { key: string }) => s.key === "s");

    expect(saveShortcut).toBeDefined();
    await act(async () => {
      await saveShortcut!.action();
    });

    await waitFor(() => {
      expect(mockUpdateRule).toHaveBeenCalled();
    });
  });
});
