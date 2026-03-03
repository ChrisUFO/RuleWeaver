import { beforeEach, describe, expect, it, vi } from "vitest";
import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
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
    vi.mocked(api.sync.previewSync).mockResolvedValue({
      success: true,
      filesWritten: [],
      errors: [],
      conflicts: [],
    });
  });

  it("disables View Full Logs when no history exists", async () => {
    vi.mocked(api.sync.getHistory).mockResolvedValue([]);

    renderWithProviders(<Dashboard onNavigate={vi.fn()} />);

    const button = await screen.findByRole("button", { name: /view full logs/i });
    expect(button).toBeDisabled();
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

    const button = await screen.findByRole("button", { name: /view full logs/i });
    expect(button).toBeEnabled();

    await userEvent.click(button);

    await waitFor(() => {
      expect(api.sync.getHistory).toHaveBeenCalledWith(100);
    });

    expect(await screen.findByText("Sync Logs")).toBeInTheDocument();
  });
});
