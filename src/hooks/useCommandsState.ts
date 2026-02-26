import { useState, useCallback, useEffect, useMemo } from "react";
import { api } from "@/lib/tauri";
import { toast } from "@/lib/toast-helpers";
import { togglePathInSet, filterByQuery } from "@/lib/collection-utils";
import type { useToast } from "@/components/ui/toast";
import type { CommandModel, ExecutionLog } from "@/types/command";

export interface AdapterInfo {
  name: string;
  supportsArgumentSubstitution: boolean;
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
  timeoutMs: number | null;
  maxRetries: number | null;
}

export interface TestOutput {
  stdout: string;
  stderr: string;
  exitCode: number;
}

export type SlashSyncStatus = "Synced" | "OutOfDate" | "NotSynced" | { Error: string };

const HISTORY_PAGE_SIZE = 10;

export interface UseCommandsStateReturn {
  commands: CommandModel[];
  selectedId: string;
  selected: CommandModel | null;
  form: CommandFormData;
  testOutput: TestOutput | null;
  history: ExecutionLog[];
  commandHistory: ExecutionLog[];
  historyFilter: string;
  historyPage: number;
  historyHasMore: boolean;
  isHistoryLoading: boolean;
  query: string;
  filtered: CommandModel[];
  availableAdapters: AdapterInfo[];
  slashStatus: Record<string, SlashSyncStatus>;
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
    handleRepairSlashCommand: (adapter: string) => Promise<void>;
    handleHistoryFilterChange: (filter: string) => void;
    handleHistoryPageChange: (page: number) => void;
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
  timeoutMs: null,
  maxRetries: null,
};

