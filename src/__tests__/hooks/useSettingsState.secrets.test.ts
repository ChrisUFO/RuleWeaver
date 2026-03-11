import { beforeEach, describe, expect, it, vi } from "vitest";
import { act, renderHook, waitFor } from "@testing-library/react";

const mockSettingsGet = vi.fn();
const mockSettingsSet = vi.fn().mockResolvedValue(undefined);
const mockListScopedSecrets = vi.fn().mockResolvedValue([]);
const mockUpsertScopedSecret = vi.fn().mockResolvedValue(undefined);
const mockDeleteScopedSecret = vi.fn().mockResolvedValue(undefined);
const mockRepoSetRoots = vi.fn();
const mockRepoRefresh = vi.fn().mockResolvedValue(undefined);

const makeMcpStatus = (overrides: Record<string, unknown> = {}) => ({
  running: false,
  port: 4545,
  uptimeSeconds: 0,
  apiToken: "test-token",
  isWatching: false,
  endpointUrl: "http://127.0.0.1:4545",
  healthState: "stopped",
  statusMessage: "MCP server is stopped",
  diagnostics: [],
  availableCommands: 0,
  availableSkills: 0,
  watchTargetCount: 0,
  ...overrides,
});

const makeMcpInstructions = () => ({
  claudeCodeJson: "{}",
  opencodeJson: "{}",
  standaloneCommand: "ruleweaver-mcp --port 4545 --token test-token",
  apiToken: "test-token",
  endpointUrl: "http://127.0.0.1:4545",
  authHeaderName: "X-API-Key",
});

vi.mock("@/lib/featureManager", () => ({
  featureManager: { isEnabled: () => true },
  FEATURE_FLAGS: { ENHANCED_ERROR_UX: "enhanced_error_ux" },
}));

vi.mock("@/lib/tauri", () => ({
  api: {
    settings: {
      get: mockSettingsGet,
      set: mockSettingsSet,
      listScopedSecrets: mockListScopedSecrets,
      upsertScopedSecret: mockUpsertScopedSecret,
      deleteScopedSecret: mockDeleteScopedSecret,
    },
    storage: {
      getMode: vi.fn().mockResolvedValue("sqlite"),
      getInfo: vi.fn().mockResolvedValue({}),
      getMigrationProgress: vi.fn().mockResolvedValue(null),
    },
    mcp: {
      getStatus: vi.fn().mockResolvedValue(makeMcpStatus()),
      getLogs: vi.fn().mockResolvedValue([]),
      getInstructions: vi.fn().mockResolvedValue(makeMcpInstructions()),
    },
    app: {
      getAppDataPath: vi.fn().mockResolvedValue("/data"),
      getVersion: vi.fn().mockResolvedValue("1.0.0"),
      openInExplorer: vi.fn(),
    },
    registry: { getTools: vi.fn().mockResolvedValue([]) },
    slashCommands: { syncAll: vi.fn().mockResolvedValue({ filesWritten: 0, errors: [] }) },
    rules: { getAll: vi.fn().mockResolvedValue([]) },
    commands: { getAll: vi.fn().mockResolvedValue([]) },
    skills: { getAll: vi.fn().mockResolvedValue([]) },
  },
}));

vi.mock("@/hooks/useRepositoryRoots", () => ({
  useRepositoryRoots: () => ({
    roots: ["/repos/app"],
    setRoots: mockRepoSetRoots,
    refresh: mockRepoRefresh,
    save: vi.fn().mockResolvedValue(undefined),
  }),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn(), save: vi.fn() }));
vi.mock("@tauri-apps/plugin-updater", () => ({ check: vi.fn() }));
vi.mock("@tauri-apps/plugin-autostart", () => ({
  enable: vi.fn().mockResolvedValue(undefined),
  disable: vi.fn().mockResolvedValue(undefined),
  isEnabled: vi.fn().mockResolvedValue(false),
}));

const { useSettingsState } = await import("@/hooks/useSettingsState");

describe("useSettingsState scoped secrets", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockSettingsGet.mockResolvedValue(null);
    mockListScopedSecrets.mockResolvedValue([
      {
        id: "secret-1",
        key: "PROJECT_API_KEY",
        value: "global",
        scope: "global",
        createdAt: 1,
        updatedAt: 1,
      },
    ]);
  });

  it("loads scoped secrets during settings initialization", async () => {
    const onNavigate = vi.fn();
    const { result } = renderHook(() => useSettingsState(onNavigate));

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    expect(result.current.scopedSecrets).toHaveLength(1);
    expect(result.current.scopedSecrets[0]?.key).toBe("PROJECT_API_KEY");
  });

  it("saves a workspace secret override and refreshes the secret list", async () => {
    const onNavigate = vi.fn();
    const { result } = renderHook(() => useSettingsState(onNavigate));

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    await act(async () => {
      await result.current.handlers.saveWorkspaceSecret(
        "PROJECT_API_KEY",
        "repo-token",
        "/repos/app"
      );
    });

    expect(mockUpsertScopedSecret).toHaveBeenCalledWith({
      key: "PROJECT_API_KEY",
      value: "repo-token",
      scope: "workspace",
      workspacePath: "/repos/app",
    });
    expect(mockListScopedSecrets).toHaveBeenCalledTimes(2);
  });
});
