import { describe, expect, it } from "vitest";
import { buildDashboardMetrics, emptyDashboardMetrics } from "@/lib/dashboard-metrics";
import type { ArtifactStatusEntry } from "@/types/status";

const now = Date.now();

describe("dashboard metrics", () => {
  it("computes entity counts across rules, commands, and skills", () => {
    const metrics = buildDashboardMetrics({
      rules: [
        {
          id: "rule-1",
          name: "Rule 1",
          description: "",
          content: "",
          scope: "global",
          targetPaths: null,
          enabledAdapters: ["gemini"],
          enabled: true,
          createdAt: now,
          updatedAt: now,
        },
        {
          id: "rule-2",
          name: "Rule 2",
          description: "",
          content: "",
          scope: "local",
          targetPaths: ["/repo"],
          enabledAdapters: ["opencode"],
          enabled: false,
          createdAt: now,
          updatedAt: now,
        },
      ],
      commands: [
        {
          id: "cmd-1",
          name: "Command 1",
          description: "",
          script: "echo 1",
          arguments: [],
          exposeViaMcp: true,
          isPlaceholder: false,
          createdAt: now,
          updatedAt: now,
        },
      ],
      skills: [
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
          createdAt: now,
          updatedAt: now,
        },
      ],
      statusEntries: [],
    });

    expect(metrics.totalArtifacts).toBe(4);
    expect(metrics.activeArtifacts).toBe(3);
    expect(metrics.rules.count).toBe(2);
    expect(metrics.commands.count).toBe(1);
    expect(metrics.skills.count).toBe(1);
  });

  it("aggregates command health from command stub and slash entries", () => {
    const statusEntries: ArtifactStatusEntry[] = [
      {
        id: "1",
        artifactId: "cmd-1",
        artifactName: "Command 1",
        artifactType: "command_stub",
        adapter: "opencode",
        scope: "global",
        status: "synced",
        expectedPath: "/path/one",
      },
      {
        id: "2",
        artifactId: "cmd-1",
        artifactName: "Command 1",
        artifactType: "slash_command",
        adapter: "claude-code",
        scope: "global",
        status: "missing",
        expectedPath: "/path/two",
      },
      {
        id: "3",
        artifactId: "cmd-2",
        artifactName: "Command 2",
        artifactType: "slash_command",
        adapter: "gemini",
        scope: "global",
        status: "synced",
        expectedPath: "/path/three",
      },
    ];

    const metrics = buildDashboardMetrics({
      rules: [],
      commands: [],
      skills: [],
      statusEntries,
    });

    expect(metrics.commands.health.tracked).toBe(2);
    expect(metrics.commands.health.synced).toBe(1);
    expect(metrics.commands.health.attention).toBe(1);
    expect(metrics.overallAttention).toBe(1);
  });

  it("returns zeroed metrics for empty inputs", () => {
    const metrics = emptyDashboardMetrics();
    expect(metrics.totalArtifacts).toBe(0);
    expect(metrics.activeArtifacts).toBe(0);
    expect(metrics.overallAttention).toBe(0);
    expect(metrics.overallTrackedStatus).toBe(0);
  });

  it("does not mask unsupported status when another adapter is synced", () => {
    const statusEntries: ArtifactStatusEntry[] = [
      {
        id: "1",
        artifactId: "rule-1",
        artifactName: "Rule 1",
        artifactType: "rule",
        adapter: "opencode",
        scope: "global",
        status: "unsupported",
        expectedPath: "/path/one",
      },
      {
        id: "2",
        artifactId: "rule-1",
        artifactName: "Rule 1",
        artifactType: "rule",
        adapter: "claude-code",
        scope: "global",
        status: "synced",
        expectedPath: "/path/two",
      },
    ];

    const metrics = buildDashboardMetrics({
      rules: [],
      commands: [],
      skills: [],
      statusEntries,
    });

    expect(metrics.rules.health.tracked).toBe(1);
    expect(metrics.rules.health.unsupported).toBe(1);
    expect(metrics.rules.health.synced).toBe(0);
  });
});
