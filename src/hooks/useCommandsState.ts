import { useState, useCallback, useEffect, useMemo } from "react";
import { api } from "@/lib/tauri";
import { toast } from "@/lib/toast-helpers";
import { togglePathInSet, filterByQuery } from "@/lib/collection-utils";
import type { useToast } from "@/components/ui/toast";
import type { CommandModel, ExecutionLog } from "@/types/command";

export interface AdapterInfo {
  name: string;
  supports_argument_substitution: boolean;
}

export interface CommandFormData {
  name: string;
  description: string;
  script: string;
  exposeViaMcp: boolean;
  generateSlashCommands: boolean;
  slashCommandAdapters: string[];
  targetPaths: string[];
  testArgs: Record<string, string>;
}

export interface TestOutput {
  stdout: string;
  stderr: string;
  exitCode: number;
}

export interface UseCommandsStateReturn {
  commands: CommandModel[];
  selectedId: string;
  selected: CommandModel | null;
  form: CommandFormData;
  testOutput: TestOutput | null;
  history: ExecutionLog[];
  query: string;
  filtered: CommandModel[];
  availableAdapters: AdapterInfo[];
  isLoading: boolean;
  isSaving: boolean;
  isTesting: boolean;
  isSyncing: boolean;
  isSlashCommandSyncing: boolean;
  handlers: {
    setSelectedId: (id: string) => void;
    setQuery: (q: string) => void;
    updateForm: (updates: Partial<CommandFormData>) => void;
    toggleTargetPath: (path: string, checked: boolean) => void;
    toggleSlashCommandAdapter: (adapter: string) => void;
    handleCreate: () => Promise<void>;
    handleSave: () => Promise<void>;
    handleDelete: () => Promise<void>;
    handleTest: () => Promise<void>;
    handleSyncCommands: () => Promise<void>;
    handleSyncSlashCommands: () => Promise<void>;
    refresh: () => Promise<void>;
  };
}

const initialFormData: CommandFormData = {
  name: "",
  description: "",
  script: "",
  exposeViaMcp: true,
  generateSlashCommands: false,
  slashCommandAdapters: [],
  targetPaths: [],
  testArgs: {},
};

