import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Settings } from "../../components/pages/Settings";
import { ToastProvider } from "../../components/ui/toast";
import { useSettingsState } from "../../hooks/useSettingsState";
import type { UseSettingsStateReturn } from "../../hooks/useSettingsState";

// Mock the complex settings state hook to avoid Tauri plugin calls
vi.mock("../../hooks/useSettingsState", () => ({
  useSettingsState: vi.fn(),
}));

// Mock Tauri plugins that settings may reference even when hook is mocked
vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn(),
  save: vi.fn(),
}));
vi.mock("@tauri-apps/plugin-updater", () => ({
  check: vi.fn(),
}));
vi.mock("@tauri-apps/plugin-autostart", () => ({
  enable: vi.fn(),
  disable: vi.fn(),
  isEnabled: vi.fn().mockResolvedValue(false),
}));

// AdapterSettingsCard reads from registryStore directly
vi.mock("../../stores/registryStore", () => ({
  useRegistryStore: () => ({
    tools: [
      {
        id: "claude_code",
        name: "Claude Code",
        description: "Claude Code adapter",
        paths: {
          globalPath: "~/.claude/CLAUDE.md",
          localPathTemplate: ".claude/CLAUDE.md",
        },
      },
      {
        id: "gemini",
        name: "Gemini",
        description: "Gemini adapter",
        paths: {
          globalPath: "~/.gemini/GEMINI.md",
          localPathTemplate: ".gemini/GEMINI.md",
        },
      },
    ],
    isLoading: false,
    error: null,
    fetchTools: vi.fn(),
  }),
}));

function makeHandlers() {
  return {
    toggleAdapter: vi.fn(),
    saveSettings: vi.fn().mockResolvedValue(undefined),
    handleOpenAppData: vi.fn(),
    addRepositoryRoot: vi.fn().mockResolvedValue(undefined),
    removeRepositoryRoot: vi.fn().mockResolvedValue(undefined),
    saveRepositoryRoots: vi.fn().mockResolvedValue(undefined),
    migrateToFileStorage: vi.fn(),
    rollbackMigration: vi.fn(),
    verifyMigration: vi.fn(),
    startMcp: vi.fn(),
    stopMcp: vi.fn(),
    refreshMcpStatus: vi.fn(),
    toggleMcpAutoStart: vi.fn(),
    toggleMinimizeToTray: vi.fn(),
    toggleLaunchOnStartup: vi.fn(),
    handleExport: vi.fn(),
    handleImport: vi.fn(),
    executeImport: vi.fn(),
    handleCheckUpdates: vi.fn(),
    confirmUpdate: vi.fn(),
    syncAllSlashCommands: vi.fn(),
    setIsImportDialogOpen: vi.fn(),
    setImportMode: vi.fn(),
    setIsUpdateDialogOpen: vi.fn(),
  };
}

function makeBaseState(overrides: Partial<UseSettingsStateReturn> = {}): UseSettingsStateReturn {
  return {
    appDataPath: "/mock/data",
    appVersion: "1.0.0",
    isLoading: false,
    adapterSettings: { claude_code: true, gemini: true },
    hasChanges: false,
    isSaving: false,
    repositoryRoots: [],
    repoPathsDirty: false,
    isSavingRepos: false,
    storageMode: "sqlite",
    storageInfo: null,
    isMigratingStorage: false,
    backupPath: "",
    migrationProgress: null,
    isRollingBack: false,
    isVerifyingMigration: false,
    mcpStatus: null,
    mcpInstructions: null,
    isMcpLoading: false,
    mcpAutoStart: false,
    minimizeToTray: true,
    launchOnStartup: false,
    mcpLogs: [],
    isExporting: false,
    isImporting: false,
    importPreview: null,
    isImportDialogOpen: false,
    importMode: "overwrite",
    isCheckingUpdates: false,
    updateData: null,
    isUpdateDialogOpen: false,
    isUpdating: false,
    handlers: makeHandlers(),
    ...overrides,
  };
}

const renderWithProviders = (ui: React.ReactElement) => render(<ToastProvider>{ui}</ToastProvider>);

describe("Settings lifecycle", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(useSettingsState).mockReturnValue(makeBaseState());
  });

  it("clicking Add Repository on Context tab calls handlers.addRepositoryRoot", async () => {
    const user = userEvent.setup();
    const handlers = makeHandlers();
    vi.mocked(useSettingsState).mockReturnValue(makeBaseState({ handlers }));

    renderWithProviders(<Settings />);

    // Navigate to Context tab
    const contextTab = screen.getByRole("button", { name: /context/i });
    await user.click(contextTab);

    await waitFor(() => {
      expect(screen.getByRole("button", { name: /add repository/i })).toBeInTheDocument();
    });

    await user.click(screen.getByRole("button", { name: /add repository/i }));

    expect(handlers.addRepositoryRoot).toHaveBeenCalledOnce();
  });

  it("clicking Remove on a repository root calls handlers.removeRepositoryRoot with the path", async () => {
    const user = userEvent.setup();
    const handlers = makeHandlers();
    vi.mocked(useSettingsState).mockReturnValue(
      makeBaseState({
        repositoryRoots: ["/projects/my-app"],
        repoPathsDirty: false,
        handlers,
      })
    );

    renderWithProviders(<Settings />);

    const contextTab = screen.getByRole("button", { name: /context/i });
    await user.click(contextTab);

    await waitFor(() => {
      expect(screen.getByText("/projects/my-app")).toBeInTheDocument();
    });

    const removeBtn = screen.getByRole("button", {
      name: /remove repository \/projects\/my-app/i,
    });
    await user.click(removeBtn);

    expect(handlers.removeRepositoryRoot).toHaveBeenCalledWith("/projects/my-app");
  });

  it("Save Changes button is visible when hasChanges is true and calls saveSettings on click", async () => {
    const user = userEvent.setup();
    const handlers = makeHandlers();
    vi.mocked(useSettingsState).mockReturnValue(makeBaseState({ hasChanges: true, handlers }));

    renderWithProviders(<Settings />);

    await waitFor(() => {
      expect(screen.getByRole("button", { name: /save changes/i })).toBeInTheDocument();
    });

    await user.click(screen.getByRole("button", { name: /save changes/i }));

    expect(handlers.saveSettings).toHaveBeenCalledOnce();
  });

  it("toggling an adapter switch on Capabilities tab calls handlers.toggleAdapter with adapter id", async () => {
    const user = userEvent.setup();
    const handlers = makeHandlers();
    vi.mocked(useSettingsState).mockReturnValue(makeBaseState({ handlers }));

    renderWithProviders(<Settings />);

    const capabilitiesTab = screen.getByRole("button", { name: /capabilities/i });
    await user.click(capabilitiesTab);

    // Wait for the adapter list to render (from AdapterSettingsCard)
    await waitFor(() => {
      expect(screen.getByText("Claude Code")).toBeInTheDocument();
    });

    // The AdapterSettingsCard renders each adapter in a row that contains both
    // the adapter name and its switch. Scope the search to that row using within()
    // to avoid picking up the MCP switches that appear earlier in the tab.
    const claudeCodeEl = screen.getByText("Claude Code");
    // Navigate up to the row div: font-medium → unnamed div → flex gap-3 → flex justify-between (row)
    const adapterRow = claudeCodeEl.closest('[class*="rounded-md border"]');
    expect(adapterRow).not.toBeNull();
    const switchBtn = within(adapterRow as HTMLElement).getByRole("switch");
    await user.click(switchBtn);

    expect(handlers.toggleAdapter).toHaveBeenCalledWith("claude_code");
  });
});
