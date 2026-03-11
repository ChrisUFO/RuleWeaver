import { useMemo, useState } from "react";
import { KeyRound, ShieldCheck } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Select } from "@/components/ui/select";
import type { ScopedSecret } from "@/types/secret";

interface ScopedSecretsCardProps {
  repositoryRoots: readonly string[];
  scopedSecrets: readonly ScopedSecret[];
  selectedWorkspace: string | null;
  isLoading: boolean;
  isSaving: boolean;
  onWorkspaceChange: (workspacePath: string | null) => void;
  onSaveGlobalSecret: (key: string, value: string) => Promise<void>;
  onSaveWorkspaceSecret: (key: string, value: string, workspacePath: string) => Promise<void>;
  onDeleteGlobalSecret: (key: string) => Promise<void>;
  onDeleteWorkspaceSecret: (key: string, workspacePath: string) => Promise<void>;
}

type EffectiveSecretRow = {
  key: string;
  source: "global" | "workspace";
};

export function ScopedSecretsCard({
  repositoryRoots,
  scopedSecrets,
  selectedWorkspace,
  isLoading,
  isSaving,
  onWorkspaceChange,
  onSaveGlobalSecret,
  onSaveWorkspaceSecret,
  onDeleteGlobalSecret,
  onDeleteWorkspaceSecret,
}: ScopedSecretsCardProps) {
  const [globalKey, setGlobalKey] = useState("");
  const [globalValue, setGlobalValue] = useState("");
  const [workspaceKey, setWorkspaceKey] = useState("");
  const [workspaceValue, setWorkspaceValue] = useState("");

  const globalSecrets = useMemo(
    () =>
      scopedSecrets
        .filter((secret) => secret.scope === "global")
        .sort((a, b) => a.key.localeCompare(b.key)),
    [scopedSecrets]
  );

  const workspaceSecrets = useMemo(
    () =>
      scopedSecrets
        .filter(
          (secret) => secret.scope === "workspace" && secret.workspacePath === selectedWorkspace
        )
        .sort((a, b) => a.key.localeCompare(b.key)),
    [scopedSecrets, selectedWorkspace]
  );

  const effectiveSecrets = useMemo<EffectiveSecretRow[]>(() => {
    const map = new Map<string, EffectiveSecretRow>();
    globalSecrets.forEach((secret) =>
      map.set(secret.key.toLowerCase(), { key: secret.key, source: "global" })
    );
    workspaceSecrets.forEach((secret) =>
      map.set(secret.key.toLowerCase(), { key: secret.key, source: "workspace" })
    );
    return [...map.values()].sort((a, b) => a.key.localeCompare(b.key));
  }, [globalSecrets, workspaceSecrets]);

  const workspaceOptions = repositoryRoots.map((path) => ({ value: path, label: path }));

  return (
    <Card className="glass-card premium-shadow border-none overflow-hidden">
      <CardHeader className="bg-white/5 pb-4">
        <CardTitle className="text-sm font-semibold tracking-wide uppercase text-muted-foreground/80">
          Scoped Secrets
        </CardTitle>
        <CardDescription>
          Define a global baseline, then override secrets per repository root without leaking raw
          values into docs or logs.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-6 pt-6">
        <div className="rounded-md border p-4">
          <div className="mb-3 flex items-center gap-2 text-sm font-medium">
            <ShieldCheck className="h-4 w-4" /> Workspace view
          </div>
          <Select
            aria-label="Secret workspace"
            options={[{ value: "", label: "Global baseline only" }, ...workspaceOptions]}
            value={selectedWorkspace ?? ""}
            onChange={(value) => onWorkspaceChange(value || null)}
            disabled={isLoading || repositoryRoots.length === 0}
          />
          <p className="mt-2 text-xs text-muted-foreground">
            {repositoryRoots.length === 0
              ? "Add repository roots first to enable workspace overrides."
              : selectedWorkspace
                ? `Viewing effective secrets for ${selectedWorkspace}`
                : "Viewing only the global secret baseline."}
          </p>
        </div>

        <div className="rounded-md border p-4">
          <div className="mb-3 flex items-center gap-2 text-sm font-medium">
            <KeyRound className="h-4 w-4" /> Global secret baseline
          </div>
          <div className="grid gap-2 md:grid-cols-[1fr_1fr_auto]">
            <Input
              placeholder="PROJECT_API_KEY"
              value={globalKey}
              onChange={(event) => setGlobalKey(event.target.value)}
              aria-label="Global secret key"
            />
            <Input
              type="password"
              placeholder="Secret value"
              value={globalValue}
              onChange={(event) => setGlobalValue(event.target.value)}
              aria-label="Global secret value"
            />
            <Button
              disabled={isSaving || !globalKey.trim() || !globalValue.trim()}
              onClick={async () => {
                await onSaveGlobalSecret(globalKey, globalValue);
                setGlobalKey("");
                setGlobalValue("");
              }}
            >
              Save Global
            </Button>
          </div>
          <div className="mt-4 space-y-2">
            {globalSecrets.length === 0 && (
              <p className="text-sm text-muted-foreground">No global secrets saved yet.</p>
            )}
            {globalSecrets.map((secret) => (
              <div
                key={secret.id}
                className="flex items-center justify-between rounded-md border p-2"
              >
                <div>
                  <div className="text-sm font-medium">{secret.key}</div>
                  <div className="text-xs text-muted-foreground">
                    Stored globally and inherited by default.
                  </div>
                </div>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => void onDeleteGlobalSecret(secret.key)}
                >
                  Delete
                </Button>
              </div>
            ))}
          </div>
        </div>

        <div className="rounded-md border p-4">
          <div className="mb-3 flex items-center justify-between gap-3">
            <div>
              <div className="text-sm font-medium">Workspace overrides</div>
              <div className="text-xs text-muted-foreground">
                Inherited secrets stay global until you override them here.
              </div>
            </div>
            {selectedWorkspace && (
              <Badge variant="outline">{workspaceSecrets.length} overrides</Badge>
            )}
          </div>

          {selectedWorkspace ? (
            <div className="space-y-4">
              <div className="grid gap-2 md:grid-cols-[1fr_1fr_auto]">
                <Input
                  placeholder="PROJECT_API_KEY"
                  value={workspaceKey}
                  onChange={(event) => setWorkspaceKey(event.target.value)}
                  aria-label="Workspace secret key"
                />
                <Input
                  type="password"
                  placeholder="Workspace override value"
                  value={workspaceValue}
                  onChange={(event) => setWorkspaceValue(event.target.value)}
                  aria-label="Workspace secret value"
                />
                <Button
                  disabled={isSaving || !workspaceKey.trim() || !workspaceValue.trim()}
                  onClick={async () => {
                    await onSaveWorkspaceSecret(workspaceKey, workspaceValue, selectedWorkspace);
                    setWorkspaceKey("");
                    setWorkspaceValue("");
                  }}
                >
                  Save Override
                </Button>
              </div>

              <div className="space-y-2">
                {effectiveSecrets.length === 0 && (
                  <p className="text-sm text-muted-foreground">
                    No effective secrets for this workspace yet.
                  </p>
                )}
                {effectiveSecrets.map((secret) => (
                  <div
                    key={`${selectedWorkspace}-${secret.key}`}
                    className="flex items-center justify-between rounded-md border p-2"
                  >
                    <div className="space-y-1">
                      <div className="text-sm font-medium">{secret.key}</div>
                      <Badge variant={secret.source === "workspace" ? "default" : "outline"}>
                        {secret.source === "workspace"
                          ? "Workspace override"
                          : "Inherited from global"}
                      </Badge>
                    </div>
                    {secret.source === "workspace" && (
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => void onDeleteWorkspaceSecret(secret.key, selectedWorkspace)}
                      >
                        Remove Override
                      </Button>
                    )}
                  </div>
                ))}
              </div>
            </div>
          ) : (
            <p className="text-sm text-muted-foreground">
              Select a repository root to manage workspace-specific overrides.
            </p>
          )}
        </div>
      </CardContent>
    </Card>
  );
}
