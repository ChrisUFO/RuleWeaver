import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Skills } from "../../../components/pages/Skills";
import { api } from "../../../lib/tauri";
import { ToastProvider } from "../../../components/ui/toast";
import { Skill, SkillParameterType } from "../../../types/skill";

// Mock Tauri API
vi.mock("../../../lib/tauri", () => ({
  api: {
    settings: {
      get: vi.fn(),
    },
    skills: {
      getAll: vi.fn(),
      create: vi.fn(),
      update: vi.fn(),
      delete: vi.fn(),
    },
    app: {
      openInExplorer: vi.fn(),
    },
  },
}));

// Provide Toast Context for tests
const renderWithProviders = (ui: React.ReactElement) => {
  return render(<ToastProvider>{ui}</ToastProvider>);
};

const mockSkills: Skill[] = [
  {
    id: "skill-1",
    name: "Test Skill 1",
    description: "A test description",
    instructions: "Do a thing",
    inputSchema: [
      {
        name: "param1",
        description: "Test param",
        paramType: SkillParameterType.String,
        required: true,
      },
    ],
    directoryPath: "/test/path",
    entryPoint: "run.sh",
    scope: "global",
    enabled: true,
    createdAt: Date.now(),
    updatedAt: Date.now(),
  },
  {
    id: "skill-2",
    name: "Disabled Skill",
    description: "Another test",
    instructions: "Do another thing",
    inputSchema: [],
    directoryPath: "/test/path2",
    entryPoint: "main.py",
    scope: "global",
    enabled: false,
    createdAt: Date.now(),
    updatedAt: Date.now(),
  },
];

describe("Skills Component", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(api.settings.get).mockResolvedValue("[]");
  });

  it("loads and displays skills on mount", async () => {
    vi.mocked(api.skills.getAll).mockResolvedValue(mockSkills);
    renderWithProviders(<Skills />);

    await waitFor(() => {
      expect(api.skills.getAll).toHaveBeenCalled();
    });

    expect(screen.getByText("Test Skill 1")).toBeInTheDocument();
    expect(screen.getByText("A test description")).toBeInTheDocument();

    expect(screen.getByText("Disabled Skill")).toBeInTheDocument();
    expect(screen.getByText("Disabled")).toBeInTheDocument(); // Badge for disabled skill
  });

  it("shows 'No skills installed' when the list is empty", async () => {
    vi.mocked(api.skills.getAll).mockResolvedValue([]);
    renderWithProviders(<Skills />);

    await waitFor(() => {
      expect(screen.getByText("No skills installed.")).toBeInTheDocument();
    });
  });

  it("creates a new skill when New button is clicked", async () => {
    const user = userEvent.setup();
    vi.mocked(api.skills.getAll).mockResolvedValue([]);

    const newSkill: Skill = {
      id: "new-skill-id",
      name: "New Skill",
      description: "Describe this workflow",
      instructions: "Step 1\nStep 2",
      inputSchema: [],
      directoryPath: "/new/path",
      entryPoint: "run.sh",
      scope: "global",
      enabled: true,
      createdAt: Date.now(),
      updatedAt: Date.now(),
    };

    vi.mocked(api.skills.create).mockResolvedValue(newSkill);
    vi.mocked(api.skills.getAll).mockResolvedValueOnce([]).mockResolvedValueOnce([newSkill]);

    renderWithProviders(<Skills />);

    // Wait for initial load
    await waitFor(() => {
      expect(screen.queryByText("No skills installed.")).toBeInTheDocument();
    });

    const newBtn = screen.getByRole("button", { name: /new/i });
    await user.click(newBtn);

    await waitFor(() => {
      expect(api.skills.create).toHaveBeenCalledWith(
        expect.objectContaining({
          name: "New Skill",
        })
      );
    });

    // Should refresh list and select the new skill
    expect(api.skills.getAll).toHaveBeenCalledTimes(2);
    expect(screen.getByDisplayValue("New Skill")).toBeInTheDocument();
  });

  it("allows selecting and editing a skill", async () => {
    const user = userEvent.setup();
    vi.mocked(api.skills.getAll).mockResolvedValue(mockSkills);
    renderWithProviders(<Skills />);

    await waitFor(() => {
      expect(screen.getByText("Test Skill 1")).toBeInTheDocument();
    });

    await user.click(screen.getByText("Test Skill 1"));

    // Form should populate
    expect(screen.getByDisplayValue("Test Skill 1")).toBeInTheDocument();
    expect(screen.getByDisplayValue("A test description")).toBeInTheDocument();
    expect(screen.getByDisplayValue("run.sh")).toBeInTheDocument();

    // Check schema editor renders the param
    expect(screen.getByDisplayValue("param1")).toBeInTheDocument();

    // Edit and save
    const nameInput = screen.getByDisplayValue("Test Skill 1");
    await user.clear(nameInput);
    await user.type(nameInput, "Updated Name");

    vi.mocked(api.skills.update).mockResolvedValue({
      ...mockSkills[0],
      name: "Updated Name",
    });

    const saveBtn = screen.getByRole("button", { name: /save changes/i });
    await user.click(saveBtn);

    await waitFor(() => {
      expect(api.skills.update).toHaveBeenCalledWith(
        "skill-1",
        expect.objectContaining({
          name: "Updated Name",
        })
      );
    });
  });

  it("allows deleting a skill", async () => {
    const user = userEvent.setup();
    vi.mocked(api.skills.getAll).mockResolvedValue(mockSkills);
    renderWithProviders(<Skills />);

    await waitFor(() => {
      expect(screen.getByText("Test Skill 1")).toBeInTheDocument();
    });

    await user.click(screen.getByText("Test Skill 1"));

    vi.mocked(api.skills.delete).mockResolvedValue();

    const deleteBtn = screen.getByRole("button", { name: /delete skill/i });
    await user.click(deleteBtn);

    await waitFor(() => {
      expect(api.skills.delete).toHaveBeenCalledWith("skill-1");
    });

    // Skill should disappear from list
    await waitFor(() => {
      const btns = screen
        .queryAllByRole("button")
        .filter((b) => b.textContent?.includes("Test Skill 1"));
      expect(btns.length).toBe(0);
    });
  });

  it("opens folder in explorer", async () => {
    const user = userEvent.setup();
    vi.mocked(api.skills.getAll).mockResolvedValue(mockSkills);
    vi.mocked(api.app.openInExplorer).mockResolvedValue();

    renderWithProviders(<Skills />);

    await waitFor(() => {
      expect(screen.getByText("Test Skill 1")).toBeInTheDocument();
    });

    await user.click(screen.getByText("Test Skill 1"));

    const openFolderBtn = screen.getByRole("button", { name: /open folder/i });
    await user.click(openFolderBtn);

    expect(api.app.openInExplorer).toHaveBeenCalledWith("/test/path");
  });
});
