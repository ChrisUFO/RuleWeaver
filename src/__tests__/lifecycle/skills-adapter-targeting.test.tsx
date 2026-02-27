import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Skills } from "../../components/pages/Skills";
import { api } from "../../lib/tauri";
import { renderWithProviders } from "./test-utils";
import type { Skill } from "../../types/skill";

// Mock Tauri API
vi.mock("../../lib/tauri", () => ({
  api: {
    settings: {
      get: vi.fn(),
    },
    skills: {
      getAll: vi.fn(),
      create: vi.fn(),
      update: vi.fn(),
      delete: vi.fn(),
      getSupportedAdapters: vi.fn(),
    },
    app: {
      openInExplorer: vi.fn(),
    },
    status: {
      getArtifactStatus: vi.fn(),
    },
  },
}));

// Realistic list of skill-supporting adapters per the registry:
// supports_skills: true → antigravity, claude_code, cline, codex, gemini, opencode, roo, windsurf
// supports_skills: false → cursor, kilo
const SKILL_SUPPORTING_ADAPTERS = [
  "antigravity",
  "claude_code",
  "cline",
  "codex",
  "gemini",
  "opencode",
  "roo",
  "windsurf",
];

const mockSkill: Skill = {
  id: "skill-1",
  name: "Test Skill",
  description: "A test skill",
  instructions: "Do things",
  inputSchema: [],
  directoryPath: "/test/skill",
  entryPoint: "run.sh",
  scope: "global",
  enabled: true,
  targetAdapters: [],
  targetPaths: [],
  createdAt: Date.now(),
  updatedAt: Date.now(),
};

describe("Skills adapter targeting lifecycle", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(api.settings.get).mockResolvedValue("[]");
    vi.mocked(api.skills.getAll).mockResolvedValue([mockSkill]);
    vi.mocked(api.skills.getSupportedAdapters).mockResolvedValue(SKILL_SUPPORTING_ADAPTERS);
    vi.mocked(api.status.getArtifactStatus).mockResolvedValue([]);
  });

  it("adapter checkbox list renders only skill-supporting adapters (no Cursor, no Kilo)", async () => {
    const user = userEvent.setup();

    renderWithProviders(<Skills />);

    await waitFor(() => {
      expect(screen.getByText("Test Skill")).toBeInTheDocument();
    });

    // Select the skill to open its editor
    await user.click(screen.getByText("Test Skill"));

    await waitFor(() => {
      expect(screen.getByText("Adapter Distribution")).toBeInTheDocument();
    });

    // Windsurf should be present (supports_skills: true)
    expect(screen.getByText("Windsurf")).toBeInTheDocument();
    // Claude Code should be present
    expect(screen.getByText("Claude Code")).toBeInTheDocument();
    // Cursor should NOT be present (supports_skills: false)
    expect(screen.queryByText("Cursor")).not.toBeInTheDocument();
    // Kilo Code should NOT be present (supports_skills: false per registry)
    expect(screen.queryByText("Kilo Code")).not.toBeInTheDocument();
  });

  it("Windsurf checkbox is present in the adapter distribution list", async () => {
    const user = userEvent.setup();

    renderWithProviders(<Skills />);

    await waitFor(() => {
      expect(screen.getByText("Test Skill")).toBeInTheDocument();
    });

    await user.click(screen.getByText("Test Skill"));

    await waitFor(() => {
      expect(screen.getByText("Windsurf")).toBeInTheDocument();
    });

    // The Windsurf label should be associated with a checkbox
    const windsurfLabel = screen.getByText("Windsurf").closest("label");
    expect(windsurfLabel).toBeInTheDocument();
  });

  it("Cursor checkbox is absent because Cursor does not support skills", async () => {
    const user = userEvent.setup();

    renderWithProviders(<Skills />);

    await waitFor(() => {
      expect(screen.getByText("Test Skill")).toBeInTheDocument();
    });

    await user.click(screen.getByText("Test Skill"));

    await waitFor(() => {
      expect(screen.getByText("Adapter Distribution")).toBeInTheDocument();
    });

    expect(screen.queryByText("Cursor")).not.toBeInTheDocument();
  });

  it("checking an adapter checkbox includes it in the targetAdapters save payload", async () => {
    const user = userEvent.setup();
    vi.mocked(api.skills.update).mockResolvedValue({ ...mockSkill });

    renderWithProviders(<Skills />);

    await waitFor(() => {
      expect(screen.getByText("Test Skill")).toBeInTheDocument();
    });

    await user.click(screen.getByText("Test Skill"));

    await waitFor(() => {
      expect(screen.getByText("Windsurf")).toBeInTheDocument();
    });

    // Check the Windsurf adapter checkbox
    const windsurfLabel = screen.getByText("Windsurf").closest("label")!;
    await user.click(windsurfLabel);

    const saveBtn = screen.getByRole("button", { name: /save changes/i });
    await user.click(saveBtn);

    await waitFor(() => {
      expect(api.skills.update).toHaveBeenCalledWith(
        "skill-1",
        expect.objectContaining({
          targetAdapters: ["windsurf"],
        })
      );
    });
  });

  it("saving with no adapters checked sends empty targetAdapters (targets all supported adapters)", async () => {
    const user = userEvent.setup();
    // Skill already has targetAdapters: [] — no adapters checked
    vi.mocked(api.skills.update).mockResolvedValue({ ...mockSkill, targetAdapters: [] });

    renderWithProviders(<Skills />);

    await waitFor(() => {
      expect(screen.getByText("Test Skill")).toBeInTheDocument();
    });

    await user.click(screen.getByText("Test Skill"));

    await waitFor(() => {
      expect(screen.getByText("Adapter Distribution")).toBeInTheDocument();
    });

    // Don't check any adapters — save with empty selection
    const saveBtn = screen.getByRole("button", { name: /save changes/i });
    await user.click(saveBtn);

    await waitFor(() => {
      expect(api.skills.update).toHaveBeenCalledWith(
        "skill-1",
        expect.objectContaining({
          targetAdapters: [],
        })
      );
    });
  });
});