export function useCommandsState(
  addToast: ReturnType<typeof useToast>["addToast"]
): UseCommandsStateReturn {
  const [commands, setCommands] = useState<CommandModel[]>([]);
  const [selectedId, setSelectedId] = useState<string>("");
  const [form, setForm] = useState<CommandFormData>(initialFormData);
  const [testOutput, setTestOutput] = useState<TestOutput | null>(null);
  const [history, setHistory] = useState<ExecutionLog[]>([]);
  const [query, setQuery] = useState("");
  const [availableAdapters, setAvailableAdapters] = useState<AdapterInfo[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [isTesting, setIsTesting] = useState(false);
  const [isSyncing, setIsSyncing] = useState(false);
  const [isSlashCommandSyncing, setIsSlashCommandSyncing] = useState(false);

  const selected = useMemo(
    () => commands.find((cmd) => cmd.id === selectedId) ?? null,
    [commands, selectedId]
  );

  const filtered = useMemo(
    () => filterByQuery(commands, query, ["name", "description"] as const),
    [commands, query]
  );

  const loadCommands = useCallback(async () => {
    const result = await api.commands.getAll();
    setCommands(result);
  }, []);

  const loadHistory = useCallback(async () => {
    const logs = await api.execution.getHistory(50);
    setHistory(logs);
  }, []);

  const loadAvailableAdapters = useCallback(async () => {
    try {
      const adapters = await api.slashCommands.getAdapters();
      setAvailableAdapters(adapters);
    } catch (error) {
      console.error("Failed to load slash command adapters", { error });
      setAvailableAdapters([]);
    }
  }, []);

  const refresh = useCallback(async () => {
    setIsLoading(true);
    try {
      await Promise.all([loadCommands(), loadHistory(), loadAvailableAdapters()]);
    } catch (error) {
      toast.error(addToast, { title: "Failed to Load Commands", error });
    } finally {
      setIsLoading(false);
    }
  }, [loadCommands, loadHistory, loadAvailableAdapters, addToast]);

  useEffect(() => {
    refresh();
  }, [refresh]);

  useEffect(() => {
    if (!selected) {
      setForm(initialFormData);
      return;
    }
    const nextArgs: Record<string, string> = {};
    for (const arg of selected.arguments) {
      nextArgs[arg.name] = arg.default_value ?? "";
    }
    setForm({
      name: selected.name,
      description: selected.description,
      script: selected.script,
      exposeViaMcp: Boolean(selected.expose_via_mcp),
      generateSlashCommands: Boolean(selected.generate_slash_commands),
      slashCommandAdapters: selected.slash_command_adapters ?? [],
      targetPaths: selected.target_paths ?? [],
      testArgs: nextArgs,
    });
  }, [selected]);

  const updateForm = useCallback((updates: Partial<CommandFormData>) => {
    setForm((prev) => ({ ...prev, ...updates }));
  }, []);

  const toggleTargetPath = useCallback((path: string, checked: boolean) => {
    setForm((prev) => ({
      ...prev,
      targetPaths: togglePathInSet(prev.targetPaths, path, checked),
    }));
  }, []);

  const toggleSlashCommandAdapter = useCallback((adapter: string) => {
    setForm((prev) => ({
      ...prev,
      slashCommandAdapters: prev.slashCommandAdapters.includes(adapter)
        ? prev.slashCommandAdapters.filter((a) => a !== adapter)
        : [...prev.slashCommandAdapters, adapter],
    }));
  }, []);

  const handleCreate = useCallback(async () => {
    setIsSaving(true);
    try {
      const created = await api.commands.create({
        name: "New Command",
        description: "Describe what this command does",
        script: "echo hello",
        arguments: [],
        expose_via_mcp: true,
        target_paths: [],
      });
      await loadCommands();
      setSelectedId(created.id);
      toast.success(addToast, { title: "Command Created", description: created.name });
    } catch (error) {
      toast.error(addToast, { title: "Create Failed", error });
    } finally {
      setIsSaving(false);
    }
  }, [loadCommands, addToast]);

  const handleSave = useCallback(async () => {
    if (!selected) return;
    setIsSaving(true);
    try {
      const updated = await api.commands.update(selected.id, {
        name: form.name,
        description: form.description,
        script: form.script,
        expose_via_mcp: form.exposeViaMcp,
        generate_slash_commands: form.generateSlashCommands,
        slash_command_adapters: form.slashCommandAdapters,
        target_paths: form.targetPaths,
      });
      setCommands((prev) => prev.map((c) => (c.id === updated.id ? updated : c)));
      toast.success(addToast, { title: "Command Saved", description: updated.name });
    } catch (error) {
      toast.error(addToast, { title: "Save Failed", error });
    } finally {
      setIsSaving(false);
    }
  }, [selected, form, addToast]);

  const handleDelete = useCallback(async () => {
    if (!selected) return;
    setIsSaving(true);
    try {
      await api.commands.delete(selected.id);
      setCommands((prev) => prev.filter((c) => c.id !== selected.id));
      setSelectedId("");
      toast.success(addToast, { title: "Command Deleted", description: selected.name });
    } catch (error) {
      toast.error(addToast, { title: "Delete Failed", error });
    } finally {
      setIsSaving(false);
    }
  }, [selected, addToast]);

  const handleTest = useCallback(async () => {
    if (!selected) return;
    setIsTesting(true);
    try {
      const payload: Record<string, string> = {};
      for (const arg of selected.arguments) {
        payload[arg.name] = form.testArgs[arg.name] ?? "";
      }
      const result = await api.commands.test(selected.id, payload);
      setTestOutput({
        stdout: result.stdout,
        stderr: result.stderr,
        exitCode: result.exit_code,
      });
      await loadHistory();
      toast[`${result.success ? "success" : "error"}`](addToast, {
        title: "Test Completed",
        description: result.success ? "Command succeeded" : "Command failed",
      });
    } catch (error) {
      toast.error(addToast, { title: "Test Failed", error });
    } finally {
      setIsTesting(false);
    }
  }, [selected, form.testArgs, loadHistory, addToast]);

  const handleSyncCommands = useCallback(async () => {
    setIsSyncing(true);
    try {
      const result = await api.commands.sync();
      if (!result.success) {
        throw new Error(result.errors[0]?.message ?? "Failed to sync commands");
      }
      toast.success(addToast, {
        title: "Commands Synced",
        description: `Wrote ${result.filesWritten.length} command files`,
      });
    } catch (error) {
      toast.error(addToast, { title: "Sync Failed", error });
    } finally {
      setIsSyncing(false);
    }
  }, [addToast]);

  const handleSyncSlashCommands = useCallback(async () => {
    if (!selected) return;
    setIsSlashCommandSyncing(true);
    try {
      const result = await api.slashCommands.sync(selected.id, true);
      if (result.errors.length > 0 || result.conflicts.length > 0) {
        const errorMessages = [
          ...result.errors,
          ...result.conflicts.map((c) => `${c.adapter_name}: ${c.message}`),
        ];
        toast.error(addToast, {
          title: `Sync completed with ${errorMessages.length} issue${errorMessages.length > 1 ? "s" : ""}`,
          description:
            errorMessages.slice(0, 3).join("\n") +
            (errorMessages.length > 3 ? `\n...and ${errorMessages.length - 3} more` : ""),
        });
        return;
      }
      toast.success(addToast, {
        title: "Slash Commands Synced",
        description:
          result.files_written > 0
            ? `Successfully wrote ${result.files_written} file${result.files_written > 1 ? "s" : ""}`
            : "All files were already up to date",
      });
    } catch (error) {
      toast.error(addToast, { title: "Slash Command Sync Failed", error });
    } finally {
      setIsSlashCommandSyncing(false);
    }
  }, [selected, addToast]);

  return {
    commands,
    selectedId,
    selected,
    form,
    testOutput,
    history,
    query,
    filtered,
    availableAdapters,
    isLoading,
    isSaving,
    isTesting,
    isSyncing,
    isSlashCommandSyncing,
    handlers: {
      setSelectedId,
      setQuery,
      updateForm,
      toggleTargetPath,
      toggleSlashCommandAdapter,
      handleCreate,
      handleSave,
      handleDelete,
      handleTest,
      handleSyncCommands,
      handleSyncSlashCommands,
      refresh,
    },
  };
}
