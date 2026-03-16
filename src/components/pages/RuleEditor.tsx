import { useState } from "react";
import { ArrowLeft, Save, Copy } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardHeader } from "@/components/ui/card";
import { MarkdownEditor } from "@/components/ui/markdown-editor";
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
  const state = useRuleEditorState({ rule, isNew, onBack, onSelectRule });
  const {
    name,
    description,
    content,
    scope,
    targetPaths,
    enabledAdapters,
    saving,
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

  useKeyboardShortcuts({
    shortcuts: [
      { ...SHORTCUTS.SAVE, action: handleSave },
      { ...SHORTCUTS.DUPLICATE, action: handleDuplicate },
    ],
  });

  return (
    <div className="flex flex-col h-full">
      {!isFullscreen && (
        <>
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
                    isFullscreen={isFullscreen}
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
        </>
      )}

      {isFullscreen && (
        <MarkdownEditor
          value={content}
          onChange={setContent}
          className="flex-1"
          isFullscreen={isFullscreen}
          onFullscreenChange={setIsFullscreen}
        />
      )}
    </div>
  );
}
