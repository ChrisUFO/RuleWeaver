import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Commands } from "../../components/pages/Commands";
import { api } from "../../lib/tauri";
import { renderWithProviders } from "./test-utils";
import type { CommandModel } from "../../types/command";

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn(),
  save: vi.fn(),
}));

vi.mock("../../lib/tauri", () => ({
  api: {
    commands: {
      getAll: vi.fn(),
      create: vi.fn(),
      update: vi.fn(),
      delete: vi.fn(),
      test: vi.fn(),
      sync: vi.fn(),
      getTemplates: vi.fn(),
    },
    slashCommands: {
      getAdapters: vi.fn(),
      getStatus: vi.fn(),
      sync: vi.fn(),
      syncAll: vi.fn(),
    },
    settings: {
      get: vi.fn(),
    },
    execution: {
      getHistoryFiltered: vi.fn(),
    },
    ruleImport: {
      getHistory: vi.fn(),
      scanAiToolCandidates: vi.fn(),
      importAiToolCommands: vi.fn(),
    },
    rules: {
      getAll: vi.fn(),
    },
    skills: {
      getAll: vi.fn(),
    },
  },
}));

const mockCommand: CommandModel = {
  id: "cmd-1",
  name: "Deploy Script",
  description: "Deploys the application",
  script: "npm run deploy",
  arguments: [],
  exposeViaMcp: true,
  isPlaceholder: false,
  createdAt: Date.now(),
  updatedAt: Date.now(),
};

describe("Commands lifecycle", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(api.commands.getAll).mockResolvedValue([mockCommand]);
    vi.mocked(api.commands.getTemplates).mockResolvedValue([]);
    vi.mocked(api.slashCommands.getAdapters).mockResolvedValue([]);
    vi.mocked(api.slashCommands.getStatus).mockResolvedValue({});
    vi.mocked(api.execution.getHistoryFiltered).mockResolvedValue([]);
    vi.mocked(api.settings.get).mockResolvedValue(null);
    vi.mocked(api.ruleImport.getHistory).mockResolvedValue([]);
  });

  it("clicking New button calls api.commands.create with default payload", async () => {
    const user = userEvent.setup();
    const newCmd: CommandModel = { ...mockCommand, id: "cmd-new", name: "New Command" };
    vi.mocked(api.commands.create).mockResolvedValue(newCmd);
    vi.mocked(api.commands.getAll)
      .mockResolvedValueOnce([mockCommand])
      .mockResolvedValueOnce([mockCommand, newCmd]);

    renderWithProviders(<Commands />);

    await waitFor(() => {
      expect(screen.getByText("Deploy Script")).toBeInTheDocument();
    });

    const newBtn = screen.getByRole("button", { name: /^new$/i });
    await user.click(newBtn);

    await waitFor(() => {
      expect(api.commands.create).toHaveBeenCalledWith(
        expect.objectContaining({ name: "New Command" })
      );
    });
  });

  it("editing and saving a selected command calls api.commands.update", async () => {
    const user = userEvent.setup();
    vi.mocked(api.commands.update).mockResolvedValue({ ...mockCommand, name: "Updated Script" });

    renderWithProviders(<Commands />);

    await waitFor(() => {
      expect(screen.getByText("Deploy Script")).toBeInTheDocument();
    });

    // Select the command from the list
    await user.click(screen.getByText("Deploy Script"));

    // Wait for the editor form to populate with the command data
    await waitFor(() => {
      expect(screen.getByDisplayValue("Deploy Script")).toBeInTheDocument();
    });

    // Edit the name
    const nameInput = screen.getByDisplayValue("Deploy Script");
    await user.clear(nameInput);
    await user.type(nameInput, "Updated Script");

    // Save
    const saveBtn = screen.getByRole("button", { name: /^save$/i });
    await user.click(saveBtn);

    await waitFor(() => {
      expect(api.commands.update).toHaveBeenCalledWith(
        "cmd-1",
        expect.objectContaining({ name: "Updated Script" })
      );
    });
  });

  it("deleting a selected command calls api.commands.delete with its id", async () => {
    const user = userEvent.setup();
    vi.mocked(api.commands.delete).mockResolvedValue(undefined);

    renderWithProviders(<Commands />);

    await waitFor(() => {
      expect(screen.getByText("Deploy Script")).toBeInTheDocument();
    });

    await user.click(screen.getByText("Deploy Script"));

    await waitFor(() => {
      expect(screen.getByRole("button", { name: /delete/i })).toBeInTheDocument();
    });

    await user.click(screen.getByRole("button", { name: /delete/i }));

    await waitFor(() => {
      expect(api.commands.delete).toHaveBeenCalledWith("cmd-1");
    });
  });

  it("Test Run button calls api.commands.test and shows stdout in output panel", async () => {
    const user = userEvent.setup();
    vi.mocked(api.commands.test).mockResolvedValue({
      success: true,
      stdout: "deployment successful",
      stderr: "",
      exitCode: 0,
      durationMs: 120,
    });

    renderWithProviders(<Commands />);

    await waitFor(() => {
      expect(screen.getByText("Deploy Script")).toBeInTheDocument();
    });

    await user.click(screen.getByText("Deploy Script"));

    await waitFor(() => {
      expect(screen.getByRole("button", { name: /test run/i })).toBeInTheDocument();
    });

    await user.click(screen.getByRole("button", { name: /test run/i }));

    await waitFor(() => {
      expect(api.commands.test).toHaveBeenCalledWith("cmd-1", {});
    });

    // Test output panel should appear
    await waitFor(() => {
      expect(screen.getByText(/deployment successful/i)).toBeInTheDocument();
    });
  });

  it("changing history filter calls getHistoryFiltered with the selected failure class", async () => {
    const user = userEvent.setup();

    renderWithProviders(<Commands />);

    await waitFor(() => {
      expect(screen.getByText("Deploy Script")).toBeInTheDocument();
    });

    await user.click(screen.getByText("Deploy Script"));

    await waitFor(() => {
      expect(screen.getByLabelText("Filter execution history by result")).toBeInTheDocument();
    });

    await userEvent.selectOptions(
      screen.getByLabelText("Filter execution history by result"),
      "NonZeroExit"
    );

    await waitFor(() => {
      expect(api.execution.getHistoryFiltered).toHaveBeenCalledWith(
        "cmd-1",
        "NonZeroExit",
        expect.any(Number),
        expect.any(Number)
      );
    });
  });
});
