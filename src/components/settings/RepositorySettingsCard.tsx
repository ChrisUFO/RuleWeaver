import { FolderOpen } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";

interface RepositorySettingsCardProps {
  repositoryRoots: readonly string[];
  repoPathsDirty: boolean;
  isSavingRepos: boolean;
  isLoading: boolean;
  onAdd: () => Promise<void>;
  onRemove: (path: string) => Promise<void>;
  onSave: () => Promise<void>;
}

export function RepositorySettingsCard({
  repositoryRoots,
  repoPathsDirty,
  isSavingRepos,
  isLoading,
  onAdd,
  onRemove,
  onSave,
}: RepositorySettingsCardProps) {
  return (
    <Card className="glass-card premium-shadow border-none overflow-hidden">
      <CardHeader className="bg-white/5 pb-4">
        <CardTitle className="text-sm font-semibold tracking-wide uppercase text-muted-foreground/80">
          Repository Roots
        </CardTitle>
        <CardDescription>
          Configure repositories once, then select them across local artifacts and import workflows.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-3 pt-6">
        <div className="flex items-center gap-2">
          <Button variant="outline" onClick={onAdd}>
            <FolderOpen className="mr-2 h-4 w-4" /> Add Repository
          </Button>
          <Button onClick={onSave} disabled={!repoPathsDirty || isSavingRepos || isLoading}>
            {isSavingRepos ? "Saving..." : "Save Repositories"}
          </Button>
        </div>

        {repositoryRoots.length === 0 ? (
          <p className="text-sm text-muted-foreground">No repository roots configured yet.</p>
        ) : (
          <div className="space-y-2">
            {repositoryRoots.map((path) => (
              <div key={path} className="flex items-center justify-between rounded-md border p-2">
                <span className="text-xs break-all">{path}</span>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => void onRemove(path)}
                  aria-label={`Remove repository ${path}`}
                >
                  Remove
                </Button>
              </div>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  );
}
