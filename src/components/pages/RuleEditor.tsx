import { useState, useEffect, useCallback } from "react";
import { ArrowLeft, Save, Eye, Check, Loader2, ExternalLink } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { MarkdownEditor } from "@/components/ui/markdown-editor";
import { useRulesStore } from "@/stores/rulesStore";
import { useToast } from "@/components/ui/toast";
import { useKeyboardShortcuts, SHORTCUTS } from "@/hooks/useKeyboardShortcuts";
import { ADAPTERS, type Rule, type Scope, type AdapterType } from "@/types/rule";
import { api } from "@/lib/tauri";

interface RuleEditorProps {
  rule: Rule | null;
  onBack: () => void;
  isNew?: boolean;
}

function getWordCount(text: string): number {
  return text.trim() ? text.trim().split(/\s+/).length : 0;
}

function getCharacterCount(text: string): number {
  return text.length;
}

export function RuleEditor({ rule, onBack, isNew = false }: RuleEditorProps) {
  const { createRule, updateRule } = useRulesStore();
  const { addToast } = useToast();

  const [name, setName] = useState(rule?.name || "");
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
  const [newPath, setNewPath] = useState("");

  const wordCount = getWordCount(content);
  const characterCount = getCharacterCount(content);

  useEffect(() => {
    const savedDefaults = localStorage.getItem("ruleweaver-default-adapters");
    if (savedDefaults) {
      try {
        const parsed = JSON.parse(savedDefaults);
        setDefaultAdapters(parsed);
      } catch {
        console.error("Failed to parse default adapters");
      }
    }
  }, []);

  useEffect(() => {
    if (rule) {
      setName(rule.name);
      setContent(rule.content);
      setScope(rule.scope);
      setTargetPaths(rule.targetPaths || []);
      setEnabledAdapters(rule.enabledAdapters);
      setPreviewAdapter(rule.enabledAdapters[0] || "gemini");
    } else if (isNew && enabledAdapters.length === 0) {
      const initialAdapters =
        defaultAdapters.length > 0 ? defaultAdapters : (["gemini", "opencode"] as AdapterType[]);
      setEnabledAdapters(initialAdapters);
      setPreviewAdapter(initialAdapters[0]);
    }
  }, [rule, isNew, defaultAdapters, enabledAdapters.length]);

  useEffect(() => {
    setHasUnsavedChanges(true);
  }, [name, content, scope, targetPaths, enabledAdapters]);

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
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "error",
      });
    } finally {
      setSaving(false);
    }
  }, [
    name,
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

  useKeyboardShortcuts({
    shortcuts: [
      {
        ...SHORTCUTS.SAVE,
        action: handleSave,
      },
    ],
  });

  const toggleAdapter = (adapter: AdapterType) => {
    if (enabledAdapters.includes(adapter)) {
      const newAdapters = enabledAdapters.filter((a) => a !== adapter);
      setEnabledAdapters(newAdapters);
      if (newAdapters.length > 0 && !newAdapters.includes(previewAdapter)) {
        setPreviewAdapter(newAdapters[0]);
      }
    } else {
      setEnabledAdapters([...enabledAdapters, adapter]);
    }
  };

  const addPath = () => {
    if (newPath.trim() && !targetPaths.includes(newPath.trim())) {
      setTargetPaths([...targetPaths, newPath.trim()]);
      setNewPath("");
    }
  };

  const removePath = (path: string) => {
    setTargetPaths(targetPaths.filter((p) => p !== path));
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
    const adapterInfo = ADAPTERS.find((a) => a.id === adapter);
    if (scope === "global") {
      return adapterInfo?.globalPath || "";
    }
    return targetPaths[0] ? `${targetPaths[0]}/${adapterInfo?.fileName}` : "";
  };

  const handleOpenFolder = async (adapter: AdapterType) => {
    const path = getAdapterPath(adapter);
    if (!path) return;
    const lastSeparatorIndex = Math.max(path.lastIndexOf("/"), path.lastIndexOf("\\"));
    const dirPath = lastSeparatorIndex >= 0 ? path.substring(0, lastSeparatorIndex) : path;
    try {
      await api.app.openInExplorer(dirPath);
    } catch {
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
          <Button onClick={handleSave} disabled={saving}>
            <Save className="mr-2 h-4 w-4" />
            {saving ? "Saving..." : "Save"}
          </Button>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-4 flex-1 min-h-0">
        <div className="lg:col-span-2 flex flex-col gap-4 min-h-0">
          <Card className="flex-1 flex flex-col min-h-0">
            <CardHeader className="pb-2">
              <Input
                placeholder="Rule name..."
                value={name}
                onChange={(e) => setName(e.target.value)}
                className="text-lg font-semibold border-none p-0 focus-visible:ring-0"
                aria-label="Rule name"
              />
            </CardHeader>
            <CardContent className="flex-1 flex flex-col min-h-0 p-0">
              <MarkdownEditor
                value={content}
                onChange={setContent}
                className="flex-1 border-0 rounded-none"
              />
              <div className="flex items-center justify-between px-3 py-2 border-t text-xs text-muted-foreground">
                <div className="flex gap-4">
                  <span>{wordCount} words</span>
                  <span>{characterCount} characters</span>
                </div>
                <span className="text-xs">
                  Press <kbd className="px-1 py-0.5 bg-muted rounded text-xs">Ctrl+S</kbd> to save
                </span>
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="text-sm flex items-center gap-2">
                <Eye className="h-4 w-4" />
                Preview
              </CardTitle>
            </CardHeader>
            <CardContent>
              {enabledAdapters.length > 0 && (
                <div className="flex items-center gap-2 mb-3 flex-wrap">
                  {enabledAdapters.map((adapter) => (
                    <Button
                      key={adapter}
                      variant={previewAdapter === adapter ? "default" : "outline"}
                      size="sm"
                      onClick={() => setPreviewAdapter(adapter)}
                    >
                      {ADAPTERS.find((a) => a.id === adapter)?.name}
                    </Button>
                  ))}
                </div>
              )}
              <pre className="p-3 rounded-md bg-muted text-xs overflow-auto max-h-40 font-mono">
                {generatePreview()}
              </pre>
              <div className="flex items-center justify-between mt-2">
                <p className="text-xs text-muted-foreground">
                  Will write to:{" "}
                  <code className="bg-muted px-1 rounded">{getAdapterPath(previewAdapter)}</code>
                </p>
                {getAdapterPath(previewAdapter) && (
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => handleOpenFolder(previewAdapter)}
                    className="h-7 text-xs"
                  >
                    <ExternalLink className="mr-1 h-3 w-3" />
                    Open folder
                  </Button>
                )}
              </div>
            </CardContent>
          </Card>
        </div>

        <Card className="h-fit">
          <CardHeader>
            <CardTitle className="text-sm">Settings</CardTitle>
          </CardHeader>
          <CardContent className="space-y-6">
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
                <label className="text-sm font-medium">Target Paths</label>
                <div className="flex gap-2">
                  <Input
                    placeholder="/path/to/repo"
                    value={newPath}
                    onChange={(e) => setNewPath(e.target.value)}
                    onKeyDown={(e) => e.key === "Enter" && addPath()}
                    aria-label="New target path"
                  />
                  <Button size="sm" onClick={addPath}>
                    Add
                  </Button>
                </div>
                <div className="space-y-1">
                  {targetPaths.map((path) => (
                    <div
                      key={path}
                      className="flex items-center justify-between p-2 rounded-md bg-muted text-xs"
                    >
                      <span className="truncate">{path}</span>
                      <Button
                        variant="ghost"
                        size="sm"
                        className="h-6 w-6 p-0"
                        onClick={() => removePath(path)}
                        aria-label={`Remove path ${path}`}
                      >
                        ×
                      </Button>
                    </div>
                  ))}
                </div>
              </div>
            )}

            <div className="space-y-2">
              <label className="text-sm font-medium">Adapters</label>
              <p className="text-xs text-muted-foreground">
                Select which AI tools should receive this rule
              </p>
              <div className="space-y-2">
                {ADAPTERS.map((adapter) => (
                  <label
                    key={adapter.id}
                    className="flex items-center justify-between p-2 rounded-md hover:bg-accent cursor-pointer"
                  >
                    <div className="flex items-center gap-2">
                      <Switch
                        checked={enabledAdapters.includes(adapter.id)}
                        onCheckedChange={() => toggleAdapter(adapter.id)}
                        aria-label={`Toggle ${adapter.name} adapter`}
                      />
                      <div>
                        <div className="text-sm font-medium">{adapter.name}</div>
                        <div className="text-xs text-muted-foreground">{adapter.fileName}</div>
                      </div>
                    </div>
                  </label>
                ))}
              </div>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