export function useCommandsState(
  addToast: ReturnType<typeof useToast>["addToast"],
  initialSelectedId?: string | null,
  onClearInitialId?: () => void
): UseCommandsStateReturn {
  const [commands, setCommands] = useState<CommandModel[]>([]);
  const [selectedId, setSelectedId] = useState<string>("");
  const [form, setForm] = useState<CommandFormData>(initialFormData);
  const [testOutput, setTestOutput] = useState<TestOutput | null>(null);
  const [history, setHistory] = useState<ExecutionLog[]>([]);
  const [commandHistory, setCommandHistory] = useState<ExecutionLog[]>([]);
  const [historyFilter, setHistoryFilter] = useState<string>("all");
  const [historyPage, setHistoryPage] = useState<number>(0);
  const [historyHasMore, setHistoryHasMore] = useState<boolean>(false);
  const [isHistoryLoading, setIsHistoryLoading] = useState<boolean>(false);
  const [query, setQuery] = useState("");
  const [availableAdapters, setAvailableAdapters] = useState<AdapterInfo[]>([]);
  const [slashStatus, setSlashStatus] = useState<Record<string, SlashSyncStatus>>({});
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [isTesting, setIsTesting] = useState(false);
  const [isSyncing, setIsSyncing] = useState(false);
  const [isSlashCommandSyncing, setIsSlashCommandSyncing] = useState(false);

  const selected = useMemo(
    () => commands.find((cmd) => cmd.id === selectedId) ?? null,
    [commands, selectedId]
  );

  useEffect(() => {
    if (initialSelectedId && commands.length > 0) {
      const exists = commands.some((c) => c.id === initialSelectedId);
      if (exists) {
        setSelectedId(initialSelectedId);
        onClearInitialId?.();
      }
    }
  }, [initialSelectedId, commands, onClearInitialId]);

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

  const loadFilteredHistory = useCallback(
    async (commandId: string, filter: string, page: number) => {
      if (!commandId) return;
      setIsHistoryLoading(true);
      try {
        const failureClass = filter !== "all" ? filter : undefined;
        const logs = await api.execution.getHistoryFiltered(
          commandId,
          failureClass,
          HISTORY_PAGE_SIZE,
          page * HISTORY_PAGE_SIZE
        );
        setCommandHistory(logs);
        setHistoryHasMore(logs.length === HISTORY_PAGE_SIZE);
      } catch (error) {
        console.error("Failed to load filtered history", { error });
        setCommandHistory([]);
        setHistoryHasMore(false);
      } finally {
        setIsHistoryLoading(false);
      }
    },
    []
  );

  const loadAvailableAdapters = useCallback(async () => {
    try {
      const adapters = await api.slashCommands.getAdapters();
      setAvailableAdapters(adapters);
    } catch (error) {
      console.error("Failed to load slash command adapters", { error });
      setAvailableAdapters([]);
    }
  }, []);

  const loadSlashStatus = useCallback(async (commandId: string) => {
    try {
      const status = await api.slashCommands.getStatus(commandId);
      setSlashStatus(status);
    } catch {
      setSlashStatus({});
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
      setSlashStatus({});
      setCommandHistory([]);
      setHistoryFilter("all");
      setHistoryPage(0);
      setHistoryHasMore(false);
      return;
    }
    const nextArgs: Record<string, string> = {};
    for (const arg of selected.arguments) {
      nextArgs[arg.name] = arg.defaultValue ?? "";
    }
    setForm({
      name: selected.name,
      description: selected.description,
      script: selected.script,
      exposeViaMcp: Boolean(selected.exposeViaMcp),
      generateSlashCommands: Boolean(selected.generateSlashCommands),
      slashCommandAdapters: selected.slashCommandAdapters ?? [],
      targetPaths: selected.targetPaths ?? [],
      testArgs: nextArgs,
      timeoutMs: selected.timeoutMs ?? null,
      maxRetries: selected.maxRetries ?? null,
    });
    if (selected.generateSlashCommands && (selected.slashCommandAdapters ?? []).length > 0) {
      loadSlashStatus(selected.id);
    } else {
      setSlashStatus({});
    }
    setHistoryFilter("all");
    setHistoryPage(0);
    loadFilteredHistory(selected.id, "all", 0);
  }, [selected, loadSlashStatus, loadFilteredHistory]);

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
        isPlaceholder: false,
        arguments: [],
        exposeViaMcp: true,
        targetPaths: [],
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
        exposeViaMcp: form.exposeViaMcp,
        generateSlashCommands: form.generateSlashCommands,
        slashCommandAdapters: form.slashCommandAdapters,
        targetPaths: form.targetPaths,
        timeoutMs: form.timeoutMs ?? undefined,
        maxRetries: form.maxRetries ?? undefined,
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
        exitCode: result.exitCode,
      });
      await loadHistory();
      await loadFilteredHistory(selected.id, historyFilter, historyPage);
      toast[`${result.success ? "success" : "error"}`](addToast, {
        title: "Test Completed",
        description: result.success ? "Command succeeded" : "Command failed",
      });
    } catch (error) {
      toast.error(addToast, { title: "Test Failed", error });
    } finally {
      setIsTesting(false);
    }
  }, [
    selected,
    form.testArgs,
    loadHistory,
    loadFilteredHistory,
    historyFilter,
    historyPage,
    addToast,
  ]);

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
          ...result.conflicts.map((c) => `${c.adapterName}: ${c.message}`),
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
          result.filesWritten > 0
            ? `Successfully wrote ${result.filesWritten} file${result.filesWritten > 1 ? "s" : ""} `
            : "All files were already up to date",
      });
      await loadSlashStatus(selected.id);
    } catch (error) {
      toast.error(addToast, { title: "Slash Command Sync Failed", error });
    } finally {
      setIsSlashCommandSyncing(false);
    }
  }, [selected, addToast, loadSlashStatus]);

  const handleRepairSlashCommand = useCallback(
    async (adapter: string) => {
      if (!selected) return;
      setIsSlashCommandSyncing(true);
      try {
        const result = await api.slashCommands.sync(selected.id, true);
        if (result.errors.length > 0) {
          toast.error(addToast, {
            title: "Repair Failed",
            description: result.errors[0],
          });
        } else {
          toast.success(addToast, {
            title: "Repaired",
            description: `${adapter} slash command file updated`,
          });
          await loadSlashStatus(selected.id);
        }
      } catch (error) {
        toast.error(addToast, { title: "Repair Failed", error });
      } finally {
        setIsSlashCommandSyncing(false);
      }
    },
    [selected, addToast, loadSlashStatus]
  );

  const handleHistoryFilterChange = useCallback(
    (filter: string) => {
      if (!selected) return;
      setHistoryFilter(filter);
      setHistoryPage(0);
      loadFilteredHistory(selected.id, filter, 0);
    },
    [selected, loadFilteredHistory]
  );

  const handleHistoryPageChange = useCallback(
    (page: number) => {
      if (!selected) return;
      setHistoryPage(page);
      loadFilteredHistory(selected.id, historyFilter, page);
    },
    [selected, historyFilter, loadFilteredHistory]
  );

  return {
    commands,
    selectedId,
    selected,
    form,
    testOutput,
    history,
    commandHistory,
    historyFilter,
    historyPage,
    historyHasMore,
    isHistoryLoading,
    query,
    filtered,
    availableAdapters,
    slashStatus,
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
      handleRepairSlashCommand,
      handleHistoryFilterChange,
      handleHistoryPageChange,
      refresh,
    },
  };
}
