import {
  Play,
  Trash2,
  CheckCircle,
  Loader2,
  Shield,
  AlertTriangle,
  EyeOff,
  HelpCircle,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import { Checkbox } from "@/components/ui/checkbox";
import { SlashCommandsSection } from "./SlashCommandsSection";
import { cn } from "@/lib/utils";
import { featureManager, FEATURE_FLAGS } from "@/lib/featureManager";
import type { CommandModel, ExecutionLog } from "@/types/command";
import type {
  CommandFormData,
  TestOutput,
  AdapterInfo,
  SlashSyncStatus,
} from "@/hooks/useCommandsState";

interface CommandEditorProps {
  selected: CommandModel | null;
  form: CommandFormData;
  testOutput: TestOutput | null;
  commandHistory: readonly ExecutionLog[];
  historyFilter: string;
  historyPage: number;
  historyHasMore: boolean;
  isHistoryLoading: boolean;
  availableRepos: readonly string[];
  availableAdapters: readonly AdapterInfo[];
  slashStatus: Record<string, SlashSyncStatus>;
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
  onRepairSlashCommand: (adapter: string) => void;
  onHistoryFilterChange: (filter: string) => void;
  onHistoryPageChange: (page: number) => void;
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
  commandHistory,
  historyFilter,
  historyPage,
  historyHasMore,
  isHistoryLoading,
  availableRepos,
  availableAdapters,
  slashStatus,
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
  onRepairSlashCommand,
  onHistoryFilterChange,
  onHistoryPageChange,
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
          slashStatus={slashStatus}
          tierConfig={TIER_CONFIG}
          onToggleGenerate={(checked) => onUpdateForm({ generateSlashCommands: checked })}
          onToggleAdapter={onToggleAdapter}
          onRepairAdapter={onRepairSlashCommand}
        />

        {featureManager.isEnabled(FEATURE_FLAGS.EXECUTION_REDACTION) && (
          <div className="rounded-xl border border-white/5 bg-white/5 p-4">
            <div className="flex items-center gap-2 mb-3">
              <Shield className="h-4 w-4 text-primary/60" />
              <span className="text-sm font-semibold">Execution Policy</span>
            </div>
            <div className="grid gap-4 sm:grid-cols-2">
              <div className="grid gap-2">
                <label
                  htmlFor="timeout-ms"
                  className="text-xs text-muted-foreground flex items-center gap-1"
                >
                  Timeout (ms)
                  <span title="Maximum execution time before identifying as a failure. Default is 30 seconds.">
                    <HelpCircle className="h-3 w-3 opacity-50 cursor-help" />
                  </span>
                </label>
                <Input
                  id="timeout-ms"
                  type="number"
                  min={1000}
                  max={600000}
                  step={1000}
                  value={form.timeoutMs ?? ""}
                  onChange={(e) => {
                    const val = e.target.value ? parseInt(e.target.value, 10) : null;
                    if (val !== null) {
                      const clamped = Math.max(1000, Math.min(600000, val));
                      onUpdateForm({ timeoutMs: clamped });
                    } else {
                      onUpdateForm({ timeoutMs: null });
                    }
                  }}
                  placeholder="30000 (default)"
                />
              </div>
              <div className="grid gap-2">
                <label
                  htmlFor="max-retries"
                  className="text-xs text-muted-foreground flex items-center gap-1"
                >
                  Max Retries
                  <span title="Automatic retries for transient failures (timeouts, network errors). Maximum 3 retries.">
                    <HelpCircle className="h-3 w-3 opacity-50 cursor-help" />
                  </span>
                </label>
                <Input
                  id="max-retries"
                  type="number"
                  min={0}
                  max={3}
                  value={form.maxRetries ?? ""}
                  onChange={(e) => {
                    const val = e.target.value ? parseInt(e.target.value, 10) : null;
                    if (val !== null) {
                      const clamped = Math.max(0, Math.min(3, val));
                      onUpdateForm({ maxRetries: clamped });
                    } else {
                      onUpdateForm({ maxRetries: null });
                    }
                  }}
                  placeholder="0 (default)"
                />
              </div>
            </div>
            <p className="mt-2 text-xs text-muted-foreground">
              Retries apply to transient failures (timeouts, network errors). Maximum 3 retries.
            </p>
          </div>
        )}

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
          <div className="flex items-center justify-between mb-2">
            <span className="font-medium">Recent Execution History</span>
            <select
              value={historyFilter}
              onChange={(e) => onHistoryFilterChange(e.target.value)}
              className="text-xs bg-transparent border border-white/10 rounded px-2 py-1 text-muted-foreground focus:outline-none focus:ring-1 focus:ring-primary/40"
              aria-label="Filter execution history by result"
            >
              <option value="all">All</option>
              <option value="Success">Success</option>
              <option value="Timeout">Timeout</option>
              <option value="PermissionDenied">Permission Denied</option>
              <option value="MissingBinary">Missing Binary</option>
              <option value="NonZeroExit">Non-Zero Exit</option>
              <option value="ValidationError">Validation Error</option>
            </select>
          </div>
          <div className="space-y-2 max-h-56 overflow-auto">
            {isHistoryLoading && (
              <div className="flex items-center justify-center py-6">
                <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
              </div>
            )}
            {!isHistoryLoading &&
              commandHistory.map((h) => (
                <div key={h.id} className="rounded border p-2 hover:bg-white/5 transition-colors">
                  <div className="flex items-center justify-between text-xs">
                    <div className="flex items-center gap-2 flex-wrap">
                      <span
                        className={cn(
                          "font-medium px-1.5 py-0.5 rounded",
                          h.exitCode === 0
                            ? "text-green-400 bg-green-500/10"
                            : "text-red-400 bg-red-500/10"
                        )}
                      >
                        exit {h.exitCode}
                      </span>
                      {featureManager.isEnabled(FEATURE_FLAGS.EXECUTION_REDACTION) &&
                        h.failureClass &&
                        h.failureClass !== "Success" && (
                          <span
                            className={cn(
                              "inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] border",
                              h.failureClass === "Timeout" ||
                                h.failureClass === "PermissionDenied" ||
                                h.failureClass === "MissingBinary"
                                ? "bg-red-500/20 text-red-400 border-red-500/30"
                                : "bg-amber-500/20 text-amber-400 border-amber-500/30"
                            )}
                            title={`Failure: ${h.failureClass}`}
                          >
                            <AlertTriangle className="h-3 w-3" />
                            {h.failureClass}
                          </span>
                        )}
                      {featureManager.isEnabled(FEATURE_FLAGS.EXECUTION_REDACTION) &&
                        h.isRedacted && (
                          <span
                            className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] bg-slate-500/20 text-slate-400 border border-slate-500/30"
                            title="Output contained sensitive data that was redacted"
                          >
                            <EyeOff className="h-3 w-3" />
                            redacted
                          </span>
                        )}
                      {featureManager.isEnabled(FEATURE_FLAGS.EXECUTION_REDACTION) &&
                        (h.attemptNumber ?? 1) > 1 && (
                          <span className="text-[10px] text-muted-foreground bg-muted px-1.5 py-0.5 rounded">
                            attempt {h.attemptNumber}
                          </span>
                        )}
                      {h.adapterContext && (
                        <span className="text-[10px] text-muted-foreground/60">
                          via {h.adapterContext}
                        </span>
                      )}
                    </div>
                    <span className="text-muted-foreground font-mono">{h.durationMs}ms</span>
                  </div>
                  <div className="mt-1 truncate text-xs text-muted-foreground">
                    {h.stdout || h.stderr || "(no output)"}
                  </div>
                </div>
              ))}
            {!isHistoryLoading && commandHistory.length === 0 && (
              <div className="flex flex-col items-center justify-center py-6 text-center">
                <Play className="h-8 w-8 text-muted-foreground/40 mb-2" />
                <p className="text-xs text-muted-foreground">No executions yet.</p>
                <p className="text-[10px] text-muted-foreground/60 mt-1">
                  Click "Test Run" to execute this command.
                </p>
              </div>
            )}
          </div>
          <div className="flex items-center justify-between mt-2 pt-2 border-t border-white/5">
            <Button
              variant="outline"
              size="sm"
              disabled={historyPage === 0 || isHistoryLoading}
              onClick={() => onHistoryPageChange(historyPage - 1)}
            >
              Previous
            </Button>
            <span className="text-xs text-muted-foreground">Page {historyPage + 1}</span>
            <Button
              variant="outline"
              size="sm"
              disabled={!historyHasMore || isHistoryLoading}
              onClick={() => onHistoryPageChange(historyPage + 1)}
            >
              Next
            </Button>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
