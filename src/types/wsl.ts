import type { AdapterType } from "./rule";

export interface WslDistribution {
  name: string;
  is_default: boolean;
  state: string;
  version: number;
}

export interface WslAdapterConfig {
  enabled: boolean;
  distribution: string;
  home_dir: string;
}

export type WslMode = "disabled" | "auto" | "manual";

export interface WslConfig {
  enabled: boolean;
  mode: WslMode;
  default_distribution: string | null;
  adapters: Record<AdapterType, WslAdapterConfig>;
}
