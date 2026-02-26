import type { AdapterType, Scope } from "./rule";

export type ArtifactType = "rule" | "command_stub" | "slash_command" | "skill";

export type ArtifactSyncStatus =
  | "synced"
  | "out_of_date"
  | "missing"
  | "conflicted"
  | "unsupported"
  | "error";

export interface ArtifactStatusEntry {
  id: string;
  artifactId: string;
  artifactName: string;
  artifactType: ArtifactType;
  adapter: AdapterType;
  scope: Scope;
  repoRoot?: string;
  status: ArtifactSyncStatus;
  expectedPath: string;
  lastOperation?: string;
  lastOperationAt?: string;
  detail?: string;
}

export interface StatusFilter {
  artifactType?: ArtifactType;
  adapter?: AdapterType;
  scope?: Scope;
  repoRoot?: string;
  status?: ArtifactSyncStatus;
}

export interface RepairResult {
  entryId: string;
  success: boolean;
  error?: string;
  updatedEntry?: ArtifactStatusEntry;
}

export interface StatusSummary {
  total: number;
  synced: number;
  outOfDate: number;
  missing: number;
  conflicted: number;
  unsupported: number;
  error: number;
}

export const ARTIFACT_TYPE_LABELS: Record<ArtifactType, string> = {
  rule: "Rule",
  command_stub: "Command Stub",
  slash_command: "Slash Command",
  skill: "Skill",
};

export const SYNC_STATUS_CONFIG: Record<
  ArtifactSyncStatus,
  { label: string; color: string; bgColor: string }
> = {
  synced: {
    label: "Synced",
    color: "text-green-500",
    bgColor: "bg-green-500/10 border-green-500/20",
  },
  out_of_date: {
    label: "Out of Date",
    color: "text-yellow-500",
    bgColor: "bg-yellow-500/10 border-yellow-500/20",
  },
  missing: {
    label: "Missing",
    color: "text-red-500",
    bgColor: "bg-red-500/10 border-red-500/20",
  },
  conflicted: {
    label: "Conflicted",
    color: "text-red-500",
    bgColor: "bg-red-500/10 border-red-500/20",
  },
  unsupported: {
    label: "Unsupported",
    color: "text-gray-500",
    bgColor: "bg-gray-500/10 border-gray-500/20",
  },
  error: {
    label: "Error",
    color: "text-red-600",
    bgColor: "bg-red-600/10 border-red-600/20",
  },
};
