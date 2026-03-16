import type { AdapterType } from "./rule";

export interface WslDistribution {
  name: string;
  isDefault: boolean;
  version: number;
}

export type WslMode = "windows" | "wsl";

export interface WslAdapterConfig {
  mode: WslMode;
  distribution?: string;
  homeDir?: string;
}

export interface WslConfig {
  enabled: boolean;
  defaultDistribution: string | null;
  adapters: Partial<Record<AdapterType, WslAdapterConfig>>;
}
