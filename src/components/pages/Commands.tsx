import { useEffect, useMemo, useState } from "react";
import { Plus, Play, Trash2, Search } from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import { Badge } from "@/components/ui/badge";
import { api } from "@/lib/tauri";
import { useToast } from "@/components/ui/toast";
import { CommandsListSkeleton } from "@/components/ui/skeleton";
import type { CommandModel, ExecutionLog } from "@/types/command";

export function Commands() {
  const [commands, setCommands] = useState<CommandModel[]>([]);
  const [selectedId, setSelectedId] = useState<string>("");
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [script, setScript] = useState("");
  const [exposeViaMcp, setExposeViaMcp] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [isTesting, setIsTesting] = useState(false);
  const [testArgs, setTestArgs] = useState<Record<string, string>>({});
  const [testOutput, setTestOutput] = useState<{
    stdout: string;
    stderr: string;
    exitCode: number;
  } | null>(null);
  const [query, setQuery] = useState("");
  const [history, setHistory] = useState<ExecutionLog[]>([]);
  const [isSyncing, setIsSyncing] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [isSlashCommandSyncing, setIsSlashCommandSyncing] = useState(false);
  const [generateSlashCommands, setGenerateSlashCommands] = useState(false);
  const [slashCommandAdapters, setSlashCommandAdapters] = useState<string[]>([]);
  const [availableAdapters, setAvailableAdapters] = useState<
    { name: string; supports_argument_substitution: boolean }[]
  >([]);
  const { addToast } = useToast();

  const selected = useMemo(
    () => commands.find((cmd) => cmd.id === selectedId) ?? null,
    [commands, selectedId]
  );

  const filtered = useMemo(
    () =>
      commands.filter((cmd) => {
        const q = query.toLowerCase().trim();
        if (!q) return true;
        return cmd.name.toLowerCase().includes(q) || cmd.description.toLowerCase().includes(q);
      }),
    [commands, query]
  );

  const loadCommands = async () => {
    const result = await api.commands.getAll();
    setCommands(result);
  };

  const loadHistory = async () => {
    const logs = await api.execution.getHistory(50);
    setHistory(logs);
  };

  const loadAvailableAdapters = async () => {
    try {
      const adapters = await api.slashCommands.getAdapters();
      setAvailableAdapters(adapters);
    } catch (error) {
      console.error("Failed to load slash command adapters:", error);
    }
  };

  useEffect(() => {
    setIsLoading(true);
    Promise.all([loadCommands(), loadHistory(), loadAvailableAdapters()])
      .catch((error) => {
        addToast({
          title: "Failed to Load Commands",
          description: error instanceof Error ? error.message : "Unknown error",
          variant: "error",
        });
      })
      .finally(() => {
        setIsLoading(false);
      });
  }, [addToast]);

  useEffect(() => {
    if (!selected) {
      setName("");
      setDescription("");
      setScript("");
      setExposeViaMcp(true);
      setGenerateSlashCommands(false);
      setSlashCommandAdapters([]);
      return;
    }

    setName(selected.name);
    setDescription(selected.description);
    setScript(selected.script);
    setExposeViaMcp(Boolean(selected.expose_via_mcp));
    setGenerateSlashCommands(Boolean(selected.generate_slash_commands));
    setSlashCommandAdapters(selected.slash_command_adapters ?? []);
    const nextArgs: Record<string, string> = {};
    for (const arg of selected.arguments) {
      nextArgs[arg.name] = arg.default_value ?? "";
    }
    setTestArgs(nextArgs);
  }, [selected]);

  const handleCreate = async () => {
    setIsSaving(true);
    try {
      const created = await api.commands.create({
        name: "New Command",
        description: "Describe what this command does",
        script: "echo hello",
        arguments: [],
        expose_via_mcp: true,
      });
      await loadCommands();
      setSelectedId(created.id);
      addToast({ title: "Command Created", description: created.name, variant: "success" });
    } catch (error) {
      addToast({
        title: "Create Failed",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "error",
      });
    } finally {
      setIsSaving(false);
    }
  };

  const handleSave = async () => {
    if (!selected) return;
    setIsSaving(true);
    try {
      const updated = await api.commands.update(selected.id, {
        name,
        description,
        script,
        expose_via_mcp: exposeViaMcp,
        generate_slash_commands: generateSlashCommands,
        slash_command_adapters: slashCommandAdapters,
      });
      setCommands((prev) => prev.map((c) => (c.id === updated.id ? updated : c)));
      addToast({ title: "Command Saved", description: updated.name, variant: "success" });
    } catch (error) {
      addToast({
        title: "Save Failed",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "error",
      });
    } finally {
      setIsSaving(false);
    }
  };

  const handleDelete = async () => {
    if (!selected) return;
    setIsSaving(true);
    try {
      await api.commands.delete(selected.id);
      setCommands((prev) => prev.filter((c) => c.id !== selected.id));
      setSelectedId("");
      addToast({ title: "Command Deleted", description: selected.name, variant: "success" });
    } catch (error) {
      addToast({
        title: "Delete Failed",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "error",
      });
    } finally {
      setIsSaving(false);
    }
  };

  const handleTest = async () => {
    if (!selected) return;
    setIsTesting(true);
    try {
      const payload: Record<string, string> = {};
      for (const arg of selected.arguments) {
        payload[arg.name] = testArgs[arg.name] ?? "";
      }

      const result = await api.commands.test(selected.id, payload);
      setTestOutput({
        stdout: result.stdout,
        stderr: result.stderr,
        exitCode: result.exit_code,
      });
      await loadHistory();
      addToast({
        title: "Test Completed",
        description: result.success ? "Command succeeded" : "Command failed",
        variant: result.success ? "success" : "error",
      });
    } catch (error) {
      addToast({
        title: "Test Failed",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "error",
      });
    } finally {
      setIsTesting(false);
    }
  };

  const handleSyncCommands = async () => {
    setIsSyncing(true);
    try {
      const result = await api.commands.sync();
      if (!result.success) {
        throw new Error(result.errors[0]?.message ?? "Failed to sync commands");
      }
      addToast({
        title: "Commands Synced",
        description: `Wrote ${result.filesWritten.length} command files`,
        variant: "success",
      });
    } catch (error) {
      addToast({
        title: "Sync Failed",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "error",
      });
    } finally {
      setIsSyncing(false);
    }
  };

  const handleSyncSlashCommands = async () => {
    if (!selected) return;
    setIsSlashCommandSyncing(true);
    try {
      const result = await api.slashCommands.sync(selected.id, true);
      if (result.errors.length > 0) {
        throw new Error(result.errors[0]);
      }
      addToast({
        title: "Slash Commands Synced",
        description: `Wrote ${result.files_written} slash command files`,
        variant: "success",
      });
    } catch (error) {
      addToast({
        title: "Slash Command Sync Failed",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "error",
      });
    } finally {
      setIsSlashCommandSyncing(false);
    }
  };

  if (isLoading) {
    return <CommandsListSkeleton />;
  }

  return (
    <div className="grid gap-6 lg:grid-cols-[320px,1fr]">
      <Card className="glass-card premium-shadow border-none overflow-hidden">
        <CardHeader className="space-y-4 bg-white/5 pb-6">
          <div className="flex items-center justify-between">
            <CardTitle className="text-sm font-semibold tracking-wide uppercase text-muted-foreground/80">
              Commands
            </CardTitle>
            <Button
              size="sm"
              onClick={handleCreate}
              disabled={isSaving}
              className="glow-primary h-8"
            >
              <Plus className="mr-1.5 h-3.5 w-3.5" />
              New
            </Button>
          </div>
          <Button
            variant="outline"
            size="sm"
            onClick={handleSyncCommands}
            disabled={isSyncing}
            className="w-full glass border-white/5 hover:bg-white/5 text-xs"
          >
            {isSyncing ? "Syncing..." : "Sync Command Files"}
          </Button>
          <div className="relative">
            <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground/60" />
            <Input
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="Filter..."
              className="pl-9 h-9 bg-black/20 border-white/5 focus-visible:ring-primary/40 rounded-lg text-sm"
            />
          </div>
        </CardHeader>
        <CardContent className="space-y-2 pt-4 px-2">
          {filtered.map((cmd) => (
            <button
              key={cmd.id}
              className={cn(
                "w-full group relative overflow-hidden flex flex-col items-start rounded-xl px-4 py-3 text-left transition-all duration-300",
                selectedId === cmd.id
                  ? "bg-primary/10 border border-primary/20 premium-shadow"
                  : "hover:bg-white/5 border border-transparent hover:border-white/5"
              )}
              onClick={() => setSelectedId(cmd.id)}
            >
              <div className="flex w-full items-center justify-between gap-2">
                <div
                  className={cn(
                    "truncate font-semibold text-sm transition-colors",
                    selectedId === cmd.id
                      ? "text-primary"
                      : "text-foreground group-hover:text-primary/80"
                  )}
                >
                  {cmd.name}
                </div>
                {cmd.expose_via_mcp ? (
                  <Badge
                    variant="default"
                    className="h-4 text-[9px] px-1.5 uppercase font-bold tracking-tighter bg-primary/20 text-primary border-primary/20"
                  >
                    MCP
                  </Badge>
                ) : (
                  <Badge
                    variant="outline"
                    className="h-4 text-[9px] px-1.5 uppercase font-bold tracking-tighter border-white/10 text-muted-foreground/60"
                  >
                    Local
                  </Badge>
                )}
              </div>
              <div className="mt-1 truncate text-[11px] text-muted-foreground/60 group-hover:text-muted-foreground/80">
                {cmd.description}
              </div>
            </button>
          ))}
          {filtered.length === 0 && (
            <p className="text-xs text-muted-foreground/60 text-center py-8">No commands found.</p>
          )}
        </CardContent>
      </Card>

      <Card className="glass-card premium-shadow border-none overflow-hidden">
        <CardHeader className="bg-white/5 pb-4">
          <CardTitle className="text-sm font-semibold tracking-wide uppercase text-primary/80">
            {selected ? name : "Select a Command"}
          </CardTitle>
          <CardDescription>
            Define script-based commands and expose them to MCP clients.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-6 pt-6">
          {!selected ? (
            <p className="text-sm text-muted-foreground">
              Choose a command from the list or create a new one.
            </p>
          ) : (
            <>
              <div className="grid gap-2">
                <label htmlFor="command-name" className="text-sm font-medium">
                  Name
                </label>
                <Input
                  id="command-name"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  placeholder="Command name"
                />
              </div>

              <div className="grid gap-2">
                <label htmlFor="command-description" className="text-sm font-medium">
                  Description
                </label>
                <Input
                  id="command-description"
                  value={description}
                  onChange={(e) => setDescription(e.target.value)}
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
                  value={script}
                  onChange={(e) => setScript(e.target.value)}
                  className="min-h-48 rounded-xl border border-white/5 bg-black/40 p-4 text-[13px] font-mono shadow-inner focus:outline-none focus:ring-1 focus:ring-primary/40 leading-relaxed text-primary/90 selection:bg-primary/20"
                  placeholder="echo hello"
                />
              </div>

              <div className="flex items-center justify-between rounded-xl border border-white/5 bg-white/5 p-4 transition-colors hover:bg-white/10">
                <div>
                  <div className="font-semibold text-sm">Expose via MCP</div>
                  <div className="text-[10px] uppercase tracking-wider text-muted-foreground/60">
                    Enable this command in tools/list responses.
                  </div>
                </div>
                <Switch
                  checked={exposeViaMcp}
                  onCheckedChange={setExposeViaMcp}
                  aria-label="Expose command via MCP"
                />
              </div>

              {/* Slash Commands Section */}
              <div className="rounded-xl border border-white/5 bg-white/5 p-4 space-y-4">
                <div className="flex items-center justify-between">
                  <div>
                    <div className="font-semibold text-sm">Generate Slash Commands</div>
                    <div className="text-[10px] uppercase tracking-wider text-muted-foreground/60">
                      Create native /command triggers in AI tools.
                    </div>
                  </div>
                  <Switch
                    checked={generateSlashCommands}
                    onCheckedChange={setGenerateSlashCommands}
                    aria-label="Generate slash commands"
                  />
                </div>

                {generateSlashCommands && availableAdapters.length > 0 && (
                  <div className="space-y-2 pt-2 border-t border-white/5">
                    <div className="text-xs font-medium text-muted-foreground">Target AI Tools</div>
                    <div className="flex flex-wrap gap-2">
                      {availableAdapters.map((adapter) => (
                        <button
                          key={adapter.name}
                          onClick={() => {
                            setSlashCommandAdapters((prev) =>
                              prev.includes(adapter.name)
                                ? prev.filter((a) => a !== adapter.name)
                                : [...prev, adapter.name]
                            );
                          }}
                          className={cn(
                            "px-3 py-1.5 rounded-full text-xs font-medium transition-colors",
                            slashCommandAdapters.includes(adapter.name)
                              ? "bg-primary text-primary-foreground"
                              : "bg-white/5 text-muted-foreground hover:bg-white/10"
                          )}
                        >
                          {adapter.name}
                        </button>
                      ))}
                    </div>
                    {slashCommandAdapters.length > 0 && (
                      <div className="text-[10px] text-muted-foreground pt-1">
                        Selected: {slashCommandAdapters.join(", ")}
                      </div>
                    )}
                  </div>
                )}
              </div>

              {selected.arguments.length > 0 && (
                <div className="rounded-md border p-3 space-y-2">
                  <div className="text-sm font-medium">Test Arguments</div>
                  {selected.arguments.map((arg) => (
                    <div key={arg.name} className="grid gap-1">
                      <label className="text-xs text-muted-foreground">
                        {arg.name} {arg.required ? "(required)" : "(optional)"}
                      </label>
                      <Input
                        value={testArgs[arg.name] ?? ""}
                        onChange={(e) =>
                          setTestArgs((prev) => ({
                            ...prev,
                            [arg.name]: e.target.value,
                          }))
                        }
                        placeholder={arg.description || arg.name}
                      />
                    </div>
                  ))}
                </div>
              )}

              <div className="flex flex-wrap gap-2">
                <Button onClick={handleSave} disabled={isSaving}>
                  {isSaving ? "Saving..." : "Save"}
                </Button>
                <Button variant="outline" onClick={handleTest} disabled={isTesting}>
                  <Play className="mr-2 h-4 w-4" />
                  {isTesting ? "Running..." : "Test Run"}
                </Button>
                {generateSlashCommands && slashCommandAdapters.length > 0 && (
                  <Button
                    variant="outline"
                    onClick={handleSyncSlashCommands}
                    disabled={isSlashCommandSyncing}
                  >
                    {isSlashCommandSyncing ? "Syncing..." : "Sync Slash Commands"}
                  </Button>
                )}
                <Button variant="outline" onClick={handleDelete} disabled={isSaving}>
                  <Trash2 className="mr-2 h-4 w-4" />
                  Delete
                </Button>
              </div>

              {testOutput && (
                <div className="rounded-md border p-3 text-sm">
                  <div className="mb-2 font-medium">
                    Test Output (exit code: {testOutput.exitCode})
                  </div>
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
                  {history
                    .filter((h) => h.command_id === selected.id)
                    .slice(0, 10)
                    .map((h) => (
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
                  {history.filter((h) => h.command_id === selected.id).length === 0 && (
                    <p className="text-xs text-muted-foreground">No executions yet.</p>
                  )}
                </div>
              </div>
            </>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
