import { useEffect, useMemo, useState } from "react";
import { Plus, Play, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import { Badge } from "@/components/ui/badge";
import { api } from "@/lib/tauri";
import { useToast } from "@/components/ui/toast";
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

  useEffect(() => {
    Promise.all([loadCommands(), loadHistory()]).catch((error) => {
      addToast({
        title: "Failed to Load Commands",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "error",
      });
    });
  }, [addToast]);

  useEffect(() => {
    if (!selected) {
      setName("");
      setDescription("");
      setScript("");
      setExposeViaMcp(true);
      return;
    }

    setName(selected.name);
    setDescription(selected.description);
    setScript(selected.script);
    setExposeViaMcp(Boolean(selected.expose_via_mcp));
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

  return (
    <div className="grid gap-6 lg:grid-cols-[320px,1fr]">
      <Card>
        <CardHeader className="space-y-3">
          <div className="flex items-center justify-between">
            <CardTitle>Commands</CardTitle>
            <Button size="sm" onClick={handleCreate} disabled={isSaving}>
              <Plus className="mr-2 h-4 w-4" />
              New
            </Button>
          </div>
          <Button variant="outline" size="sm" onClick={handleSyncCommands} disabled={isSyncing}>
            {isSyncing ? "Syncing..." : "Sync Command Files"}
          </Button>
          <Input
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Search commands"
          />
        </CardHeader>
        <CardContent className="space-y-2">
          {filtered.map((cmd) => (
            <button
              key={cmd.id}
              className={`w-full rounded-md border px-3 py-2 text-left transition ${
                selectedId === cmd.id ? "border-primary bg-accent" : "hover:bg-accent"
              }`}
              onClick={() => setSelectedId(cmd.id)}
            >
              <div className="flex items-center justify-between gap-2">
                <div className="truncate font-medium">{cmd.name}</div>
                {cmd.expose_via_mcp ? <Badge>MCP</Badge> : <Badge variant="outline">Local</Badge>}
              </div>
              <div className="mt-1 truncate text-xs text-muted-foreground">{cmd.description}</div>
            </button>
          ))}
          {filtered.length === 0 && (
            <p className="text-sm text-muted-foreground">No commands found.</p>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>{selected ? "Edit Command" : "Select a Command"}</CardTitle>
          <CardDescription>
            Define script-based commands and expose them to MCP clients.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          {!selected ? (
            <p className="text-sm text-muted-foreground">
              Choose a command from the list or create a new one.
            </p>
          ) : (
            <>
              <div className="grid gap-2">
                <label className="text-sm font-medium">Name</label>
                <Input
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  placeholder="Command name"
                />
              </div>

              <div className="grid gap-2">
                <label className="text-sm font-medium">Description</label>
                <Input
                  value={description}
                  onChange={(e) => setDescription(e.target.value)}
                  placeholder="What this command does"
                />
              </div>

              <div className="grid gap-2">
                <label className="text-sm font-medium">Script</label>
                <textarea
                  value={script}
                  onChange={(e) => setScript(e.target.value)}
                  className="min-h-36 rounded-md border bg-background p-3 text-sm"
                  placeholder="echo hello"
                />
              </div>

              <div className="flex items-center justify-between rounded-md border p-3">
                <div>
                  <div className="font-medium">Expose via MCP</div>
                  <div className="text-xs text-muted-foreground">
                    Enable this command in tools/list responses.
                  </div>
                </div>
                <Switch checked={exposeViaMcp} onCheckedChange={setExposeViaMcp} />
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
