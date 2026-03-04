import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";

// --- Feature flag mock (toggled per test) ---
let enhancedErrorUxEnabled = true;

vi.mock("@/lib/featureManager", () => ({
  featureManager: { isEnabled: () => enhancedErrorUxEnabled },
  FEATURE_FLAGS: {
    DIALOG_ACCESSIBILITY: "dialog_accessibility",
    THEME_PERSISTENCE: "theme_persistence",
    ENHANCED_ERROR_UX: "enhanced_error_ux",
    NATIVE_SKILL_SYNC: "native_skill_sync",
    UNIFIED_ARTIFACT_STATUS: "unified_artifact_status",
    EXECUTION_REDACTION: "execution_redaction",
  },
}));

// --- Tauri API mock ---
const mockSettingsGet = vi.fn();
const mockSettingsSet = vi.fn().mockResolvedValue(undefined);
const mockStorageGetMode = vi.fn().mockResolvedValue("sqlite");
const mockStorageGetInfo = vi.fn().mockResolvedValue({});
const mockStorageGetMigrationProgress = vi.fn().mockResolvedValue(null);
const mockMcpGetStatus = vi.fn().mockResolvedValue({ running: false, port: 0 });
const mockMcpGetLogs = vi.fn().mockResolvedValue([]);
const mockMcpStart = vi.fn().mockResolvedValue(undefined);
const mockMcpGetInstructions = vi.fn().mockResolvedValue("");
const mockAppGetAppDataPath = vi.fn().mockResolvedValue("/data");
const mockAppGetVersion = vi.fn().mockResolvedValue("1.0.0");
const mockRegistryGetTools = vi.fn().mockResolvedValue([]);

vi.mock("@/lib/tauri", () => ({
  api: {
    settings: { get: mockSettingsGet, set: mockSettingsSet },
    storage: {
      getMode: mockStorageGetMode,
      getInfo: mockStorageGetInfo,
      getMigrationProgress: mockStorageGetMigrationProgress,
    },
    mcp: {
      getStatus: mockMcpGetStatus,
      getLogs: mockMcpGetLogs,
      start: mockMcpStart,
      stop: vi.fn().mockResolvedValue(undefined),
      getInstructions: mockMcpGetInstructions,
    },
    app: { getAppDataPath: mockAppGetAppDataPath, getVersion: mockAppGetVersion },
    registry: { getTools: mockRegistryGetTools },
  },
}));

vi.mock("@tauri-apps/plugin-autostart", () => ({
  isEnabled: vi.fn().mockResolvedValue(false),
  enable: vi.fn().mockResolvedValue(undefined),
  disable: vi.fn().mockResolvedValue(undefined),
}));

const mockRepoSave = vi.fn().mockResolvedValue(undefined);

vi.mock("@/hooks/useRepositoryRoots", () => ({
  useRepositoryRoots: () => ({
    roots: [],
    setRoots: vi.fn(),
    refresh: vi.fn().mockResolvedValue(undefined),
    save: mockRepoSave,
  }),
}));

const { useSettingsState } = await import("@/hooks/useSettingsState");

describe("useSettingsState — error UX (flag enabled)", () => {
  beforeEach(() => {
    enhancedErrorUxEnabled = true;
    vi.clearAllMocks();
    // Default: settings.get returns null (no values stored)
    mockSettingsGet.mockResolvedValue(null);
    mockStorageGetMode.mockResolvedValue("sqlite");
    mockStorageGetInfo.mockResolvedValue({});
    mockStorageGetMigrationProgress.mockResolvedValue(null);
    mockMcpGetStatus.mockResolvedValue({ running: false, port: 0 });
    mockMcpGetLogs.mockResolvedValue([]);
    mockMcpStart.mockResolvedValue(undefined);
    mockMcpGetInstructions.mockResolvedValue("");
    mockRepoSave.mockResolvedValue(undefined);
    mockAppGetAppDataPath.mockResolvedValue("/data");
    mockAppGetVersion.mockResolvedValue("1.0.0");
    mockRegistryGetTools.mockResolvedValue([]);
  });

  it("shows a user-visible error toast when settings fail to load", async () => {
    const loadError = new Error("DB connection failed");
    mockAppGetAppDataPath.mockRejectedValue(loadError);

    const mockAddToast = vi.fn();
    renderHook(() => useSettingsState(mockAddToast));

    await waitFor(() => {
      expect(mockAddToast).toHaveBeenCalledWith(
        expect.objectContaining({
          variant: "error",
          title: "Failed to Load Settings",
        })
      );
    });
  });

  it("shows 'Open Settings' action on load failure when onNavigate is provided", async () => {
    const loadError = new Error("DB connection failed");
    mockAppGetAppDataPath.mockRejectedValue(loadError);
    const mockAddToast = vi.fn();
    const mockNavigate = vi.fn();

    renderHook(() => useSettingsState(mockAddToast, mockNavigate));

    await waitFor(() => {
      const call = mockAddToast.mock.calls.find((c) =>
        c[0]?.title?.includes("Failed to Load Settings")
      );
      expect(call).toBeDefined();
      expect(call![0].action).toBeDefined();
      expect(call![0].action.label).toBe("Open Settings");
    });
  });

  it("shows retry action on settings save failure", async () => {
    mockSettingsSet.mockRejectedValue(new Error("Write error"));
    const mockAddToast = vi.fn();

    const { result } = renderHook(() => useSettingsState(mockAddToast));
    await waitFor(() => expect(result.current.isLoading).toBe(false));

    await result.current.handlers.saveSettings();

    await waitFor(() => {
      const errorCall = mockAddToast.mock.calls.find((c) => c[0]?.variant === "error");
      expect(errorCall).toBeDefined();
      expect(errorCall![0].action).toBeDefined();
      expect(errorCall![0].action.label).toBe("Retry");
    });
  });
});

