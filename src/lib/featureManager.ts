const FLAGS: Record<string, boolean> = {
  native_skill_sync: true,
  unified_artifact_status: true,
  execution_redaction: true,
};

export const FEATURE_FLAGS = {
  NATIVE_SKILL_SYNC: "native_skill_sync",
  UNIFIED_ARTIFACT_STATUS: "unified_artifact_status",
  EXECUTION_REDACTION: "execution_redaction",
} as const;

export type FeatureFlagKey = (typeof FEATURE_FLAGS)[keyof typeof FEATURE_FLAGS];

export const featureManager = {
  isEnabled: (key: FeatureFlagKey): boolean => {
    // Check for process.env override (Vite/Tauri)
    const envKey = `RULEWEAVER_FEATURE_${key.toUpperCase()}`;
    const envVal =
      (import.meta as unknown as { env: Record<string, string> }).env?.[envKey] ||
      (process as unknown as { env: Record<string, string> }).env?.[envKey];
    if (envVal !== undefined) {
      return envVal === "true" || envVal === "1";
    }
    return FLAGS[key] ?? false;
  },
};
