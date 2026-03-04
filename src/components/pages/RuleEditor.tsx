import { ArrowLeft, Save, Copy, FileText, History as HistoryIcon } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { MarkdownEditor } from "@/components/ui/markdown-editor";
import { useKeyboardShortcuts, SHORTCUTS } from "@/hooks/useKeyboardShortcuts";
import { type Rule } from "@/types/rule";
import { useRuleEditorState } from "@/hooks/useRuleEditorState";
import { RulePreviewCard } from "@/components/rules/RulePreviewCard";
import { RuleEditorSettingsPanel } from "@/components/rules/RuleEditorSettingsPanel";

interface RuleEditorProps {
  rule: Rule | null;
  onBack: () => void;
  onSelectRule: (rule: Rule) => void;
  isNew?: boolean;
}

export function RuleEditor({ rule, onBack, onSelectRule, isNew = false }: RuleEditorProps) {
  const state = useRuleEditorState({ rule, isNew, onBack, onSelectRule });
  const {
    name,
    description,
    content,
    scope,
    targetPaths,
    enabledAdapters,
    previewAdapter,
    wordCount,
    characterCount,
    saving,
    tools,
    availableRepos,
    setName,
    setDescription,
    setContent,
    setScope,
    setPreviewAdapter,
    handleSave,
    handleDuplicate,
    toggleAdapter,
    toggleTargetPath,
    generatePreview,
    getAdapterPath,
    handleOpenFolder,
    getSaveStatus,
  } = state;

  useKeyboardShortcuts({
    shortcuts: [
      { ...SHORTCUTS.SAVE, action: handleSave },
      { ...SHORTCUTS.DUPLICATE, action: handleDuplicate },
    ],
  });

  return (
    <div className="flex flex-col h-full">
      {/* Header bar */}
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-4">
          <Button variant="ghost" size="icon" onClick={onBack} aria-label="Go back">
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
            {saving ? "Saving..." : "Save Selection"}
          </Button>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6 flex-1 min-h-0">
        <div className="lg:col-span-2 flex flex-col gap-6 min-h-0">
          {/* Editor card */}
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
            <CardContent className="flex-1 flex flex-col min-h-0 p-0">
              <MarkdownEditor
                value={content}
                onChange={setContent}
                className="flex-1 border-0 rounded-none bg-transparent"
              />
              <div className="flex items-center justify-between px-4 py-3 bg-white/5 border-t border-white/5 text-[10px] font-bold uppercase tracking-widest text-muted-foreground/60">
                <div className="flex gap-6">
                  <span className="flex items-center gap-1.5">
                    <FileText className="h-3 w-3" /> {wordCount} words
                  </span>
                  <span className="flex items-center gap-1.5">
                    <HistoryIcon className="h-3 w-3" /> {characterCount} chars
                  </span>
                </div>
                <div className="flex items-center gap-2">
                  <span className="opacity-60">Shortcut:</span>
                  <kbd className="px-1.5 py-0.5 bg-white/5 border border-white/10 rounded text-xs lowercase">
                    Ctrl+S
                  </kbd>
                </div>
              </div>
            </CardContent>
          </Card>

          {/* Preview card */}
          <RulePreviewCard
            enabledAdapters={enabledAdapters}
            previewAdapter={previewAdapter}
            onSelectPreviewAdapter={setPreviewAdapter}
            previewText={generatePreview()}
            targetPath={getAdapterPath(previewAdapter)}
            onOpenFolder={() => handleOpenFolder(previewAdapter)}
            tools={tools}
          />
        </div>

        {/* Settings panel */}
        <RuleEditorSettingsPanel
          scope={scope}
          onScopeChange={setScope}
          targetPaths={targetPaths}
          onToggleTargetPath={toggleTargetPath}
          availableRepos={availableRepos}
          tools={tools}
          enabledAdapters={enabledAdapters}
          onToggleAdapter={toggleAdapter}
        />
      </div>
    </div>
  );
}
