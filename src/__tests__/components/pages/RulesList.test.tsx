import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { fireEvent } from "@testing-library/react";
import { RulesList } from "../../../components/pages/RulesList";
import { ToastProvider } from "../../../components/ui/toast";
import type { Rule } from "../../../types/rule";

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn(),
}));

vi.mock("../../../lib/tauri", () => ({
  api: {
    ruleImport: {
      scanAiToolCandidates: vi.fn(),
      importAiToolRules: vi.fn(),
      getHistory: vi.fn(),
      scanFromFile: vi.fn(),
      scanFromDirectory: vi.fn(),
      scanFromUrl: vi.fn(),
      scanFromClipboard: vi.fn(),
      importFromFile: vi.fn(),
      importFromDirectory: vi.fn(),
      importFromUrl: vi.fn(),
      importFromClipboard: vi.fn(),
    },
  },
}));

const mockStore = {
  rules: [] as Rule[],
  fetchRules: vi.fn(),
  toggleRule: vi.fn(),
  deleteRule: vi.fn(),
  bulkDeleteRules: vi.fn(),
  duplicateRule: vi.fn(),
  restoreRecentlyDeleted: vi.fn(),
  isLoading: false,
};

vi.mock("../../../stores/rulesStore", () => ({
  useRulesStore: () => mockStore,
}));

const mockRegistryStore = {
  tools: [
    {
      id: "gemini",
      name: "Gemini",
      description: "Gemini Adapter",
      icon: "gemini",
      fileFormat: "md",
      capabilities: { hasChat: true, hasFileWriting: true, hasCommandExecution: true },
      paths: { globalPath: "~/.gemini/GEMINI.md", localPathTemplate: ".gemini/GEMINI.md" },
    },
    {
      id: "cline",
      name: "Cline",
      description: "Cline Adapter",
      icon: "cline",
      fileFormat: "md",
      capabilities: { hasChat: true, hasFileWriting: true, hasCommandExecution: true },
      paths: { globalPath: "~/.clinerules", localPathTemplate: ".clinerules" },
    },
  ],
  isLoading: false,
  error: null,
  fetchTools: vi.fn(),
};

vi.mock("../../../stores/registryStore", () => ({
  useRegistryStore: () => mockRegistryStore,
}));

const renderWithProviders = (ui: React.ReactElement) => render(<ToastProvider>{ui}</ToastProvider>);

