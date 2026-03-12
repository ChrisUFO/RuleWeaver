import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { ScopedSecretsCard } from "@/components/settings/ScopedSecretsCard";
import type { ScopedSecret } from "@/types/secret";

const scopedSecrets: ScopedSecret[] = [
  {
    id: "global-1",
    key: "PROJECT_API_KEY",
    value: "••••••••",
    scope: "global",
    createdAt: 1,
    updatedAt: 1,
  },
  {
    id: "workspace-1",
    key: "PROJECT_API_KEY",
    value: "••••••••",
    scope: "workspace",
    workspacePath: "C:/repo-a",
    createdAt: 1,
    updatedAt: 1,
  },
];

describe("ScopedSecretsCard", () => {
  it("shows saving feedback and disables secret actions while saving", () => {
    render(
      <ScopedSecretsCard
        repositoryRoots={["C:/repo-a"]}
        scopedSecrets={scopedSecrets}
        secretStorageStatus={{
          backend: "os-keychain",
          storesSecretsInOsCredentialManager: true,
          exportsIncludeSecrets: false,
          importsIncludeSecrets: false,
        }}
        selectedWorkspace="C:/repo-a"
        isLoading={false}
        isSaving
        onWorkspaceChange={vi.fn()}
        onSaveGlobalSecret={vi.fn().mockResolvedValue(undefined)}
        onSaveWorkspaceSecret={vi.fn().mockResolvedValue(undefined)}
        onDeleteGlobalSecret={vi.fn().mockResolvedValue(undefined)}
        onDeleteWorkspaceSecret={vi.fn().mockResolvedValue(undefined)}
      />
    );

    expect(screen.getByText(/saving secret changes/i)).toBeInTheDocument();
    expect(screen.getByText(/secure secret handling/i)).toBeInTheDocument();
    expect(screen.getByText(/os credential manager/i)).toBeInTheDocument();
    expect(screen.getByText(/never shown again in plain text/i)).toBeInTheDocument();
    expect(screen.getByText(/export\/import only transfers metadata/i)).toBeInTheDocument();
    expect(screen.getAllByRole("button", { name: /saving/i }).length).toBeGreaterThan(0);
    screen.getAllByRole("button", { name: /saving/i }).forEach((button) => {
      expect(button).toBeDisabled();
    });
  });
});
