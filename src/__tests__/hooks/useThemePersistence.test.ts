import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";

const mockSettingsGet = vi.fn();
const mockSettingsSet = vi.fn().mockResolvedValue(undefined);

vi.mock("@/lib/tauri", () => ({
  api: {
    settings: {
      get: mockSettingsGet,
      set: mockSettingsSet,
    },
  },
}));

// featureManager mock — toggled per-test via moduleState
let featureEnabled = true;

vi.mock("@/lib/featureManager", () => ({
  featureManager: {
    isEnabled: () => featureEnabled,
  },
  FEATURE_FLAGS: {
    DIALOG_ACCESSIBILITY: "dialog_accessibility",
    THEME_PERSISTENCE: "theme_persistence",
    ENHANCED_ERROR_UX: "enhanced_error_ux",
    NATIVE_SKILL_SYNC: "native_skill_sync",
    UNIFIED_ARTIFACT_STATUS: "unified_artifact_status",
    EXECUTION_REDACTION: "execution_redaction",
  },
}));

// Import AFTER mocks are established
const { useThemePersistence } = await import("@/hooks/useThemePersistence");

describe("useThemePersistence — flag enabled", () => {
  beforeEach(() => {
    featureEnabled = true;
    mockSettingsGet.mockReset();
    mockSettingsSet.mockReset();
    mockSettingsSet.mockResolvedValue(undefined);
  });

  it("loads a saved valid theme from settings on mount", async () => {
    mockSettingsGet.mockResolvedValue("dark");

    const { result } = renderHook(() => useThemePersistence());

    await waitFor(() => expect(result.current.isLoaded).toBe(true));
    expect(result.current.theme).toBe("dark");
  });

  it("defaults to 'system' when no value is stored", async () => {
    mockSettingsGet.mockResolvedValue(null);

    const { result } = renderHook(() => useThemePersistence());

    await waitFor(() => expect(result.current.isLoaded).toBe(true));
    expect(result.current.theme).toBe("system");
  });

  it("falls back to 'system' for an invalid stored value", async () => {
    mockSettingsGet.mockResolvedValue("rainbow");

    const { result } = renderHook(() => useThemePersistence());

    await waitFor(() => expect(result.current.isLoaded).toBe(true));
    expect(result.current.theme).toBe("system");
  });

  it("persists theme change via settings.set", async () => {
    mockSettingsGet.mockResolvedValue("system");

    const { result } = renderHook(() => useThemePersistence());
    await waitFor(() => expect(result.current.isLoaded).toBe(true));

    act(() => {
      result.current.setTheme("light");
    });

    expect(result.current.theme).toBe("light");
    await waitFor(() => expect(mockSettingsSet).toHaveBeenCalledWith("ui_theme", "light"));
  });

  it("cycles light → dark → system via cycleTheme", async () => {
    mockSettingsGet.mockResolvedValue("light");

    const { result } = renderHook(() => useThemePersistence());
    await waitFor(() => expect(result.current.isLoaded).toBe(true));
    expect(result.current.theme).toBe("light");

    act(() => {
      result.current.cycleTheme();
    });
    expect(result.current.theme).toBe("dark");

    act(() => {
      result.current.cycleTheme();
    });
    expect(result.current.theme).toBe("system");

    act(() => {
      result.current.cycleTheme();
    });
    expect(result.current.theme).toBe("light");
  });
});

describe("useThemePersistence — flag disabled", () => {
  beforeEach(() => {
    featureEnabled = false;
    mockSettingsGet.mockReset();
    mockSettingsSet.mockReset();
  });

  it("always initializes to 'system' without calling settings.get", async () => {
    const { result } = renderHook(() => useThemePersistence());

    await waitFor(() => expect(result.current.isLoaded).toBe(true));
    expect(result.current.theme).toBe("system");
    expect(mockSettingsGet).not.toHaveBeenCalled();
  });

  it("does not call settings.set when theme changes", async () => {
    const { result } = renderHook(() => useThemePersistence());
    await waitFor(() => expect(result.current.isLoaded).toBe(true));

    act(() => {
      result.current.setTheme("dark");
    });

    expect(result.current.theme).toBe("dark");
    expect(mockSettingsSet).not.toHaveBeenCalled();
  });
});
