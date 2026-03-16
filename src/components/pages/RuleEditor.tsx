import { useState, useMemo } from "react";
import { ArrowLeft, Save, Copy } from "lucide-react";
import { confirm } from "@tauri-apps/plugin-dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardHeader } from "@/components/ui/card";
import { MarkdownEditor, type FullscreenSaveState } from "@/components/ui/markdown-editor";
import { useKeyboardShortcuts, SHORTCUTS } from "@/hooks/useKeyboardShortcuts";
import { type Rule } from "@/types/rule";
import { useRuleEditorState } from "@/hooks/useRuleEditorState";
import { RuleEditorSettingsPanel } from "@/components/rules/RuleEditorSettingsPanel";

interface RuleEditorProps {
  rule: Rule | null;
  onBack: () => void;
  onSelectRule: (rule: Rule) => void;
  isNew?: boolean;
}

export function RuleEditor({ rule, onBack, onSelectRule, isNew = false }: RuleEditorProps) {
  const [isFullscreen, setIsFullscreen] = useState(false);
  const state = useRuleEditorState({ rule, isNew, onSelectRule });
  const {
    name,
    description,
    content,
    scope,
    targetPaths,
    enabledAdapters,
    saving,
    lastSaved,
    hasUnsavedChanges,
    autoSaveError,
    tools,
    availableRepos,
    setName,
    setDescription,
    setContent,
    setScope,
    handleSave,
    handleDuplicate,
    toggleAdapter,
    toggleTargetPath,
    getAdapterPath,
    handleOpenFolder,
    getSaveStatus,
  } = state;

  const fullscreenSaveState = useMemo<FullscreenSaveState>(
    () => ({
      saving,
      hasUnsavedChanges,
      lastSaved,
      autoSaveError,
      onSave: handleSave,
    }),
    [saving, hasUnsavedChanges, lastSaved, autoSaveError, handleSave]
  );

  const ruleTitle = name || rule?.name || "Untitled";

  const handleBackNavigation = async () => {
    if (hasUnsavedChanges) {
      const confirmed = await confirm(
        "You have unsaved changes that will be lost. Are you sure you want to leave?",
        { title: "Unsaved Changes", kind: "warning" }
      );
      if (confirmed) {
        state.cancelPendingAutoSave();
        onBack();
      }
    } else {
      onBack();
    }
  };

  useKeyboardShortcuts({
    shortcuts: [
      { ...SHORTCUTS.SAVE, action: handleSave },
      { ...SHORTCUTS.DUPLICATE, action: handleDuplicate },
    ],
  });

  if (isFullscreen) {
    return (
      <MarkdownEditor
        value={content}
        onChange={setContent}
        className="h-full"
        isFullscreen={true}
        onFullscreenChange={setIsFullscreen}
        fullscreenSaveState={fullscreenSaveState}
        fullscreenTitle={ruleTitle}
        fullscreenSaveStatus={getSaveStatus()}
      />
    );
  }

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-4">
          <Button variant="ghost" size="icon" onClick={handleBackNavigation} aria-label="Go back">
            <ArrowLeft className="h-4 w-4" />
          </Button>
          <h1 className="text-xl font-bold">{isNew ? "Create Rule" : `Edit: ${rule?.name}`}</h1>
        </div>
        <div className="flex items-center gap-4">
          {getSaveStatus()}
          {!isNew && (
            <Button
              variant="outline"
              onClick={handleDuplicate}
              disabled={saving}
              title="Duplicate (Ctrl+D)"
              className="glass border-white/5 hover:bg-white/5"
            >
              <Copy className="mr-2 h-4 w-4" />
              Duplicate
            </Button>
          )}
          <Button onClick={handleSave} disabled={saving} className="glow-primary">
            <Save className="mr-2 h-4 w-4" />
            {saving ? "Saving..." : "Save"}
          </Button>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-4 gap-6 flex-1 min-h-0">
        <div className="lg:col-span-3 flex flex-col min-h-0">
          <Card className="flex-1 flex flex-col min-h-0 glass-card premium-shadow border-none overflow-hidden">
            <CardHeader className="pb-2 space-y-2">
              <Input
                placeholder="Rule name..."
                value={name}
                onChange={(e) => setName(e.target.value)}
                className="text-lg font-semibold border-none p-0 focus-visible:ring-0"
                aria-label="Rule name"
              />
              <Input
                placeholder="Brief description of what this rule does..."
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                className="text-sm border-none p-0 h-auto text-muted-foreground focus-visible:ring-0"
                aria-label="Rule description"
              />
            </CardHeader>
            <div className="flex-1 flex flex-col min-h-0">
              <MarkdownEditor
                value={content}
                onChange={setContent}
                className="flex-1 border-0 rounded-none bg-transparent"
                isFullscreen={false}
                onFullscreenChange={setIsFullscreen}
              />
            </div>
          </Card>
        </div>

        <div className="lg:col-span-1">
          <RuleEditorSettingsPanel
            scope={scope}
            onScopeChange={setScope}
            targetPaths={targetPaths}
            onToggleTargetPath={toggleTargetPath}
            availableRepos={availableRepos}
            tools={tools}
            enabledAdapters={enabledAdapters}
            onToggleAdapter={toggleAdapter}
            getAdapterPath={getAdapterPath}
            onOpenFolder={handleOpenFolder}
          />
        </div>
      </div>
    </div>
  );
}