describe("useSettingsState — error UX (flag disabled)", () => {
  beforeEach(() => {
    enhancedErrorUxEnabled = false;
    vi.clearAllMocks();
    mockSettingsGet.mockResolvedValue(null);
    mockStorageGetMode.mockResolvedValue("sqlite");
    mockStorageGetInfo.mockResolvedValue({});
    mockStorageGetMigrationProgress.mockResolvedValue(null);
    mockMcpGetStatus.mockResolvedValue({ running: false, port: 0 });
    mockMcpGetLogs.mockResolvedValue([]);
    mockMcpStart.mockResolvedValue(undefined);
    mockMcpGetInstructions.mockResolvedValue("");
    mockRepoSave.mockResolvedValue(undefined);
    mockAppGetAppDataPath.mockResolvedValue("/data");
    mockAppGetVersion.mockResolvedValue("1.0.0");
    mockRegistryGetTools.mockResolvedValue([]);
  });

  it("does not show a toast when settings fail to load and flag is disabled", async () => {
    const loadError = new Error("DB connection failed");
    mockAppGetAppDataPath.mockRejectedValue(loadError);
    const mockAddToast = vi.fn();

    renderHook(() => useSettingsState(mockAddToast));

    // Give it time to settle — toast should NOT be called for load failure
    await new Promise((r) => setTimeout(r, 50));
    const errorCalls = mockAddToast.mock.calls.filter((c) => c[0]?.variant === "error");
    expect(errorCalls).toHaveLength(0);
  });

  it("does not include retry action on save failure when flag is disabled", async () => {
    mockSettingsSet.mockRejectedValue(new Error("Write error"));
    const mockAddToast = vi.fn();

    const { result } = renderHook(() => useSettingsState(mockAddToast));
    await waitFor(() => expect(result.current.isLoading).toBe(false));

    await result.current.handlers.saveSettings();

    await waitFor(() => {
      const errorCall = mockAddToast.mock.calls.find((c) => c[0]?.variant === "error");
      expect(errorCall).toBeDefined();
      // action should be undefined when flag is off
      expect(errorCall![0].action).toBeUndefined();
    });
  });
});

