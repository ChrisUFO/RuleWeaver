import { useToast } from "@/components/ui/toast";
import { CommandsListSkeleton } from "@/components/ui/skeleton";
import { useRepositoryRoots } from "@/hooks/useRepositoryRoots";
import { useCommandsState } from "@/hooks/useCommandsState";
import { CommandList } from "@/components/commands/CommandList";
import { CommandEditor } from "@/components/commands/CommandEditor";

export function Commands() {
  const { addToast } = useToast();
  const { roots: availableRepos } = useRepositoryRoots();
  const {
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
    handlers,
  } = useCommandsState(addToast);

  if (isLoading) {
    return <CommandsListSkeleton />;
  }

  return (
    <div className="grid gap-6 lg:grid-cols-[320px,1fr] max-w-7xl mx-auto">
      <CommandList
        commands={filtered}
        selectedId={selectedId}
        query={query}
        isSaving={isSaving}
        isSyncing={isSyncing}
        onSelect={handlers.setSelectedId}
        onQueryChange={handlers.setQuery}
        onCreate={handlers.handleCreate}
        onSync={handlers.handleSyncCommands}
      />

      <CommandEditor
        selected={selected}
        form={form}
        testOutput={testOutput}
        history={history}
        availableRepos={availableRepos}
        availableAdapters={availableAdapters}
        isSaving={isSaving}
        isTesting={isTesting}
        isSlashCommandSyncing={isSlashCommandSyncing}
        onUpdateForm={handlers.updateForm}
        onToggleTargetPath={handlers.toggleTargetPath}
        onToggleAdapter={handlers.toggleSlashCommandAdapter}
        onSave={handlers.handleSave}
        onDelete={handlers.handleDelete}
        onTest={handlers.handleTest}
        onSyncSlashCommands={handlers.handleSyncSlashCommands}
      />
    </div>
  );
}
