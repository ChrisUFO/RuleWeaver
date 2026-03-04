import type { CommandModel } from "@/types/command";
import type { Rule } from "@/types/rule";
import type { Skill } from "@/types/skill";
import type { ArtifactStatusEntry, ArtifactSyncStatus } from "@/types/status";

type MetricGroup = "rule" | "command" | "skill";

export interface ArtifactHealthSummary {
  tracked: number;
  synced: number;
  attention: number;
  unsupported: number;
  error: number;
}

export interface DashboardMetrics {
  totalArtifacts: number;
  activeArtifacts: number;
  rules: {
    count: number;
    enabled: number;
    health: ArtifactHealthSummary;
  };
  commands: {
    count: number;
    health: ArtifactHealthSummary;
  };
  skills: {
    count: number;
    enabled: number;
    health: ArtifactHealthSummary;
  };
  overallAttention: number;
  overallTrackedStatus: number;
}

export interface DashboardMetricsInput {
  rules: Rule[];
  commands: CommandModel[];
  skills: Skill[];
  statusEntries: ArtifactStatusEntry[];
}

const STATUS_SEVERITY: Record<ArtifactSyncStatus, number> = {
  synced: 0,
  unsupported: 1,
  out_of_date: 2,
  missing: 3,
  conflicted: 4,
  error: 5,
};

function metricGroupForStatus(entry: ArtifactStatusEntry): MetricGroup | null {
  if (entry.artifactType === "rule") {
    return "rule";
  }
  if (entry.artifactType === "command_stub" || entry.artifactType === "slash_command") {
    return "command";
  }
  if (entry.artifactType === "skill") {
    return "skill";
  }
  return null;
}

function chooseHigherSeverity(
  current: ArtifactSyncStatus | undefined,
  next: ArtifactSyncStatus
): ArtifactSyncStatus {
  if (!current) {
    return next;
  }
  const currentSeverity = STATUS_SEVERITY[current];
  const nextSeverity = STATUS_SEVERITY[next];
  if (nextSeverity > currentSeverity) {
    return next;
  }
  return current;
}

function summarizeHealth(
  statusEntries: ArtifactStatusEntry[],
  group: MetricGroup
): ArtifactHealthSummary {
  const statusByArtifact = new Map<string, ArtifactSyncStatus>();

  for (const entry of statusEntries) {
    if (metricGroupForStatus(entry) !== group) {
      continue;
    }
    const current = statusByArtifact.get(entry.artifactId);
    statusByArtifact.set(entry.artifactId, chooseHigherSeverity(current, entry.status));
  }

  let synced = 0;
  let attention = 0;
  let unsupported = 0;
  let error = 0;

  for (const status of statusByArtifact.values()) {
    if (status === "synced") {
      synced += 1;
      continue;
    }
    if (status === "unsupported") {
      unsupported += 1;
      continue;
    }
    if (status === "error") {
      error += 1;
    }
    attention += 1;
  }

  return {
    tracked: statusByArtifact.size,
    synced,
    attention,
    unsupported,
    error,
  };
}

export function buildDashboardMetrics(input: DashboardMetricsInput): DashboardMetrics {
  const rulesHealth = summarizeHealth(input.statusEntries, "rule");
  const commandsHealth = summarizeHealth(input.statusEntries, "command");
  const skillsHealth = summarizeHealth(input.statusEntries, "skill");

  const totalArtifacts = input.rules.length + input.commands.length + input.skills.length;
  const activeArtifacts =
    input.rules.filter((rule) => rule.enabled).length +
    input.commands.length +
    input.skills.filter((skill) => skill.enabled).length;

  const overallAttention =
    rulesHealth.attention + commandsHealth.attention + skillsHealth.attention;
  const overallTrackedStatus = rulesHealth.tracked + commandsHealth.tracked + skillsHealth.tracked;

  return {
    totalArtifacts,
    activeArtifacts,
    rules: {
      count: input.rules.length,
      enabled: input.rules.filter((rule) => rule.enabled).length,
      health: rulesHealth,
    },
    commands: {
      count: input.commands.length,
      health: commandsHealth,
    },
    skills: {
      count: input.skills.length,
      enabled: input.skills.filter((skill) => skill.enabled).length,
      health: skillsHealth,
    },
    overallAttention,
    overallTrackedStatus,
  };
}

export function emptyDashboardMetrics(): DashboardMetrics {
  return buildDashboardMetrics({
    rules: [],
    commands: [],
    skills: [],
    statusEntries: [],
  });
}
