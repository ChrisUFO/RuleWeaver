export type SecretScope = "global" | "workspace" | "command" | "skill";

export interface ScopedSecret {
  id: string;
  key: string;
  value: string;
  scope: SecretScope;
  workspacePath?: string | null;
  artifactId?: string | null;
  createdAt: number;
  updatedAt: number;
}

export interface EffectiveSecret {
  key: string;
  value: string;
  sourceScope: SecretScope;
  workspacePath?: string | null;
  artifactId?: string | null;
}

export interface SecretStorageStatus {
  backend: string;
  storesSecretsInOsCredentialManager: boolean;
  exportsIncludeSecrets: boolean;
  importsIncludeSecrets: boolean;
}

export interface UpsertScopedSecretInput {
  key: string;
  value: string;
  scope: SecretScope;
  workspacePath?: string | null;
  artifactId?: string | null;
}

export interface DeleteScopedSecretInput {
  key: string;
  scope: SecretScope;
  workspacePath?: string | null;
  artifactId?: string | null;
}
