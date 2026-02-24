import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useCommandsState } from "@/hooks/useCommandsState";

vi.mock("@/lib/tauri", () => ({
  api: {
    commands: {
      getAll: vi.fn().mockResolvedValue([
        {
          id: "1",
          name: "Test Command",
          description: "A test",
          script: "echo test",
          arguments: [],
          expose_via_mcp: true,
        },
      ]),
      create: vi.fn().mockResolvedValue({
        id: "2",
        name: "New Command",
        description: "",
        script: "",
        arguments: [],
      }),
      update: vi.fn().mockResolvedValue({
        id: "1",
        name: "Updated",
        description: "",
        script: "",
        arguments: [],
      }),
      delete: vi.fn().mockResolvedValue(undefined),
      test: vi.fn().mockResolvedValue({ stdout: "ok", stderr: "", exitCode: 0, success: true }),
      sync: vi.fn().mockResolvedValue({ success: true, filesWritten: [{ path: "/test" }] }),
    },
    execution: {
      getHistory: vi.fn().mockResolvedValue([]),
    },
    slashCommands: {
      getAdapters: vi
        .fn()
        .mockResolvedValue([{ name: "gemini", supportsArgumentSubstitution: true }]),
      sync: vi.fn().mockResolvedValue({ errors: [], conflicts: [], filesWritten: 1 }),
    },
  },
}));

const mockAddToast = vi.fn();

describe("useCommandsState", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("initializes with empty state", () => {
    const { result } = renderHook(() => useCommandsState(mockAddToast));

    expect(result.current.commands).toEqual([]);
    expect(result.current.selectedId).toBe("");
    expect(result.current.isLoading).toBe(true);
  });

  it("loads commands on mount", async () => {
    const { result } = renderHook(() => useCommandsState(mockAddToast));

    await act(async () => {
      await new Promise((resolve) => setTimeout(resolve, 100));
    });

    expect(result.current.commands).toHaveLength(1);
    expect(result.current.commands[0].name).toBe("Test Command");
    expect(result.current.isLoading).toBe(false);
  });

  it("updates form when command selected", async () => {
    const { result } = renderHook(() => useCommandsState(mockAddToast));

    await act(async () => {
      await new Promise((resolve) => setTimeout(resolve, 100));
    });

    act(() => {
      result.current.handlers.setSelectedId("1");
    });

    expect(result.current.selectedId).toBe("1");
    expect(result.current.form.name).toBe("Test Command");
  });

  it("filters commands by query", async () => {
    const { result } = renderHook(() => useCommandsState(mockAddToast));

    await act(async () => {
      await new Promise((resolve) => setTimeout(resolve, 100));
    });

    act(() => {
      result.current.handlers.setQuery("test");
    });

    expect(result.current.filtered).toHaveLength(1);

    act(() => {
      result.current.handlers.setQuery("nonexistent");
    });

    expect(result.current.filtered).toHaveLength(0);
  });

  it("updates form fields", async () => {
    const { result } = renderHook(() => useCommandsState(mockAddToast));

    act(() => {
      result.current.handlers.updateForm({ name: "New Name" });
    });

    expect(result.current.form.name).toBe("New Name");
  });

  it("toggles target paths", async () => {
    const { result } = renderHook(() => useCommandsState(mockAddToast));

    act(() => {
      result.current.handlers.toggleTargetPath("/path/to/repo", true);
    });

    expect(result.current.form.targetPaths).toContain("/path/to/repo");

    act(() => {
      result.current.handlers.toggleTargetPath("/path/to/repo", false);
    });

    expect(result.current.form.targetPaths).not.toContain("/path/to/repo");
  });

  it("toggles slash command adapters", async () => {
    const { result } = renderHook(() => useCommandsState(mockAddToast));

    act(() => {
      result.current.handlers.toggleSlashCommandAdapter("gemini");
    });

    expect(result.current.form.slashCommandAdapters).toContain("gemini");

    act(() => {
      result.current.handlers.toggleSlashCommandAdapter("gemini");
    });

    expect(result.current.form.slashCommandAdapters).not.toContain("gemini");
  });
});