// ---------------------------------------------------------------------------
// Repository roots save failure — retry action
// ---------------------------------------------------------------------------
describe("useSettingsState — repo roots save failure (flag enabled)", () => {
  beforeEach(() => {
    enhancedErrorUxEnabled = true;
    vi.clearAllMocks();
    mockSettingsGet.mockResolvedValue(null);
    mockStorageGetMode.mockResolvedValue("sqlite");
    mockStorageGetInfo.mockResolvedValue({});
    mockStorageGetMigrationProgress.mockResolvedValue(null);
    mockMcpGetStatus.mockResolvedValue({ running: false, port: 0 });
    mockMcpGetLogs.mockResolvedValue([]);
    mockMcpStart.mockResolvedValue(undefined);
    mockMcpGetInstructions.mockResolvedValue("");
    mockRepoSave.mockResolvedValue(undefined);
    mockAppGetAppDataPath.mockResolvedValue("/data");
    mockAppGetVersion.mockResolvedValue("1.0.0");
    mockRegistryGetTools.mockResolvedValue([]);
  });

  it("shows Retry action on repository roots save failure", async () => {
    mockRepoSave.mockRejectedValue(new Error("Disk write error"));
    const mockAddToast = vi.fn();

    const { result } = renderHook(() => useSettingsState(mockAddToast));
    await waitFor(() => expect(result.current.isLoading).toBe(false));

    await result.current.handlers.saveRepositoryRoots();

    await waitFor(() => {
      const errorCall = mockAddToast.mock.calls.find((c) => c[0]?.variant === "error");
      expect(errorCall).toBeDefined();
      expect(errorCall![0].action).toBeDefined();
      expect(errorCall![0].action.label).toBe("Retry");
    });
  });

  it("does not show Retry action on repo roots save failure when flag is disabled", async () => {
    enhancedErrorUxEnabled = false;
    mockRepoSave.mockRejectedValue(new Error("Disk write error"));
    const mockAddToast = vi.fn();

    const { result } = renderHook(() => useSettingsState(mockAddToast));
    await waitFor(() => expect(result.current.isLoading).toBe(false));

    await result.current.handlers.saveRepositoryRoots();

    await waitFor(() => {
      const errorCall = mockAddToast.mock.calls.find((c) => c[0]?.variant === "error");
      expect(errorCall).toBeDefined();
      expect(errorCall![0].action).toBeUndefined();
    });
  });
});

// ---------------------------------------------------------------------------
// MCP start failure — "View Logs" action
// ---------------------------------------------------------------------------
describe("useSettingsState — MCP start failure (flag enabled)", () => {
  beforeEach(() => {
    enhancedErrorUxEnabled = true;
    vi.clearAllMocks();
    mockSettingsGet.mockResolvedValue(null);
    mockStorageGetMode.mockResolvedValue("sqlite");
    mockStorageGetInfo.mockResolvedValue({});
    mockStorageGetMigrationProgress.mockResolvedValue(null);
    mockMcpGetStatus.mockResolvedValue({ running: false, port: 0 });
    mockMcpGetLogs.mockResolvedValue([]);
    mockMcpStart.mockResolvedValue(undefined);
    mockMcpGetInstructions.mockResolvedValue("");
    mockRepoSave.mockResolvedValue(undefined);
    mockAppGetAppDataPath.mockResolvedValue("/data");
    mockAppGetVersion.mockResolvedValue("1.0.0");
    mockRegistryGetTools.mockResolvedValue([]);
  });

  it("shows 'View Logs' action when MCP start fails and onNavigate is provided", async () => {
    mockMcpStart.mockRejectedValue(new Error("Connection refused"));
    const mockAddToast = vi.fn();
    const mockNavigate = vi.fn();

    const { result } = renderHook(() => useSettingsState(mockAddToast, mockNavigate));
    await waitFor(() => expect(result.current.isLoading).toBe(false));

    await result.current.handlers.startMcp();

    await waitFor(() => {
      const errorCall = mockAddToast.mock.calls.find((c) => c[0]?.variant === "error");
      expect(errorCall).toBeDefined();
      expect(errorCall![0].title).toBe("MCP Start Failed");
      expect(errorCall![0].action).toBeDefined();
      expect(errorCall![0].action.label).toBe("View Logs");
    });
  });

  it("does not show 'View Logs' action when MCP start fails without onNavigate", async () => {
    mockMcpStart.mockRejectedValue(new Error("Connection refused"));
    const mockAddToast = vi.fn();

    const { result } = renderHook(() => useSettingsState(mockAddToast));
    await waitFor(() => expect(result.current.isLoading).toBe(false));

    await result.current.handlers.startMcp();

    await waitFor(() => {
      const errorCall = mockAddToast.mock.calls.find((c) => c[0]?.variant === "error");
      expect(errorCall).toBeDefined();
      expect(errorCall![0].action).toBeUndefined();
    });
  });

  it("does not show 'View Logs' action when flag is disabled", async () => {
    enhancedErrorUxEnabled = false;
    mockMcpStart.mockRejectedValue(new Error("Connection refused"));
    const mockAddToast = vi.fn();
    const mockNavigate = vi.fn();

    const { result } = renderHook(() => useSettingsState(mockAddToast, mockNavigate));
    await waitFor(() => expect(result.current.isLoading).toBe(false));

    await result.current.handlers.startMcp();

    await waitFor(() => {
      const errorCall = mockAddToast.mock.calls.find((c) => c[0]?.variant === "error");
      expect(errorCall).toBeDefined();
      expect(errorCall![0].action).toBeUndefined();
    });
  });
});
