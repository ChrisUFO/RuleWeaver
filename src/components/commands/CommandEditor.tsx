import { Play, Trash2, CheckCircle, Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import { Checkbox } from "@/components/ui/checkbox";
import { SlashCommandsSection } from "./SlashCommandsSection";
import type { CommandModel, ExecutionLog } from "@/types/command";
import type { CommandFormData, TestOutput, AdapterInfo } from "@/hooks/useCommandsState";

interface CommandEditorProps {
  selected: CommandModel | null;
  form: CommandFormData;
  testOutput: TestOutput | null;
  history: readonly ExecutionLog[];
  availableRepos: readonly string[];
  availableAdapters: readonly AdapterInfo[];
  isSaving: boolean;
  isTesting: boolean;
  isSlashCommandSyncing: boolean;
  onUpdateForm: (updates: Partial<CommandFormData>) => void;
  onToggleTargetPath: (path: string, checked: boolean) => void;
  onToggleAdapter: (adapter: string) => void;
  onSave: () => void;
  onDelete: () => void;
  onTest: () => void;
  onSyncSlashCommands: () => void;
}

const TIER_CONFIG: Record<string, { tier: number; label: string; color: string }> = {
  opencode: {
    tier: 1,
    label: "Stable",
    color: "bg-green-500/20 text-green-400 border-green-500/30",
  },
  "claude-code": {
    tier: 1,
    label: "Stable",
    color: "bg-green-500/20 text-green-400 border-green-500/30",
  },
  cline: { tier: 1, label: "Stable", color: "bg-green-500/20 text-green-400 border-green-500/30" },
  gemini: { tier: 1, label: "Stable", color: "bg-green-500/20 text-green-400 border-green-500/30" },
  cursor: { tier: 1, label: "Stable", color: "bg-green-500/20 text-green-400 border-green-500/30" },
  roo: { tier: 1, label: "Stable", color: "bg-green-500/20 text-green-400 border-green-500/30" },
  antigravity: {
    tier: 1,
    label: "Stable",
    color: "bg-green-500/20 text-green-400 border-green-500/30",
  },
  codex: { tier: 1, label: "Stable", color: "bg-green-500/20 text-green-400 border-green-500/30" },
};

export function CommandEditor({
  selected,
  form,
  testOutput,
  history,
  availableRepos,
  availableAdapters,
  isSaving,
  isTesting,
  isSlashCommandSyncing,
  onUpdateForm,
  onToggleTargetPath,
  onToggleAdapter,
  onSave,
  onDelete,
  onTest,
  onSyncSlashCommands,
}: CommandEditorProps) {
  if (!selected) {
    return (
      <Card className="glass-card premium-shadow border-none overflow-hidden">
        <CardHeader className="bg-white/5 pb-4">
          <CardTitle className="text-sm font-semibold tracking-wide uppercase text-primary/80">
            Select a Command
          </CardTitle>
          <CardDescription>
            Define script-based commands and expose them to MCP clients.
          </CardDescription>
        </CardHeader>
        <CardContent className="pt-6">
          <p className="text-sm text-muted-foreground">
            Choose a command from the list or create a new one.
          </p>
        </CardContent>
      </Card>
    );
  }

  const commandHistory = history.filter((h) => h.command_id === selected.id);

  return (
    <Card className="glass-card premium-shadow border-none overflow-hidden">
      <CardHeader className="bg-white/5 pb-4">
        <CardTitle className="text-sm font-semibold tracking-wide uppercase text-primary/80">
          {form.name || "Select a Command"}
        </CardTitle>
        <CardDescription>
          Define script-based commands and expose them to MCP clients.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-6 pt-6">
        <div className="grid gap-2">
          <label htmlFor="command-name" className="text-sm font-medium">
            Name
          </label>
          <Input
            id="command-name"
            value={form.name}
            onChange={(e) => onUpdateForm({ name: e.target.value })}
            placeholder="Command name"
          />
        </div>

        <div className="grid gap-2">
          <label htmlFor="command-description" className="text-sm font-medium">
            Description
          </label>
          <Input
            id="command-description"
            value={form.description}
            onChange={(e) => onUpdateForm({ description: e.target.value })}
            placeholder="What this command does"
          />
        </div>

        <div className="grid gap-2">
          <label
            htmlFor="command-script"
            className="text-xs font-bold uppercase tracking-widest text-muted-foreground/60"
          >
            Script
          </label>
          <textarea
            id="command-script"
            value={form.script}
            onChange={(e) => onUpdateForm({ script: e.target.value })}
            className="min-h-48 rounded-xl border border-white/5 bg-black/40 p-4 text-[13px] font-mono shadow-inner focus:outline-none focus:ring-1 focus:ring-primary/40 leading-relaxed text-primary/90 selection:bg-primary/20"
            placeholder="echo hello"
          />
        </div>

        <div className="grid gap-2">
          <label className="text-sm font-medium">Target Repositories (Optional)</label>
          {availableRepos.length === 0 ? (
            <p className="text-xs text-muted-foreground">
              No repositories configured. Add repository roots in Settings.
            </p>
          ) : (
            <div className="rounded-md border p-3 space-y-2">
              {availableRepos.map((repo) => (
                <label key={repo} className="flex items-center gap-2 text-sm">
                  <Checkbox
                    checked={form.targetPaths.includes(repo)}
                    onChange={(checked) => onToggleTargetPath(repo, checked)}
                    aria-label={`Target repository ${repo}`}
                  />
                  <span className="truncate">{repo}</span>
                </label>
              ))}
            </div>
          )}
        </div>

        <div className="flex items-center justify-between rounded-xl border border-white/5 bg-white/5 p-4 transition-colors hover:bg-white/10">
          <div>
            <div className="font-semibold text-sm">Expose via MCP</div>
            <div className="text-[10px] uppercase tracking-wider text-muted-foreground/60">
              Enable this command in tools/list responses.
            </div>
          </div>
          <Switch
            checked={form.exposeViaMcp}
            onCheckedChange={(checked) => onUpdateForm({ exposeViaMcp: checked })}
            aria-label="Expose command via MCP"
          />
        </div>

        <SlashCommandsSection
          generateSlashCommands={form.generateSlashCommands}
          slashCommandAdapters={form.slashCommandAdapters}
          availableAdapters={availableAdapters}
          tierConfig={TIER_CONFIG}
          onToggleGenerate={(checked) => onUpdateForm({ generateSlashCommands: checked })}
          onToggleAdapter={onToggleAdapter}
        />

        {selected.arguments.length > 0 && (
          <div className="rounded-md border p-3 space-y-2">
            <div className="text-sm font-medium">Test Arguments</div>
            {selected.arguments.map((arg) => (
              <div key={arg.name} className="grid gap-1">
                <label className="text-xs text-muted-foreground">
                  {arg.name} {arg.required ? "(required)" : "(optional)"}
                </label>
                <Input
                  value={form.testArgs[arg.name] ?? ""}
                  onChange={(e) =>
                    onUpdateForm({
                      testArgs: { ...form.testArgs, [arg.name]: e.target.value },
                    })
                  }
                  placeholder={arg.description || arg.name}
                />
              </div>
            ))}
          </div>
        )}

        <div className="flex flex-wrap gap-2">
          <Button onClick={onSave} disabled={isSaving}>
            {isSaving ? "Saving..." : "Save"}
          </Button>
          <Button variant="outline" onClick={onTest} disabled={isTesting}>
            <Play className="mr-2 h-4 w-4" />
            {isTesting ? "Running..." : "Test Run"}
          </Button>
          {form.generateSlashCommands && form.slashCommandAdapters.length > 0 && (
            <Button
              variant="outline"
              onClick={onSyncSlashCommands}
              disabled={isSlashCommandSyncing || isSaving}
            >
              {isSlashCommandSyncing ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Syncing...
                </>
              ) : (
                <>
                  <CheckCircle className="mr-2 h-4 w-4" />
                  Sync Slash Commands
                </>
              )}
            </Button>
          )}
          <Button variant="outline" onClick={onDelete} disabled={isSaving}>
            <Trash2 className="mr-2 h-4 w-4" />
            Delete
          </Button>
        </div>

        {testOutput && (
          <div className="rounded-md border p-3 text-sm">
            <div className="mb-2 font-medium">Test Output (exit code: {testOutput.exitCode})</div>
            <div className="grid gap-2 md:grid-cols-2">
              <div>
                <div className="mb-1 text-xs text-muted-foreground">stdout</div>
                <pre className="max-h-48 overflow-auto rounded bg-muted p-2 text-xs whitespace-pre-wrap">
                  {testOutput.stdout || "(empty)"}
                </pre>
              </div>
              <div>
                <div className="mb-1 text-xs text-muted-foreground">stderr</div>
                <pre className="max-h-48 overflow-auto rounded bg-muted p-2 text-xs whitespace-pre-wrap">
                  {testOutput.stderr || "(empty)"}
                </pre>
              </div>
            </div>
          </div>
        )}

        <div className="rounded-md border p-3 text-sm">
          <div className="mb-2 font-medium">Recent Execution History</div>
          <div className="space-y-2 max-h-56 overflow-auto">
            {commandHistory.slice(0, 10).map((h) => (
              <div key={h.id} className="rounded border p-2">
                <div className="flex items-center justify-between text-xs">
                  <span className="font-medium">exit {h.exit_code}</span>
                  <span className="text-muted-foreground">{h.duration_ms}ms</span>
                </div>
                <div className="mt-1 truncate text-xs text-muted-foreground">
                  {h.stdout || h.stderr || "(no output)"}
                </div>
              </div>
            ))}
            {commandHistory.length === 0 && (
              <p className="text-xs text-muted-foreground">No executions yet.</p>
            )}
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
