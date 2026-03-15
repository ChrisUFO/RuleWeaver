import { describe, it, expect, vi, beforeEach } from "vitest";
import { act, renderHook, waitFor } from "@testing-library/react";

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
const mockSettingsListScopedSecrets = vi.fn().mockResolvedValue([]);
const mockSettingsUpsertScopedSecret = vi.fn().mockResolvedValue(undefined);
const mockSettingsDeleteScopedSecret = vi.fn().mockResolvedValue(undefined);
const mockSettingsGetSecretStorageStatus = vi.fn().mockResolvedValue({
  backend: "os-credential-manager",
  storesSecretsInOsCredentialManager: true,
  exportsIncludeSecrets: false,
  importsIncludeSecrets: false,
});
const mockStorageGetMode = vi.fn().mockResolvedValue("sqlite");
const mockStorageGetInfo = vi.fn().mockResolvedValue({});
const mockStorageGetMigrationProgress = vi.fn().mockResolvedValue(null);
const mockAppGetAppDataPath = vi.fn().mockResolvedValue("/data");
const mockAppGetVersion = vi.fn().mockResolvedValue("1.0.0");
const mockRegistryGetTools = vi.fn().mockResolvedValue([]);

vi.mock("@/lib/tauri", () => ({
  api: {
    settings: {
      get: mockSettingsGet,
      set: mockSettingsSet,
      listScopedSecrets: mockSettingsListScopedSecrets,
      upsertScopedSecret: mockSettingsUpsertScopedSecret,
      deleteScopedSecret: mockSettingsDeleteScopedSecret,
      getSecretStorageStatus: mockSettingsGetSecretStorageStatus,
    },
    storage: {
      getMode: mockStorageGetMode,
      getInfo: mockStorageGetInfo,
      getMigrationProgress: mockStorageGetMigrationProgress,
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
const mockRepoSetRoots = vi.fn();
const mockRepoRefresh = vi.fn().mockResolvedValue(undefined);

vi.mock("@/hooks/useRepositoryRoots", () => ({
  useRepositoryRoots: () => ({
    roots: [],
    setRoots: mockRepoSetRoots,
    refresh: mockRepoRefresh,
    save: mockRepoSave,
  }),
}));

const { useSettingsState } = await import("@/hooks/useSettingsState");

describe("useSettingsState — error UX (flag enabled)", () => {
  beforeEach(() => {
    enhancedErrorUxEnabled = true;
    vi.clearAllMocks();
    mockSettingsGet.mockResolvedValue(null);
    mockStorageGetMode.mockResolvedValue("sqlite");
    mockStorageGetInfo.mockResolvedValue({});
    mockStorageGetMigrationProgress.mockResolvedValue(null);
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

    await act(async () => {
      await result.current.handlers.saveSettings();
    });

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
    mockRepoSave.mockResolvedValue(undefined);
    mockAppGetAppDataPath.mockResolvedValue("/data");
    mockAppGetVersion.mockResolvedValue("1.0.0");
    mockRegistryGetTools.mockResolvedValue([]);
  });

  it("does not show a toast when settings fail to load and flag is disabled", async () => {
    const loadError = new Error("DB connection failed");
    mockAppGetAppDataPath.mockRejectedValue(loadError);
    const mockAddToast = vi.fn();

    const { result } = renderHook(() => useSettingsState(mockAddToast));
    await waitFor(() => expect(result.current.isLoading).toBe(false));

    const errorCalls = mockAddToast.mock.calls.filter((c) => c[0]?.variant === "error");
    expect(errorCalls).toHaveLength(0);
  });

  it("does not include retry action on save failure when flag is disabled", async () => {
    mockSettingsSet.mockRejectedValue(new Error("Write error"));
    const mockAddToast = vi.fn();

    const { result } = renderHook(() => useSettingsState(mockAddToast));
    await waitFor(() => expect(result.current.isLoading).toBe(false));

    await act(async () => {
      await result.current.handlers.saveSettings();
    });

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

    await act(async () => {
      await result.current.handlers.saveRepositoryRoots();
    });

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

    await act(async () => {
      await result.current.handlers.saveRepositoryRoots();
    });

    await waitFor(() => {
      const errorCall = mockAddToast.mock.calls.find((c) => c[0]?.variant === "error");
      expect(errorCall).toBeDefined();
      expect(errorCall![0].action).toBeUndefined();
    });
  });
});
