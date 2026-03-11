import { beforeEach, describe, expect, it, vi } from "vitest";
import { act, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { listen } from "@tauri-apps/api/event";
import { Dashboard } from "../../components/pages/Dashboard";
import { api } from "../../lib/tauri";
import { renderWithProviders } from "./test-utils";

vi.mock("../../stores/rulesStore", () => ({
  useRulesStore: () => ({
    rules: [
      {
        id: "rule-1",
        name: "Rule 1",
        description: "desc",
        content: "content",
        scope: "global",
        targetPaths: null,
        enabledAdapters: ["gemini"],
        enabled: true,
        createdAt: Date.now(),
        updatedAt: Date.now(),
      },
    ],
    fetchRules: vi.fn(),
    isLoading: false,
  }),
}));

vi.mock("../../lib/tauri", () => ({
  api: {
    rules: {
      getAll: vi.fn(),
    },
    commands: {
      getAll: vi.fn(),
    },
    skills: {
      getAll: vi.fn(),
    },
    status: {
      getArtifactStatus: vi.fn(),
    },
    sync: {
      getHistory: vi.fn(),
      previewSync: vi.fn(),
      syncRules: vi.fn(),
    },
  },
}));

describe("Dashboard lifecycle", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(listen).mockResolvedValue(() => {});
    vi.mocked(api.rules.getAll).mockResolvedValue([]);
    vi.mocked(api.commands.getAll).mockResolvedValue([]);
    vi.mocked(api.skills.getAll).mockResolvedValue([]);
    vi.mocked(api.status.getArtifactStatus).mockResolvedValue([]);
    vi.mocked(api.sync.previewSync).mockResolvedValue({
      success: true,
      filesWritten: [],
      errors: [],
      conflicts: [],
    });
  });

  it("shows multi-artifact dashboard counts", async () => {
    vi.mocked(api.rules.getAll).mockResolvedValue([
      {
        id: "rule-1",
        name: "Rule 1",
        description: "",
        content: "",
        scope: "global",
        targetPaths: null,
        enabledAdapters: ["gemini"],
        enabled: true,
        createdAt: Date.now(),
        updatedAt: Date.now(),
      },
    ]);
    vi.mocked(api.commands.getAll).mockResolvedValue([
      {
        id: "cmd-1",
        name: "Command 1",
        description: "",
        script: "echo test",
        arguments: [],
        exposeViaMcp: true,
        isPlaceholder: false,
        createdAt: Date.now(),
        updatedAt: Date.now(),
      },
    ]);
    vi.mocked(api.skills.getAll).mockResolvedValue([
      {
        id: "skill-1",
        name: "Skill 1",
        description: "",
        instructions: "",
        scope: "global",
        inputSchema: [],
        directoryPath: "",
        entryPoint: "",
        enabled: true,
        targetAdapters: [],
        targetPaths: [],
        createdAt: Date.now(),
        updatedAt: Date.now(),
      },
    ]);
    vi.mocked(api.status.getArtifactStatus).mockResolvedValue([]);
    vi.mocked(api.sync.getHistory).mockResolvedValue([]);

    renderWithProviders(<Dashboard onNavigate={vi.fn()} />);

    expect(
      await screen.findByText(/system operational\. 3 artifacts monitored\./i)
    ).toBeInTheDocument();
    expect(screen.getByText("Rules")).toBeInTheDocument();
    expect(screen.getByText("Commands")).toBeInTheDocument();
    expect(screen.getByText("Skills")).toBeInTheDocument();
  });

  it("maps sync progress events into the progress dialog", async () => {
    let progressHandler: ((event: { payload: unknown }) => void) | undefined;

    vi.mocked(listen).mockImplementation(async (eventName, handler) => {
      if (eventName === "sync-progress") {
        progressHandler = handler as (event: { payload: unknown }) => void;
      }
      return () => {};
    });
    vi.mocked(api.sync.getHistory).mockResolvedValue([]);

    renderWithProviders(<Dashboard onNavigate={vi.fn()} />);

    await waitFor(() => {
      expect(progressHandler).toBeDefined();
    });

    act(() => {
      progressHandler?.({
        payload: { phase: "start", currentFileIndex: 0, totalFiles: 2 },
      });
    });

    act(() => {
      progressHandler?.({
        payload: {
          phase: "progress",
          currentFile: "C:\\Users\\chris\\.config\\opencode\\rules\\my-rule.md",
          currentFileIndex: 1,
          totalFiles: 2,
          itemSuccess: true,
        },
      });
    });

    expect(await screen.findByText(/syncing artifacts/i)).toBeInTheDocument();
    expect(screen.getByText(/1 of 2 files/i)).toBeInTheDocument();
    expect(screen.getAllByText("my-rule.md").length).toBeGreaterThan(0);
  });

  it("disables View Full Logs when no history exists", async () => {
    vi.mocked(api.sync.getHistory).mockResolvedValue([]);

    renderWithProviders(<Dashboard onNavigate={vi.fn()} />);

    const button = await screen.findByRole("button", { name: /sync logs/i });
    expect(button).toBeDisabled();
    expect(screen.getByText(/run a sync to generate logs/i)).toBeInTheDocument();
  });

  it("opens full logs dialog from dashboard history", async () => {
    const history = [
      {
        id: "sync-1",
        timestamp: Math.floor(Date.now() / 1000),
        filesWritten: 2,
        status: "success" as const,
        triggeredBy: "manual" as const,
      },
    ];

    vi.mocked(api.sync.getHistory).mockImplementation(async (limit?: number) => {
      if (limit === 100) {
        return history;
      }
      return history;
    });

    renderWithProviders(<Dashboard onNavigate={vi.fn()} />);

    const button = await screen.findByRole("button", { name: /sync logs/i });
    expect(button).toBeEnabled();

    await userEvent.click(button);

    await waitFor(() => {
      expect(api.sync.getHistory).toHaveBeenCalledWith(100);
    });

    expect(await screen.findByRole("heading", { name: "Sync Logs" })).toBeInTheDocument();
  });

  it("shows retry UI when full log loading fails", async () => {
    vi.mocked(api.sync.getHistory).mockImplementation(async (limit?: number) => {
      if (limit === 100) {
        throw new Error("history unavailable");
      }
      return [
        {
          id: "sync-1",
          timestamp: Math.floor(Date.now() / 1000),
          filesWritten: 1,
          status: "success" as const,
          triggeredBy: "manual" as const,
        },
      ];
    });

    renderWithProviders(<Dashboard onNavigate={vi.fn()} />);

    const button = await screen.findByRole("button", { name: /sync logs/i });
    await userEvent.click(button);

    expect(await screen.findByText(/unable to load sync logs/i)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /retry/i })).toBeInTheDocument();
  });
});
