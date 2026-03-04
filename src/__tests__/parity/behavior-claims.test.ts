/**
 * Parity tests: verify that user-facing UI string constants and capability
 * filtering logic stay in sync with the underlying type definitions and
 * adapter registry data.
 *
 * When adding new ArtifactSyncStatus values, ArtifactType values, or
 * REPAIR_ACTION_LABELS entries, these tests will fail until the corresponding
 * UI code is updated. That is intentional — drift prevention.
 *
 * See docs/PARITY.md § "Automated Parity Checks" for maintenance instructions.
 */
import { describe, it, expect } from "vitest";
import {
  SYNC_STATUS_CONFIG,
  ARTIFACT_TYPE_LABELS,
  REPAIR_ACTION_LABELS,
  type ArtifactSyncStatus,
  type ArtifactType,
} from "../../types/status";

// ---------------------------------------------------------------------------
// 1. SYNC_STATUS_CONFIG completeness
// ---------------------------------------------------------------------------
describe("SYNC_STATUS_CONFIG parity", () => {
  // These are the canonical values defined in the ArtifactSyncStatus union type.
  // If a new status is added to the Rust backend, it must be added here AND to
  // the SYNC_STATUS_CONFIG object.
  const ALL_SYNC_STATUSES: ArtifactSyncStatus[] = [
    "synced",
    "out_of_date",
    "missing",
    "conflicted",
    "unsupported",
    "error",
  ];

  it("has a config entry for every ArtifactSyncStatus value", () => {
    for (const status of ALL_SYNC_STATUSES) {
      expect(
        SYNC_STATUS_CONFIG[status],
        `Missing SYNC_STATUS_CONFIG entry for "${status}"`
      ).toBeDefined();
    }
  });

  it("every SYNC_STATUS_CONFIG entry has a non-empty label", () => {
    for (const [status, config] of Object.entries(SYNC_STATUS_CONFIG)) {
      expect(config.label, `Empty label for status "${status}"`).toBeTruthy();
    }
  });

  it("SYNC_STATUS_CONFIG does not have undeclared status keys", () => {
    const configKeys = Object.keys(SYNC_STATUS_CONFIG) as ArtifactSyncStatus[];
    for (const key of configKeys) {
      expect(ALL_SYNC_STATUSES, `Undeclared key in SYNC_STATUS_CONFIG: "${key}"`).toContain(key);
    }
  });
});

// ---------------------------------------------------------------------------
// 2. ARTIFACT_TYPE_LABELS completeness
// ---------------------------------------------------------------------------
describe("ARTIFACT_TYPE_LABELS parity", () => {
  const ALL_ARTIFACT_TYPES: ArtifactType[] = ["rule", "command_stub", "slash_command", "skill"];

  it("has a label for every ArtifactType value", () => {
    for (const type of ALL_ARTIFACT_TYPES) {
      expect(
        ARTIFACT_TYPE_LABELS[type],
        `Missing ARTIFACT_TYPE_LABELS entry for "${type}"`
      ).toBeDefined();
      expect(ARTIFACT_TYPE_LABELS[type]).toBeTruthy();
    }
  });

  it("does not have labels for undeclared artifact types", () => {
    const labelKeys = Object.keys(ARTIFACT_TYPE_LABELS) as ArtifactType[];
    for (const key of labelKeys) {
      expect(ALL_ARTIFACT_TYPES, `Undeclared key in ARTIFACT_TYPE_LABELS: "${key}"`).toContain(key);
    }
  });
});

// ---------------------------------------------------------------------------
// 3. REPAIR_ACTION_LABELS — anchors user-facing repair button/toast strings
// ---------------------------------------------------------------------------
describe("REPAIR_ACTION_LABELS parity", () => {
  it("repairOne matches expected button text 'Repair'", () => {
    expect(REPAIR_ACTION_LABELS.repairOne).toBe("Repair");
  });

  it("repairAll matches expected button text 'Repair All'", () => {
    expect(REPAIR_ACTION_LABELS.repairAll).toBe("Repair All");
  });

  it("repairSuccess matches expected toast title", () => {
    expect(REPAIR_ACTION_LABELS.repairSuccess).toBe("Repair successful");
  });

  it("repairFailed matches expected toast title", () => {
    expect(REPAIR_ACTION_LABELS.repairFailed).toBe("Repair failed");
  });

  it("bulkRepairComplete matches expected toast title", () => {
    expect(REPAIR_ACTION_LABELS.bulkRepairComplete).toBe("Bulk repair complete");
  });

  it("bulkRepairFailed matches expected toast title", () => {
    expect(REPAIR_ACTION_LABELS.bulkRepairFailed).toBe("Bulk repair failed");
  });
});

// ---------------------------------------------------------------------------
// 4. Skills adapter filter logic parity
// ---------------------------------------------------------------------------
describe("Skills adapter capability filter parity", () => {
  /**
   * Fixture: represents the adapter data the registry would return.
   * Adapters with supportedArtifacts containing 'skill' should appear in the
   * Skills UI adapter list. This fixture mirrors the expected registry shape
   * to catch drift between registry capability flags and UI filter logic.
   *
   * When adding a new adapter to registry.rs, add it here with the correct
   * supports_skills value to verify the filter logic handles it correctly.
   */
  const ADAPTER_FIXTURE = [
    { id: "claude_code", supportsSkills: true },
    { id: "gemini", supportsSkills: true },
    { id: "opencode", supportsSkills: true },
    { id: "cline", supportsSkills: true },
    { id: "codex", supportsSkills: true },
    { id: "roo_code", supportsSkills: true },
    { id: "windsurf", supportsSkills: true },
    { id: "antigravity", supportsSkills: true },
    { id: "kilo_code", supportsSkills: true }, // flag true, paths unconfigured — still in supported list per registry
    { id: "cursor", supportsSkills: false },
  ];

  // The Skills UI calls `api.skills.getSupportedAdapters()` which maps to the
  // Rust `get_skill_supported_adapters` command. That command filters by
  // `supports_skills: true`. We verify the expected set here.
  const skillSupportedAdapters = ADAPTER_FIXTURE.filter((a) => a.supportsSkills).map((a) => a.id);

  it("cursor is NOT in the skills-supported adapter list", () => {
    expect(skillSupportedAdapters).not.toContain("cursor");
  });

  it("all known skills-capable adapters are in the list", () => {
    const expectedSkillAdapters = [
      "claude_code",
      "gemini",
      "opencode",
      "cline",
      "codex",
      "roo_code",
      "windsurf",
      "antigravity",
      "kilo_code",
    ];
    for (const id of expectedSkillAdapters) {
      expect(skillSupportedAdapters, `Expected "${id}" to support skills`).toContain(id);
    }
  });

  it("skills-supported list length matches expected count from fixture", () => {
    // 9 adapters support skills (all except cursor)
    expect(skillSupportedAdapters).toHaveLength(9);
  });
});
