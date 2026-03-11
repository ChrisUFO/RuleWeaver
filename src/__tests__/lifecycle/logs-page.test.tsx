import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { Logs } from "@/components/pages/Logs";
import { api } from "@/lib/tauri";
import { renderWithProviders } from "./test-utils";

const saveMock = vi.fn();

vi.mock("@tauri-apps/plugin-dialog", () => ({
  save: (...args: unknown[]) => saveMock(...args),
}));

vi.mock("@/lib/tauri", () => ({
  api: {
    observability: {
      list: vi.fn(),
      export: vi.fn(),
    },
  },
}));

const sampleEvent = {
  id: "event-1",
  timestamp: 1_710_000_000,
  eventType: "commandRun" as const,
  status: "error" as const,
  source: "mcp",
  entityName: "Deploy docs",
  summary: "Command execution failed",
  metadata: '{"toolName":"Deploy docs"}',
  stdoutExcerpt: null,
  stderrExcerpt: "timed out",
  durationMs: 412,
  exitCode: 1,
  failureClass: "timeout",
  attemptNumber: 1,
  isRedacted: true,
};

describe("Logs page", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(api.observability.list).mockResolvedValue([sampleEvent]);
    vi.mocked(api.observability.export).mockResolvedValue(1);
    saveMock.mockResolvedValue("C:/tmp/ruleweaver-logs.json");
  });

  it("loads and refetches logs when filters change", async () => {
    const user = userEvent.setup();
    renderWithProviders(<Logs />);

    expect(await screen.findByText("Command execution failed")).toBeInTheDocument();
    expect(api.observability.list).toHaveBeenCalledWith(expect.objectContaining({ limit: 250 }));

    await user.type(screen.getByLabelText("Tool or skill"), "Deploy");

    await waitFor(() => {
      expect(api.observability.list).toHaveBeenLastCalledWith(
        expect.objectContaining({ entityName: "Deploy", limit: 250 })
      );
    });

    await user.selectOptions(screen.getByLabelText("Event type"), "commandRun");

    await waitFor(() => {
      expect(api.observability.list).toHaveBeenLastCalledWith(
        expect.objectContaining({ entityName: "Deploy", eventType: "commandRun", limit: 250 })
      );
    });
  });

  it("exports selected log entries", async () => {
    const user = userEvent.setup();
    renderWithProviders(<Logs />);

    expect(await screen.findByText("Command execution failed")).toBeInTheDocument();

    await user.click(screen.getByLabelText("Select log Command execution failed"));
    await user.click(screen.getByRole("button", { name: /export selected/i }));

    await waitFor(() => {
      expect(api.observability.export).toHaveBeenCalledWith(
        "C:/tmp/ruleweaver-logs.json",
        expect.objectContaining({ limit: 250 }),
        ["event-1"]
      );
    });
  });

  it("shows an empty state when no log entries match", async () => {
    vi.mocked(api.observability.list).mockResolvedValueOnce([]);

    renderWithProviders(<Logs />);

    expect(
      await screen.findByText("No log entries match the current filters.")
    ).toBeInTheDocument();
  });

  it("shows an inline error when loading logs fails", async () => {
    vi.mocked(api.observability.list).mockRejectedValueOnce(new Error("backend unavailable"));

    renderWithProviders(<Logs />);

    expect((await screen.findAllByText("backend unavailable")).length).toBeGreaterThan(0);
  });
});
