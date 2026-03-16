import { useState, useEffect, useCallback, useRef } from "react";
import { Check, Loader2 } from "lucide-react";
import React from "react";
import { useRulesStore } from "@/stores/rulesStore";
import { useRegistryStore } from "@/stores/registryStore";
import { useToast } from "@/components/ui/toast";
import { useRepositoryRoots } from "@/hooks/useRepositoryRoots";
import { type Rule, type Scope, type AdapterType, type ToolEntry } from "@/types/rule";
import { api } from "@/lib/tauri";
import { featureManager, FEATURE_FLAGS } from "@/lib/featureManager";

const AUTO_SAVE_DELAY_MS = 3000;

function formatRelativeTime(date: Date): string {
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffSeconds = Math.floor(diffMs / 1000);
  if (diffSeconds < 60) return "just now";
  const diffMinutes = Math.floor(diffSeconds / 60);
  if (diffMinutes < 60) return `${diffMinutes}m ago`;
  const diffHours = Math.floor(diffMinutes / 60);
  if (diffHours < 24) return `${diffHours}h ago`;
  const diffDays = Math.floor(diffHours / 24);
  return `${diffDays}d ago`;
}

interface UseRuleEditorStateProps {
  rule: Rule | null;
  isNew: boolean;
  onSelectRule: (rule: Rule) => void;
}

export interface RuleEditorState {
  name: string;
  description: string;
  content: string;
  scope: Scope;
  targetPaths: string[];
  enabledAdapters: AdapterType[];
  previewAdapter: AdapterType;
  saving: boolean;
  lastSaved: Date | null;
  hasUnsavedChanges: boolean;
  autoSaveError: string | null;
  tools: ToolEntry[];
  availableRepos: string[];
  setName: (v: string) => void;
  setDescription: (v: string) => void;
  setContent: (v: string) => void;
  setScope: (v: Scope) => void;
  setPreviewAdapter: (v: AdapterType) => void;
  handleSave: () => Promise<boolean>;
  handleDuplicate: () => Promise<void>;
  toggleAdapter: (adapter: AdapterType) => void;
  toggleTargetPath: (path: string, checked: boolean) => void;
  generatePreview: () => string;
  getAdapterPath: (adapter: AdapterType) => string;
  handleOpenFolder: (adapter: AdapterType) => Promise<void>;
  getSaveStatus: () => React.ReactNode;
  cancelPendingAutoSave: () => void;
}

interface InitialSnapshot {
  name: string;
  description: string;
  content: string;
  scope: Scope;
  targetPaths: string[];
  enabledAdapters: AdapterType[];
}

function slugRuleName(ruleName: string): string {
  return ruleName
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .slice(0, 200);
}

