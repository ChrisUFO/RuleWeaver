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

interface UseRuleEditorStateProps {
  rule: Rule | null;
  isNew: boolean;
  onBack: () => void;
  onSelectRule: (rule: Rule) => void;
}

export interface RuleEditorState {
  // Form fields
  name: string;
  description: string;
  content: string;
  scope: Scope;
  targetPaths: string[];
  enabledAdapters: AdapterType[];
  previewAdapter: AdapterType;
  // Derived
  wordCount: number;
  characterCount: number;
  // Save state
  saving: boolean;
  lastSaved: Date | null;
  hasUnsavedChanges: boolean;
  // External data
  tools: ToolEntry[];
  availableRepos: string[];
  // Setters
  setName: (v: string) => void;
  setDescription: (v: string) => void;
  setContent: (v: string) => void;
  setScope: (v: Scope) => void;
  setPreviewAdapter: (v: AdapterType) => void;
  // Handlers
  handleSave: () => Promise<void>;
  handleDuplicate: () => Promise<void>;
  toggleAdapter: (adapter: AdapterType) => void;
  toggleTargetPath: (path: string, checked: boolean) => void;
  generatePreview: () => string;
  getAdapterPath: (adapter: AdapterType) => string;
  handleOpenFolder: (adapter: AdapterType) => Promise<void>;
  getSaveStatus: () => React.ReactNode;
}

function getWordCount(text: string): number {
  return text.trim() ? text.trim().split(/\s+/).length : 0;
}

function getCharacterCount(text: string): number {
  return text.length;
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
  onBack,
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
  const [previewAdapter, setPreviewAdapter] = useState<AdapterType>("gemini");
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
        "Saved"
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
    wordCount,
    characterCount,
    saving,
    lastSaved,
    hasUnsavedChanges,
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
  };
}