describe("RulesList import workflow", () => {
  beforeEach(async () => {
    vi.clearAllMocks();
    const { api } = await import("../../../lib/tauri");
    const { open } = await import("@tauri-apps/plugin-dialog");
    vi.mocked(api.ruleImport.getHistory).mockResolvedValue([]);
    vi.mocked(open).mockResolvedValue(null);
  });

  it("opens AI import preview dialog after scan", async () => {
    const { api } = await import("../../../lib/tauri");
    vi.mocked(api.ruleImport.scanAiToolCandidates).mockResolvedValue({
      candidates: [
        {
          id: "cand-1",
          sourceType: "ai_tool",
          sourceLabel: "Cline",
          sourcePath: "C:/tmp/.clinerules",
          sourceTool: "cline",
          name: "quality",
          proposedName: "quality-cline",
          content: "rule content",
          scope: "global",
          targetPaths: null,
          enabledAdapters: ["cline"],
          artifactType: "rule",
          contentHash: "hash",
          fileSize: 12,
        },
      ],
      errors: [],
    });

    renderWithProviders(<RulesList onSelectRule={vi.fn()} onCreateRule={vi.fn()} />);

    await userEvent.click(screen.getByRole("button", { name: /import ai/i }));

    await waitFor(() => {
      expect(api.ruleImport.scanAiToolCandidates).toHaveBeenCalled();
    });

    expect(screen.getByText("Import Existing AI Tool Rules")).toBeInTheDocument();
    expect(screen.getByText("quality-cline")).toBeInTheDocument();
  });

  it("imports selected AI candidates from preview dialog", async () => {
    const { api } = await import("../../../lib/tauri");
    vi.mocked(api.ruleImport.scanAiToolCandidates).mockResolvedValue({
      candidates: [
        {
          id: "cand-1",
          sourceType: "ai_tool",
          sourceLabel: "Cline",
          sourcePath: "C:/tmp/.clinerules",
          sourceTool: "cline",
          name: "quality",
          proposedName: "quality-cline",
          content: "rule content",
          scope: "global",
          targetPaths: null,
          enabledAdapters: ["cline"],
          artifactType: "rule",
          contentHash: "hash",
          fileSize: 12,
        },
      ],
      errors: [],
    });
    vi.mocked(api.ruleImport.importAiToolRules).mockResolvedValue({
      imported: [],
      importedRules: [],
      importedCommands: [],
      importedSkills: [],
      skipped: [],
      conflicts: [],
      errors: [],
    });

    renderWithProviders(<RulesList onSelectRule={vi.fn()} onCreateRule={vi.fn()} />);

    await userEvent.click(screen.getByRole("button", { name: /import ai/i }));
    await waitFor(() => expect(screen.getByText("quality-cline")).toBeInTheDocument());

    await userEvent.click(screen.getByRole("button", { name: /process import/i }));

    await waitFor(() => {
      expect(api.ruleImport.importAiToolRules).toHaveBeenCalledWith(
        expect.objectContaining({
          conflictMode: "rename",
          selectedCandidateIds: ["cand-1"],
        })
      );
    });
  });

  it("uses selected conflict mode when importing", async () => {
    const { api } = await import("../../../lib/tauri");
    vi.mocked(api.ruleImport.scanAiToolCandidates).mockResolvedValue({
      candidates: [
        {
          id: "cand-1",
          sourceType: "ai_tool",
          sourceLabel: "Cline",
          sourcePath: "C:/tmp/.clinerules",
          sourceTool: "cline",
          name: "quality",
          proposedName: "quality-cline",
          content: "rule content",
          scope: "global",
          targetPaths: null,
          enabledAdapters: ["cline"],
          artifactType: "rule",
          contentHash: "hash",
          fileSize: 12,
        },
      ],
      errors: [],
    });
    vi.mocked(api.ruleImport.importAiToolRules).mockResolvedValue({
      imported: [],
      importedRules: [],
      importedCommands: [],
      importedSkills: [],
      skipped: [],
      conflicts: [],
      errors: [],
    });

    renderWithProviders(<RulesList onSelectRule={vi.fn()} onCreateRule={vi.fn()} />);

    await userEvent.click(screen.getByRole("button", { name: /import ai/i }));
    await waitFor(() => expect(screen.getByText("quality-cline")).toBeInTheDocument());

    await userEvent.selectOptions(screen.getByLabelText(/conflict mode/i), "replace");
    await userEvent.click(screen.getByRole("button", { name: /process import/i }));

    await waitFor(() => {
      expect(api.ruleImport.importAiToolRules).toHaveBeenCalledWith(
        expect.objectContaining({
          conflictMode: "replace",
          selectedCandidateIds: ["cand-1"],
        })
      );
    });
  });

  it("scans and imports from file through preview flow", async () => {
    const { api } = await import("../../../lib/tauri");
    const { open } = await import("@tauri-apps/plugin-dialog");
    vi.mocked(open).mockResolvedValue("C:/tmp/rule.md");
    vi.mocked(api.ruleImport.scanFromFile).mockResolvedValue({
      candidates: [
        {
          id: "file-1",
          sourceType: "file",
          sourceLabel: "File",
          sourcePath: "C:/tmp/rule.md",
          sourceTool: undefined,
          name: "rule",
          proposedName: "rule",
          content: "rule content",
          scope: "global",
          targetPaths: null,
          enabledAdapters: ["gemini"],
          artifactType: "rule",
          contentHash: "hash",
          fileSize: 12,
        },
      ],
      errors: [],
    });
    vi.mocked(api.ruleImport.importFromFile).mockResolvedValue({
      imported: [],
      importedRules: [],
      importedCommands: [],
      importedSkills: [],
      skipped: [],
      conflicts: [],
      errors: [],
    });

    renderWithProviders(<RulesList onSelectRule={vi.fn()} onCreateRule={vi.fn()} />);

    await userEvent.click(screen.getByRole("button", { name: /import file/i }));
    await waitFor(() => {
      expect(api.ruleImport.scanFromFile).toHaveBeenCalledWith("C:/tmp/rule.md");
    });

    expect(screen.getByText("Import Rules From File")).toBeInTheDocument();
    expect(screen.getAllByText(/C:\/tmp\/rule\.md/i).length).toBeGreaterThan(0);
    await userEvent.click(screen.getByRole("button", { name: /process import/i }));

    await waitFor(() => {
      expect(api.ruleImport.importFromFile).toHaveBeenCalledWith(
        "C:/tmp/rule.md",
        expect.objectContaining({ selectedCandidateIds: ["file-1"] })
      );
    });
  });

  it("sends scope and adapter overrides when selected", async () => {
    const { api } = await import("../../../lib/tauri");
    vi.mocked(api.ruleImport.scanAiToolCandidates).mockResolvedValue({
      candidates: [
        {
          id: "cand-1",
          sourceType: "ai_tool",
          sourceLabel: "Cline",
          sourcePath: "C:/tmp/.clinerules",
          sourceTool: "cline",
          name: "quality",
          proposedName: "quality-cline",
          content: "rule content",
          scope: "global",
          targetPaths: null,
          enabledAdapters: ["cline"],
          artifactType: "rule",
          contentHash: "hash",
          fileSize: 12,
        },
      ],
      errors: [],
    });
    vi.mocked(api.ruleImport.importAiToolRules).mockResolvedValue({
      imported: [],
      importedRules: [],
      importedCommands: [],
      importedSkills: [],
      skipped: [],
      conflicts: [],
      errors: [],
    });

    renderWithProviders(<RulesList onSelectRule={vi.fn()} onCreateRule={vi.fn()} />);

    await userEvent.click(screen.getByRole("button", { name: /import ai/i }));
    await waitFor(() => expect(screen.getByText("quality-cline")).toBeInTheDocument());

    await userEvent.selectOptions(screen.getByLabelText(/scope override/i), "local");
    await userEvent.click(screen.getByLabelText(/enable adapter override/i));
    await userEvent.click(screen.getByLabelText(/use adapter gemini/i));

    await userEvent.click(screen.getByRole("button", { name: /process import/i }));

    await waitFor(() => {
      expect(api.ruleImport.importAiToolRules).toHaveBeenCalledWith(
        expect.objectContaining({
          defaultScope: "local",
          defaultAdapters: ["gemini"],
          selectedCandidateIds: ["cand-1"],
        })
      );
    });
  });

  it("shows URL required validation when scanning URL without input", async () => {
    renderWithProviders(<RulesList onSelectRule={vi.fn()} onCreateRule={vi.fn()} />);

    await userEvent.click(screen.getByRole("button", { name: /import url/i }));
    await userEvent.click(screen.getByRole("button", { name: /scan remote source/i }));

    expect(screen.getByText(/URL Required/i)).toBeInTheDocument();
  });

  it("shows drop not supported for dropped files without path", async () => {
    renderWithProviders(<RulesList onSelectRule={vi.fn()} onCreateRule={vi.fn()} />);

    const dropZone = screen.getByText(/Drag and drop a rule file here/i).closest("div");
    expect(dropZone).toBeTruthy();

    const file = new File(["x"], "rule.md", { type: "text/markdown" });
    const dataTransfer = { files: [file] } as unknown as DataTransfer;

    fireEvent.drop(dropZone as Element, { dataTransfer });

    expect(screen.getByText(/Drop Not Supported/i)).toBeInTheDocument();
  });
});
