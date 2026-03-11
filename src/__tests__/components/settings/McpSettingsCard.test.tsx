import { beforeEach, describe, expect, it, vi } from "vitest";
import { act, fireEvent, screen } from "@testing-library/react";
import { McpSettingsCard } from "@/components/settings/McpSettingsCard";
import { renderWithProviders } from "@/__tests__/lifecycle/test-utils";
import type { McpConnectionInstructions, McpStatus } from "@/types/command";

const baseStatus: McpStatus = {
  running: true,
  port: 4545,
  uptimeSeconds: 12,
  apiToken: "test-token",
  isWatching: false,
  endpointUrl: "http://127.0.0.1:4545",
  healthState: "degraded",
  statusMessage: "MCP server is reachable, but no commands or skills are exposed yet",
  diagnostics: [
    {
      code: "no_tools_exposed",
      severity: "warning",
      title: "No MCP tools or skills exposed",
      message: "Clients can connect, but they will not see any callable tools yet.",
      hint: "Expose a command via MCP or add a skill.",
    },
    {
      code: "client_configuration",
      severity: "info",
      title: "Verify client configuration",
      message: "Standalone clients should target the current endpoint.",
      hint: "Restart the client after config changes.",
    },
  ],
  availableCommands: 0,
  availableSkills: 0,
  watchTargetCount: 0,
};

const baseInstructions: McpConnectionInstructions = {
  claudeCodeJson: '{"mcpServers":{"ruleweaver":{"url":"http://127.0.0.1:4545"}}}',
  opencodeJson: '{"mcp":{"servers":[{"name":"ruleweaver"}]}}',
  standaloneCommand: "ruleweaver-mcp --port 4545",
  apiToken: "test-token",
  endpointUrl: "http://127.0.0.1:4545",
  authHeaderName: "X-API-Key",
  tokenEnvVarName: "RULEWEAVER_MCP_TOKEN",
};

const noopAsync = vi.fn().mockResolvedValue(undefined);
const writeText = vi.fn().mockResolvedValue(undefined);

describe("McpSettingsCard", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    Object.defineProperty(window.navigator, "clipboard", {
      configurable: true,
      value: { writeText },
    });
  });

  it("renders standalone onboarding and actionable diagnostics", () => {
    renderWithProviders(
      <McpSettingsCard
        mcpStatus={baseStatus}
        mcpInstructions={baseInstructions}
        mcpLogs={["Starting MCP server"]}
        isMcpLoading={false}
        mcpAutoStart={false}
        minimizeToTray={true}
        launchOnStartup={false}
        onStart={noopAsync}
        onStop={noopAsync}
        onRefresh={noopAsync}
        onToggleAutoStart={noopAsync}
        onToggleMinimizeToTray={noopAsync}
        onToggleLaunchOnStartup={noopAsync}
      />
    );

    expect(screen.getByText(/standalone onboarding/i)).toBeInTheDocument();
    expect(screen.getByText(/no mcp tools or skills exposed/i)).toBeInTheDocument();
    expect(screen.getByText(/verify client configuration/i)).toBeInTheDocument();
    expect(screen.getByText(/copy a working client config from this screen/i)).toBeInTheDocument();
    expect(screen.getAllByText(/RULEWEAVER_MCP_TOKEN/i).length).toBeGreaterThan(0);
    expect(screen.getAllByText(/http:\/\/127.0.0.1:4545/i).length).toBeGreaterThan(0);
  });

  it("copies endpoint, token, and standalone command details to the clipboard", async () => {
    renderWithProviders(
      <McpSettingsCard
        mcpStatus={baseStatus}
        mcpInstructions={baseInstructions}
        mcpLogs={[]}
        isMcpLoading={false}
        mcpAutoStart={false}
        minimizeToTray={true}
        launchOnStartup={false}
        onStart={noopAsync}
        onStop={noopAsync}
        onRefresh={noopAsync}
        onToggleAutoStart={noopAsync}
        onToggleMinimizeToTray={noopAsync}
        onToggleLaunchOnStartup={noopAsync}
      />
    );

    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: /copy endpoint/i }));
      fireEvent.click(screen.getByRole("button", { name: /copy token/i }));
      fireEvent.click(screen.getByRole("button", { name: /copy command/i }));
    });

    expect(writeText).toHaveBeenNthCalledWith(1, "http://127.0.0.1:4545");
    expect(writeText).toHaveBeenNthCalledWith(2, "test-token");
    expect(writeText).toHaveBeenNthCalledWith(3, "ruleweaver-mcp --port 4545");
    expect(writeText.mock.calls[2]?.[0]).not.toContain("test-token");
  });
});
