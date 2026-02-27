import { useState, useEffect, useCallback, useRef } from "react";
import {
  ArrowLeft,
  Save,
  Copy,
  Eye,
  Check,
  Loader2,
  ExternalLink,
  FileText,
  History as HistoryIcon,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { MarkdownEditor } from "@/components/ui/markdown-editor";
import { useRulesStore } from "@/stores/rulesStore";
import { useToast } from "@/components/ui/toast";
import { useKeyboardShortcuts, SHORTCUTS } from "@/hooks/useKeyboardShortcuts";
import { useRepositoryRoots } from "@/hooks/useRepositoryRoots";
import { type Rule, type Scope, type AdapterType } from "@/types/rule";
import { useRegistryStore } from "@/stores/registryStore";
import { api } from "@/lib/tauri";

interface RuleEditorProps {
  rule: Rule | null;
  onBack: () => void;
  onSelectRule: (rule: Rule) => void;
  isNew?: boolean;
}

// TODO: Refactor complexity - Component exceeds 500 lines. Consider extracting:
// - useRuleEditorState hook for form state management
// - RulePreview component
// - AdapterSettings component
function getWordCount(text: string): number {
  return text.trim() ? text.trim().split(/\s+/).length : 0;
}

function getCharacterCount(text: string): number {
  return text.length;
}

export function RuleEditor({ rule, onBack, onSelectRule, isNew = false }: RuleEditorProps) {
  const { createRule, updateRule, duplicateRule } = useRulesStore();
  const { tools } = useRegistryStore();
  const { addToast } = useToast();

  const [name, setName] = useState(rule?.name || "");
  const [description, setDescription] = useState(rule?.description || "");
  const [content, setContent] = useState(rule?.content || "");
  const [scope, setScope] = useState<Scope>(rule?.scope || "global");
  const [targetPaths, setTargetPaths] = useState<string[]>(rule?.targetPaths || []);
  const [defaultAdapters, setDefaultAdapters] = useState<AdapterType[]>([]);
  const [enabledAdapters, setEnabledAdapters] = useState<AdapterType[]>(
    rule?.enabledAdapters || []
  );
  const [saving, setSaving] = useState(false);
  const [lastSaved, setLastSaved] = useState<Date | null>(null);
  const [hasUnsavedChanges, setHasUnsavedChanges] = useState(false);
  const [previewAdapter, setPreviewAdapter] = useState<AdapterType>("gemini");
  const { roots: availableRepos } = useRepositoryRoots();
  const isInitialized = useRef(false);

  const wordCount = getWordCount(content);
  const characterCount = getCharacterCount(content);

  // Load default adapters from database settings
  useEffect(() => {
    const loadDefaultAdapters = async () => {
      try {
        const savedDefaults = await api.settings.get("default_adapters");
        if (savedDefaults) {
          const parsed = JSON.parse(savedDefaults);
          setDefaultAdapters(parsed);
        } else {
          // Fallback if no settings found yet
          setDefaultAdapters(["gemini", "opencode"]);
        }
      } catch (error) {
        console.error("Failed to load default adapters from database", { error });
        setDefaultAdapters(["gemini", "opencode"]);
      }
    };
    loadDefaultAdapters();
  }, []);

  useEffect(() => {
    if (isInitialized.current) return;

    if (rule) {
      setName(rule.name);
      setDescription(rule.description);
      setContent(rule.content);
      setScope(rule.scope);
      setTargetPaths(rule.targetPaths || []);
      setEnabledAdapters(rule.enabledAdapters);
      setPreviewAdapter(rule.enabledAdapters[0] || "gemini");
      isInitialized.current = true;
    } else if (isNew && defaultAdapters.length > 0) {
      setEnabledAdapters(defaultAdapters);
      setPreviewAdapter(defaultAdapters[0]);
      isInitialized.current = true;
    }
  }, [rule, isNew, defaultAdapters]);

  useEffect(() => {
    // Only set unsaved changes if we've finished initializing
    if (isInitialized.current) {
      setHasUnsavedChanges(true);
    }
  }, [name, description, content, scope, targetPaths, enabledAdapters]);

  const handleSave = useCallback(async () => {
    if (!name.trim()) {
      addToast({
        title: "Validation Error",
        description: "Rule name is required",
        variant: "error",
      });
      return;
    }

    if (enabledAdapters.length === 0) {
      addToast({
        title: "Validation Error",
        description: "At least one adapter must be selected",
        variant: "error",
      });
      return;
    }

    if (scope === "local" && targetPaths.length === 0) {
      addToast({
        title: "Validation Error",
        description: "Local rules require at least one target path",
        variant: "error",
      });
      return;
    }

    if (content.trim().length === 0) {
      addToast({
        title: "Validation Error",
        description: "Rule content cannot be empty",
        variant: "error",
      });
      return;
    }

    setSaving(true);
    try {
      if (isNew) {
        await createRule({
          name: name.trim(),
          description: description.trim(),
          content,
          scope,
          targetPaths: scope === "local" ? targetPaths : undefined,
          enabledAdapters,
        });
        addToast({
          title: "Rule Created",
          description: `"${name}" has been created`,
          variant: "success",
        });
      } else if (rule) {
        await updateRule(rule.id, {
          name: name.trim(),
          description: description.trim(),
          content,
          scope,
          targetPaths: scope === "local" ? targetPaths : undefined,
          enabledAdapters,
        });
        addToast({
          title: "Rule Saved",
          description: `"${name}" has been updated`,
          variant: "success",
        });
      }
      setLastSaved(new Date());
      setHasUnsavedChanges(false);
      onBack();
    } catch (error) {
      addToast({
        title: "Save Failed",
        description:
          typeof error === "string"
            ? error
            : error instanceof Error
              ? error.message
              : "Unknown error",
        variant: "error",
      });
    } finally {
      setSaving(false);
    }
  }, [
    name,
    description,
    content,
    scope,
    targetPaths,
    enabledAdapters,
    isNew,
    rule,
    createRule,
    updateRule,
    addToast,
    onBack,
  ]);

  const handleDuplicate = useCallback(async () => {
    if (!rule) return;

    setSaving(true);
    try {
      const newRule = await duplicateRule({
        ...rule,
        name: name.trim(),
        description: description.trim(),
        content,
        scope,
        targetPaths: scope === "local" ? targetPaths : undefined,
        enabledAdapters,
      });
      addToast({
        title: "Rule Duplicated",
        description: `"${name}" has been duplicated`,
        variant: "success",
      });
      setHasUnsavedChanges(false);
      onSelectRule(newRule);
    } catch (error) {
      addToast({
        title: "Duplicate Failed",
        description:
          typeof error === "string"
            ? error
            : error instanceof Error
              ? error.message
              : "Unknown error",
        variant: "error",
      });
    } finally {
      setSaving(false);
    }
  }, [
    rule,
    name,
    description,
    content,
    scope,
    targetPaths,
    enabledAdapters,
    duplicateRule,
    addToast,
    onSelectRule,
  ]);

  useKeyboardShortcuts({
    shortcuts: [
      {
        ...SHORTCUTS.SAVE,
        action: handleSave,
      },
      {
        ...SHORTCUTS.DUPLICATE,
        action: handleDuplicate,
      },
    ],
  });

  const toggleAdapter = useCallback(
    (adapter: AdapterType) => {
      setEnabledAdapters((prev) => {
        if (prev.includes(adapter)) {
          const next = prev.filter((a) => a !== adapter);
          // If the current preview adapter was removed, switch to another one
          if (next.length > 0 && adapter === previewAdapter) {
            setPreviewAdapter(next[0]);
          }
          return next;
        } else {
          return [...prev, adapter];
        }
      });
    },
    [previewAdapter]
  );

  const toggleTargetPath = (path: string, checked: boolean) => {
    setTargetPaths((prev) => {
      if (checked) {
        if (prev.includes(path)) return prev;
        return [...prev, path];
      }
      return prev.filter((p) => p !== path);
    });
  };

  const generatePreview = (): string => {
    const timestamp = rule ? new Date(rule.updatedAt * 1000).toISOString() : "New Rule";
    let preview = `<!-- Generated by RuleWeaver - Do not edit manually -->\n`;
    preview += `<!-- Last synced: ${timestamp} -->\n\n`;

    if (previewAdapter === "cline") {
      preview = `# Generated by RuleWeaver - Do not edit manually\n`;
      preview += `# Last synced: ${timestamp}\n\n`;
      preview += `# Rule: ${name || "Untitled"}\n${content}`;
    } else {
      preview += `<!-- Rule: ${name || "Untitled"} -->\n${content}`;
    }

    return preview;
  };

  const getAdapterPath = (adapter: AdapterType): string => {
    const adapterInfo = tools.find((a) => a.id === adapter);
    if (scope === "global") {
      return adapterInfo?.paths.globalPath || "";
    }
    const fileName = adapterInfo?.paths.localPathTemplate.split(/[/\\]/).pop();
    return targetPaths[0] && fileName ? `${targetPaths[0]}/${fileName}` : "";
  };

  const handleOpenFolder = async (adapter: AdapterType) => {
    const path = getAdapterPath(adapter);
    if (!path) return;
    const lastSeparatorIndex = Math.max(path.lastIndexOf("/"), path.lastIndexOf("\\"));
    const dirPath = lastSeparatorIndex >= 0 ? path.substring(0, lastSeparatorIndex) : path;
    try {
      await api.app.openInExplorer(dirPath);
    } catch (error) {
      console.error("Failed to open folder in explorer", { dirPath, error });
      addToast({
        title: "Error",
        description: "Could not open folder",
        variant: "error",
      });
    }
  };

  const getSaveStatus = () => {
    if (saving) {
      return (
        <span className="flex items-center gap-1 text-muted-foreground text-sm">
          <Loader2 className="h-3 w-3 animate-spin" />
          Saving...
        </span>
      );
    }
    if (hasUnsavedChanges) {
      return <span className="text-muted-foreground text-sm">Unsaved changes</span>;
    }
    if (lastSaved) {
      return (
        <span className="flex items-center gap-1 text-muted-foreground text-sm">
          <Check className="h-3 w-3 text-success" />
          Saved
        </span>
      );
    }
    return null;
  };

  return (
    <div className="flex flex-col h-full">
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

          <Card className="glass-card premium-shadow border-none overflow-hidden">
            <CardHeader className="bg-white/5 pb-4">
              <CardTitle className="text-sm font-semibold tracking-wide uppercase text-muted-foreground/80 flex items-center gap-2">
                <Eye className="h-4 w-4 text-primary" />
                Preview
              </CardTitle>
            </CardHeader>
            <CardContent className="pt-6">
              {enabledAdapters.length > 0 && (
                <div className="flex items-center gap-1.5 mb-4 p-1 glass border border-white/5 rounded-lg w-fit">
                  {enabledAdapters.map((adapter) => (
                    <Button
                      key={adapter}
                      variant={previewAdapter === adapter ? "default" : "ghost"}
                      size="sm"
                      onClick={() => setPreviewAdapter(adapter)}
                      className={cn(
                        "h-8 px-3 rounded-md transition-all",
                        previewAdapter === adapter
                          ? "glow-active shadow-sm"
                          : "text-muted-foreground"
                      )}
                    >
                      {tools.find((a) => a.id === adapter)?.name}
                    </Button>
                  ))}
                </div>
              )}
              <pre className="p-4 rounded-xl bg-black/40 border border-white/5 text-[11px] overflow-auto max-h-60 font-mono text-primary/80 selection:bg-primary/20">
                {generatePreview()}
              </pre>
              <div className="flex items-center justify-between mt-4">
                <p className="text-[10px] uppercase font-bold tracking-wider text-muted-foreground/40">
                  Target:{" "}
                  <span className="text-muted-foreground/80 lowercase font-normal">
                    {getAdapterPath(previewAdapter)}
                  </span>
                </p>
                {getAdapterPath(previewAdapter) && (
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => handleOpenFolder(previewAdapter)}
                    className="h-7 text-[10px] uppercase font-bold tracking-widest text-primary/60 hover:text-primary hover:bg-primary/5"
                  >
                    <ExternalLink className="mr-1.5 h-3 w-3" />
                    Explorer
                  </Button>
                )}
              </div>
            </CardContent>
          </Card>
        </div>

        <Card className="h-fit glass-card premium-shadow border-none overflow-hidden">
          <CardHeader className="bg-white/5 pb-4">
            <CardTitle className="text-sm font-semibold tracking-wide uppercase text-muted-foreground/80">
              Settings
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-6 pt-6">
            <div className="space-y-2">
              <label className="text-sm font-medium">Scope</label>
              <div className="flex items-center gap-4">
                <label className="flex items-center gap-2 cursor-pointer">
                  <input
                    type="radio"
                    name="scope"
                    checked={scope === "global"}
                    onChange={() => setScope("global")}
                    className="h-4 w-4"
                  />
                  <span className="text-sm">Global</span>
                </label>
                <label className="flex items-center gap-2 cursor-pointer">
                  <input
                    type="radio"
                    name="scope"
                    checked={scope === "local"}
                    onChange={() => setScope("local")}
                    className="h-4 w-4"
                  />
                  <span className="text-sm">Local</span>
                </label>
              </div>
            </div>

            {scope === "local" && (
              <div className="space-y-2">
                <label className="text-sm font-medium">Target Repositories</label>
                {availableRepos.length === 0 ? (
                  <p className="text-xs text-muted-foreground">
                    No repositories configured. Add repository roots in Settings first.
                  </p>
                ) : (
                  <div className="space-y-1">
                    {availableRepos.map((repoPath) => (
                      <label
                        key={repoPath}
                        className="flex items-center gap-2 p-2 rounded-md border text-xs"
                      >
                        <input
                          type="checkbox"
                          checked={targetPaths.includes(repoPath)}
                          onChange={(e) => toggleTargetPath(repoPath, e.target.checked)}
                        />
                        <span className="truncate">{repoPath}</span>
                      </label>
                    ))}
                  </div>
                )}
              </div>
            )}

            <div className="space-y-2">
              <label className="text-sm font-medium">Adapters</label>
              <p className="text-xs text-muted-foreground">
                Select which AI tools should receive this rule
              </p>
              <div className="space-y-2">
                {tools.map((adapter) => {
                  const fileName = adapter.paths.localPathTemplate.split(/[/\\]/).pop();
                  return (
                    <div
                      key={adapter.id}
                      className="flex items-center justify-between p-2 rounded-md hover:bg-accent cursor-pointer transition-colors"
                      onClick={() => toggleAdapter(adapter.id)}
                    >
                      <div className="flex items-center gap-2">
                        <Switch
                          checked={enabledAdapters.includes(adapter.id)}
                          onCheckedChange={() => toggleAdapter(adapter.id)}
                          aria-label={`Toggle ${adapter.name} adapter`}
                        />
                        <div>
                          <div className="text-sm font-medium">{adapter.name}</div>
                          <div className="text-xs text-muted-foreground">{fileName}</div>
                        </div>
                      </div>
                    </div>
                  );
                })}
              </div>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