export function useRuleEditorState({
  rule,
  isNew,
  onSelectRule,
}: UseRuleEditorStateProps): RuleEditorState {
  const { createRule, updateRule, duplicateRule } = useRulesStore();
  const { tools } = useRegistryStore();
  const { addToast } = useToast();
  const { roots: availableRepos } = useRepositoryRoots();

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
  const [isOpeningFolder, setIsOpeningFolder] = useState(false);
  const [lastSaved, setLastSaved] = useState<Date | null>(null);
  const [hasUnsavedChanges, setHasUnsavedChanges] = useState(false);
  const [autoSaveError, setAutoSaveError] = useState<string | null>(null);
  const [previewAdapter, setPreviewAdapter] = useState<AdapterType>("gemini");
  const isInitialized = useRef(false);
  const autoSaveTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const initialSnapshotRef = useRef<InitialSnapshot | null>(null);

  useEffect(() => {
    const loadDefaultAdapters = async () => {
      try {
        const savedDefaults = await api.settings.get("default_adapters");
        if (savedDefaults) {
          const parsed = JSON.parse(savedDefaults);
          setDefaultAdapters(parsed);
        } else {
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
      initialSnapshotRef.current = {
        name: rule.name,
        description: rule.description,
        content: rule.content,
        scope: rule.scope,
        targetPaths: rule.targetPaths || [],
        enabledAdapters: rule.enabledAdapters,
      };
      isInitialized.current = true;
    } else if (isNew && defaultAdapters.length > 0) {
      setEnabledAdapters(defaultAdapters);
      setPreviewAdapter(defaultAdapters[0]);
      initialSnapshotRef.current = {
        name: "",
        description: "",
        content: "",
        scope: "global",
        targetPaths: [],
        enabledAdapters: defaultAdapters,
      };
      isInitialized.current = true;
    }
  }, [rule, isNew, defaultAdapters]);

  useEffect(() => {
    if (!isInitialized.current || !initialSnapshotRef.current) return;

    const currentSnapshot: InitialSnapshot = {
      name,
      description,
      content,
      scope,
      targetPaths,
      enabledAdapters,
    };

    const hasChanges =
      JSON.stringify(currentSnapshot) !== JSON.stringify(initialSnapshotRef.current);
    setHasUnsavedChanges(hasChanges);
    if (hasChanges) {
      setAutoSaveError(null);
    }
  }, [name, description, content, scope, targetPaths, enabledAdapters]);

  useEffect(() => {
    const handleBeforeUnload = (e: BeforeUnloadEvent) => {
      if (hasUnsavedChanges) {
        e.preventDefault();
        e.returnValue = "";
      }
    };

    window.addEventListener("beforeunload", handleBeforeUnload);
    return () => window.removeEventListener("beforeunload", handleBeforeUnload);
  }, [hasUnsavedChanges]);

  const performSave = useCallback(
    async (silent = false): Promise<boolean> => {
      if (!name.trim()) {
        if (!silent) {
          addToast({
            title: "Validation Error",
            description: "Rule name is required",
            variant: "error",
          });
        }
        return false;
      }
      if (enabledAdapters.length === 0) {
        if (!silent) {
          addToast({
            title: "Validation Error",
            description: "At least one adapter must be selected",
            variant: "error",
          });
        }
        return false;
      }
      if (scope === "local" && targetPaths.length === 0) {
        if (!silent) {
          addToast({
            title: "Validation Error",
            description: "Local rules require at least one target path",
            variant: "error",
          });
        }
        return false;
      }
      if (content.trim().length === 0) {
        if (!silent) {
          addToast({
            title: "Validation Error",
            description: "Rule content cannot be empty",
            variant: "error",
          });
        }
        return false;
      }

      setSaving(true);
      if (silent) {
        setAutoSaveError(null);
      }
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
          if (!silent) {
            addToast({
              title: "Rule Created",
              description: `"${name}" has been created`,
              variant: "success",
            });
          }
        } else if (rule) {
          await updateRule(rule.id, {
            name: name.trim(),
            description: description.trim(),
            content,
            scope,
            targetPaths: scope === "local" ? targetPaths : undefined,
            enabledAdapters,
          });
          if (!silent) {
            addToast({
              title: "Rule Saved",
              description: `"${name}" has been updated`,
              variant: "success",
            });
          }
        }
        setLastSaved(new Date());
        setHasUnsavedChanges(false);
        initialSnapshotRef.current = {
          name: name.trim(),
          description: description.trim(),
          content,
          scope,
          targetPaths: scope === "local" ? targetPaths : [],
          enabledAdapters,
        };
        return true;
      } catch (error) {
        const errorMessage =
          typeof error === "string"
            ? error
            : error instanceof Error
              ? error.message
              : "Unknown error";
        if (silent) {
          setAutoSaveError(errorMessage);
        }
        addToast({
          title: silent ? "Auto-save Failed" : "Save Failed",
          description: errorMessage,
          variant: "error",
        });
        return false;
      } finally {
        setSaving(false);
      }
    },
    [
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
    ]
  );

  useEffect(() => {
    if (!hasUnsavedChanges || isNew || !isInitialized.current) return;

    if (autoSaveTimerRef.current) {
      clearTimeout(autoSaveTimerRef.current);
    }

    autoSaveTimerRef.current = setTimeout(() => {
      performSave(true);
    }, AUTO_SAVE_DELAY_MS);

    return () => {
      if (autoSaveTimerRef.current) {
        clearTimeout(autoSaveTimerRef.current);
      }
    };
  }, [hasUnsavedChanges, isNew, performSave]);

  const handleSave = useCallback(async (): Promise<boolean> => {
    if (autoSaveTimerRef.current) {
      clearTimeout(autoSaveTimerRef.current);
      autoSaveTimerRef.current = null;
    }
    return performSave(false);
  }, [performSave]);

  const cancelPendingAutoSave = useCallback(() => {
    if (autoSaveTimerRef.current) {
      clearTimeout(autoSaveTimerRef.current);
      autoSaveTimerRef.current = null;
    }
  }, []);

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
        targetPaths: scope === "local" ? targetPaths : null,
        enabledAdapters,
      });
      addToast({
        title: "Rule Duplicated",
        description: `"${name}" has been duplicated`,
        variant: "success",
      });
      onSelectRule(newRule);
      setHasUnsavedChanges(false);
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

  const toggleAdapter = useCallback(
    (adapter: AdapterType) => {
      setEnabledAdapters((prev) => {
        if (prev.includes(adapter)) {
          const next = prev.filter((a) => a !== adapter);
          if (next.length > 0 && adapter === previewAdapter) {
            setPreviewAdapter(next[0]);
          }
          return next;
        }
        return [...prev, adapter];
      });
    },
    [previewAdapter]
  );

  const toggleTargetPath = useCallback((path: string, checked: boolean) => {
    setTargetPaths((prev) => {
      if (checked) {
        if (prev.includes(path)) return prev;
        return [...prev, path];
      }
      return prev.filter((p) => p !== path);
    });
  }, []);

  const generatePreview = useCallback((): string => {
    const ruleName = name || "Untitled";
    return `<!-- Generated by RuleWeaver - Do not edit manually -->\n\n## ${ruleName}\n\n${content}\n`;
  }, [name, content]);

  const getAdapterPath = useCallback(
    (adapter: AdapterType): string => {
      const adapterInfo = tools.find((a) => a.id === adapter);
      const slug = slugRuleName(name) || (rule ? `rule-${rule.id}` : "rule");

      if (scope === "global") {
        const model = adapterInfo?.paths.globalRuleFileModel ?? adapterInfo?.ruleFileModel;
        if (model === "per_rule_dir" && adapterInfo?.paths.globalRulesDir) {
          return `${adapterInfo.paths.globalRulesDir}/${slug}.md`;
        }
        return adapterInfo?.paths.globalPath || "";
      }

      const localModel = adapterInfo?.paths.localRuleFileModel ?? adapterInfo?.ruleFileModel;
      if (localModel === "per_rule_dir" && adapterInfo?.paths.localRulesDirTemplate) {
        const localDir = adapterInfo.paths.localRulesDirTemplate.replace(
          "{repo}",
          targetPaths[0] || ""
        );
        if (!targetPaths[0]) return "";
        if (localDir.startsWith(targetPaths[0])) {
          return `${localDir}/${slug}.md`;
        }
        return `${targetPaths[0]}/${localDir}/${slug}.md`;
      }

      const fileName = adapterInfo?.paths.localPathTemplate.split(/[/\\]/).pop();
      return targetPaths[0] && fileName ? `${targetPaths[0]}/${fileName}` : "";
    },
    [tools, scope, targetPaths, name, rule]
  );

  const handleOpenFolder = useCallback(
    async (adapter: AdapterType) => {
      if (isOpeningFolder) return;

      const path = getAdapterPath(adapter);
      if (!path) return;
      const lastSeparatorIndex = Math.max(path.lastIndexOf("/"), path.lastIndexOf("\\"));
      const dirPath = lastSeparatorIndex >= 0 ? path.substring(0, lastSeparatorIndex) : path;

      setIsOpeningFolder(true);
      try {
        await api.app.openInExplorer(dirPath);
      } catch (error) {
        console.error("Failed to open folder in explorer", { dirPath, error });
        addToast({
          title: "Error",
          description: "Could not open folder",
          variant: "error",
          action: featureManager.isEnabled(FEATURE_FLAGS.ENHANCED_ERROR_UX)
            ? { label: "Retry", onClick: () => handleOpenFolder(adapter) }
            : undefined,
        });
      } finally {
        setIsOpeningFolder(false);
      }
    },
    [getAdapterPath, addToast, isOpeningFolder]
  );

  const getSaveStatus = useCallback((): React.ReactNode => {
    if (saving) {
      return React.createElement(
        "span",
        { className: "flex items-center gap-1 text-muted-foreground text-sm" },
        React.createElement(Loader2, { className: "h-3 w-3 animate-spin" }),
        "Saving..."
      );
    }
    if (hasUnsavedChanges) {
      return React.createElement(
        "span",
        { className: "text-muted-foreground text-sm" },
        "Unsaved changes"
      );
    }
    if (lastSaved) {
      return React.createElement(
        "span",
        { className: "flex items-center gap-1 text-muted-foreground text-sm" },
        React.createElement(Check, { className: "h-3 w-3 text-success" }),
        `Saved ${formatRelativeTime(lastSaved)}`
      );
    }
    return null;
  }, [saving, hasUnsavedChanges, lastSaved]);

  return {
    name,
    description,
    content,
    scope,
    targetPaths,
    enabledAdapters,
    previewAdapter,
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
    setPreviewAdapter,
    handleSave,
    handleDuplicate,
    toggleAdapter,
    toggleTargetPath,
    generatePreview,
    getAdapterPath,
    handleOpenFolder,
    getSaveStatus,
    cancelPendingAutoSave,
  };
}
