import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Status } from "../../components/pages/Status";
import { api } from "../../lib/tauri";
import { ToastProvider } from "../../components/ui/toast";
import type { ArtifactStatusEntry, StatusSummary } from "../../types/status";

vi.mock("../../lib/tauri", () => ({
  api: {
    status: {
      getArtifactStatus: vi.fn(),
      getSummary: vi.fn(),
      repairArtifact: vi.fn(),
      repairAll: vi.fn(),
    },
  },
}));

vi.mock("../../stores/registryStore", () => ({
  useRegistryStore: () => ({
    tools: [
      {
        id: "claude_code",
        name: "Claude Code",
        description: "Claude Code adapter",
        icon: "claude",
        fileFormat: "md",
        capabilities: {},
        paths: { globalPath: "~/.claude/CLAUDE.md", localPathTemplate: ".claude/CLAUDE.md" },
      },
      {
        id: "gemini",
        name: "Gemini",
        description: "Gemini adapter",
        icon: "gemini",
        fileFormat: "toml",
        capabilities: {},
        paths: { globalPath: "~/.gemini/GEMINI.md", localPathTemplate: ".gemini/GEMINI.md" },
      },
    ],
    isLoading: false,
    error: null,
    fetchTools: vi.fn(),
  }),
}));

const mockMissingEntry: ArtifactStatusEntry = {
  id: "entry-1",
  artifactId: "rule-1",
  artifactName: "My Rule",
  artifactType: "rule",
  adapter: "claude-code",
  scope: "global",
  status: "missing",
  expectedPath: "/home/.claude/CLAUDE.md",
};

const mockOutOfDateEntry: ArtifactStatusEntry = {
  id: "entry-2",
  artifactId: "skill-1",
  artifactName: "My Skill",
  artifactType: "skill",
  adapter: "gemini",
  scope: "global",
  status: "out_of_date",
  expectedPath: "/home/.gemini/skills/my-skill/SKILL.md",
};

const mockSummary: StatusSummary = {
  total: 2,
  synced: 0,
  outOfDate: 1,
  missing: 1,
  conflicted: 0,
  unsupported: 0,
  error: 0,
};

const renderWithProviders = (ui: React.ReactElement) => render(<ToastProvider>{ui}</ToastProvider>);

describe("Status lifecycle", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(api.status.getArtifactStatus).mockResolvedValue([
      mockMissingEntry,
      mockOutOfDateEntry,
    ]);
    vi.mocked(api.status.getSummary).mockResolvedValue(mockSummary);
  });

  it("changing Artifact Type filter makes api call with artifactType in filter", async () => {
    const user = userEvent.setup();

    renderWithProviders(<Status />);

    await waitFor(() => {
      expect(screen.getByText("My Rule")).toBeInTheDocument();
    });

    // The Type filter is the first <select> (combobox) in the filters section
    const selects = screen.getAllByRole("combobox");
    await user.selectOptions(selects[0], "skill");

    await waitFor(() => {
      expect(api.status.getArtifactStatus).toHaveBeenCalledWith(
        expect.objectContaining({ artifactType: "skill" })
      );
    });
  });

  it("changing Tool filter makes api call with adapter in filter", async () => {
    const user = userEvent.setup();

    renderWithProviders(<Status />);

    await waitFor(() => {
      expect(screen.getByText("My Rule")).toBeInTheDocument();
    });

    // The Tool filter is the second <select>
    const selects = screen.getAllByRole("combobox");
    await user.selectOptions(selects[1], "claude_code");

    await waitFor(() => {
      expect(api.status.getArtifactStatus).toHaveBeenCalledWith(
        expect.objectContaining({ adapter: "claude_code" })
      );
    });
  });

  it("changing Status filter makes api call with status in filter", async () => {
    const user = userEvent.setup();

    renderWithProviders(<Status />);

    await waitFor(() => {
      expect(screen.getByText("My Rule")).toBeInTheDocument();
    });

    // The Status filter is the third <select>
    const selects = screen.getAllByRole("combobox");
    await user.selectOptions(selects[2], "missing");

    await waitFor(() => {
      expect(api.status.getArtifactStatus).toHaveBeenCalledWith(
        expect.objectContaining({ status: "missing" })
      );
    });
  });

  it("Repair button on an entry calls repairArtifact with that entry id", async () => {
    const user = userEvent.setup();
    vi.mocked(api.status.repairArtifact).mockResolvedValue({
      entryId: "entry-1",
      success: true,
    });

    renderWithProviders(<Status />);

    await waitFor(() => {
      expect(screen.getByText("My Rule")).toBeInTheDocument();
    });

    // Both entries have Repair buttons (status: missing and out_of_date).
    // entry-1 (My Rule / missing) is first in the array → first button in DOM.
    const repairBtns = screen.getAllByRole("button", { name: /^repair$/i });
    expect(repairBtns.length).toBeGreaterThanOrEqual(2);
    await user.click(repairBtns[0]);

    await waitFor(() => {
      expect(api.status.repairArtifact).toHaveBeenCalledWith("entry-1");
    });
  });

  it("Repair All button calls repairAll and refreshes entries", async () => {
    const user = userEvent.setup();
    vi.mocked(api.status.repairAll).mockResolvedValue([
      { entryId: "entry-1", success: true },
      { entryId: "entry-2", success: true },
    ]);

    renderWithProviders(<Status />);

    await waitFor(() => {
      expect(screen.getByText("My Rule")).toBeInTheDocument();
    });

    const bulkRepairBtn = screen.getByRole("button", { name: /repair all/i });
    await user.click(bulkRepairBtn);

    await waitFor(() => {
      expect(api.status.repairAll).toHaveBeenCalled();
    });

    // Status should be refreshed after bulk repair
    await waitFor(() => {
      expect(api.status.getArtifactStatus).toHaveBeenCalledTimes(2);
    });
  });
});
