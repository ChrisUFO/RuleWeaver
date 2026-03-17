import { useState, useEffect, useCallback } from "react";
import { Sparkles, RefreshCw, Eye, EyeOff, Loader2, Check, ChevronDown, Info } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Switch } from "@/components/ui/switch";
import { Badge } from "@/components/ui/badge";
import { api } from "@/lib/tauri";
import { toast } from "@/lib/toast-helpers";
import type { useToast } from "@/components/ui/toast";
import type { AiProvider, AiSettings, ModelInfo, SaveAiSettingsInput } from "@/types/ai";
import { AI_PROVIDER_INFO } from "@/types/ai";

interface AiSettingsCardProps {
  addToast: ReturnType<typeof useToast>["addToast"];
}

export function AiSettingsCard({ addToast }: AiSettingsCardProps) {
  const [settings, setSettings] = useState<AiSettings | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [isTesting, setIsTesting] = useState(false);
  const [isLoadingModels, setIsLoadingModels] = useState(false);
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [showApiKey, setShowApiKey] = useState(false);
  const [apiKey, setApiKey] = useState("");
  const [customModel, setCustomModel] = useState("");
  const [showCustomModelInput, setShowCustomModelInput] = useState(false);

  const [formState, setFormState] = useState({
    provider: "openai" as AiProvider,
    baseUrl: "" as string | null,
    model: "",
    enabled: false,
    improvementPrompt: "" as string | null,
    generationPrompt: "" as string | null,
  });

  const loadModels = useCallback(async () => {
    try {
      setIsLoadingModels(true);
      const result = await api.ai.listModels();
      setModels(result);
    } catch {
      setModels([]);
    } finally {
      setIsLoadingModels(false);
    }
  }, []);

  const loadSettings = useCallback(async () => {
    try {
      setIsLoading(true);
      const result = await api.ai.getSettings();
      setSettings(result);
      setFormState({
        provider: result.provider,
        baseUrl: result.baseUrl,
        model: result.model,
        enabled: result.enabled,
        improvementPrompt: result.improvementPrompt,
        generationPrompt: result.generationPrompt,
      });
      if (result.enabled && result.hasApiKey) {
        loadModels();
      }
    } catch (error) {
      toast.error(addToast, { title: "Failed to load AI settings", error });
    } finally {
      setIsLoading(false);
    }
  }, [addToast, loadModels]);

  useEffect(() => {
    loadSettings();
  }, [loadSettings]);

  const handleSave = async () => {
    try {
      setIsSaving(true);
      const input: SaveAiSettingsInput = {
        provider: formState.provider,
        baseUrl: formState.baseUrl || null,
        model: formState.model,
        apiKey: apiKey || undefined,
        improvementPrompt: formState.improvementPrompt || null,
        generationPrompt: formState.generationPrompt || null,
        enabled: formState.enabled,
      };
      const result = await api.ai.saveSettings(input);
      setSettings(result);
      setApiKey("");
      toast.success(addToast, {
        title: "AI settings saved",
        description: "Your AI configuration has been updated",
      });
      if (result.enabled && result.hasApiKey) {
        loadModels();
      }
    } catch (error) {
      toast.error(addToast, { title: "Failed to save AI settings", error });
    } finally {
      setIsSaving(false);
    }
  };

  const handleTest = async () => {
    try {
      setIsTesting(true);
      const result = await api.ai.testConnection();
      if (result.success) {
        toast.success(addToast, {
          title: "Connection successful",
          description: result.modelAvailable
            ? "Model is available"
            : "Connected but model availability unknown",
        });
      } else {
        toast.error(addToast, {
          title: "Connection failed",
          description: result.error || "Unknown error",
        });
      }
    } catch (error) {
      toast.error(addToast, { title: "Connection test failed", error });
    } finally {
      setIsTesting(false);
    }
  };

  const handleProviderChange = (provider: AiProvider) => {
    const info = AI_PROVIDER_INFO[provider];
    setFormState((prev) => ({
      ...prev,
      provider,
      baseUrl: info.requiresBaseUrl ? prev.baseUrl : null,
      model: "",
    }));
    setModels([]);
    setShowCustomModelInput(false);
    setCustomModel("");
  };

  const handleModelSelect = (modelId: string) => {
    if (modelId === "__custom__") {
      setShowCustomModelInput(true);
      setFormState((prev) => ({ ...prev, model: customModel }));
    } else {
      setShowCustomModelInput(false);
      setFormState((prev) => ({ ...prev, model: modelId }));
    }
  };

  const providerInfo = AI_PROVIDER_INFO[formState.provider] ?? {
    name: formState.provider,
    description: "",
    requiresBaseUrl: false,
  };
  const hasChanges =
    settings &&
    (formState.provider !== settings.provider ||
      formState.baseUrl !== settings.baseUrl ||
      formState.model !== settings.model ||
      formState.enabled !== settings.enabled ||
      formState.improvementPrompt !== settings.improvementPrompt ||
      formState.generationPrompt !== settings.generationPrompt ||
      apiKey !== "");

  if (isLoading) {
    return (
      <Card className="glass-card premium-shadow border-none overflow-hidden">
        <CardContent className="flex items-center justify-center py-12">
          <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
        </CardContent>
      </Card>
    );
  }

  return (
    <Card className="glass-card premium-shadow border-none overflow-hidden">
      <CardHeader className="bg-white/5 pb-4">
        <div className="flex items-center justify-between">
          <div>
            <CardTitle className="text-sm font-bold uppercase tracking-widest text-muted-foreground/60 flex items-center gap-2">
              <Sparkles className="h-4 w-4" />
              AI Integration
            </CardTitle>
            <CardDescription>
              Configure AI providers for rule improvement and generation
            </CardDescription>
          </div>
          <Switch
            checked={formState.enabled}
            onCheckedChange={(checked) => setFormState((prev) => ({ ...prev, enabled: checked }))}
          />
        </div>
      </CardHeader>
      <CardContent className="space-y-4 pt-6">
        {!formState.enabled && (
          <div className="rounded-xl border border-white/5 bg-white/5 p-4 text-sm text-muted-foreground">
            Enable AI integration to improve rules with AI assistance or generate new rules from
            descriptions.
          </div>
        )}

        {formState.enabled && (
          <>
            <div className="rounded-xl border border-amber-500/30 bg-amber-500/10 p-3 text-sm text-amber-200 flex items-start gap-2">
              <Info className="h-4 w-4 mt-0.5 flex-shrink-0" />
              <div>
                <p className="font-medium">Usage Notice</p>
                <p className="text-xs text-amber-200/80 mt-1">
                  API calls may incur costs depending on your provider. Your rule content will be
                  sent to the configured AI provider for processing.
                </p>
              </div>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div className="space-y-2">
                <label className="text-xs font-bold uppercase tracking-wider text-muted-foreground">
                  Provider
                </label>
                <div className="relative">
                  <select
                    value={formState.provider}
                    onChange={(e) => handleProviderChange(e.target.value as AiProvider)}
                    className="w-full h-10 rounded-lg border border-white/10 bg-white/5 px-3 text-sm appearance-none cursor-pointer focus:outline-none focus:ring-2 focus:ring-primary/50"
                  >
                    {(
                      Object.entries(AI_PROVIDER_INFO) as [
                        AiProvider,
                        (typeof AI_PROVIDER_INFO)[AiProvider],
                      ][]
                    ).map(([id, info]) => (
                      <option key={id} value={id} className="bg-background text-foreground">
                        {info.name}
                      </option>
                    ))}
                  </select>
                  <ChevronDown className="absolute right-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground pointer-events-none" />
                </div>
                <p className="text-[10px] text-muted-foreground">{providerInfo.description}</p>
              </div>

              <div className="space-y-2">
                <label className="text-xs font-bold uppercase tracking-wider text-muted-foreground">
                  Model
                </label>
                {isLoadingModels ? (
                  <div className="flex items-center gap-2 h-10 px-3 rounded-lg border border-white/10 bg-white/5">
                    <Loader2 className="h-4 w-4 animate-spin" />
                    <span className="text-sm text-muted-foreground">Loading models...</span>
                  </div>
                ) : models.length > 0 ? (
                  <div className="relative">
                    <select
                      value={showCustomModelInput ? "__custom__" : formState.model}
                      onChange={(e) => handleModelSelect(e.target.value)}
                      className="w-full h-10 rounded-lg border border-white/10 bg-white/5 px-3 text-sm appearance-none cursor-pointer focus:outline-none focus:ring-2 focus:ring-primary/50"
                    >
                      <option value="" className="bg-background text-muted-foreground">
                        Select a model
                      </option>
                      {models.map((model) => (
                        <option
                          key={model.id}
                          value={model.id}
                          className="bg-background text-foreground"
                        >
                          {model.name || model.id}
                        </option>
                      ))}
                      <option value="__custom__" className="bg-background text-foreground">
                        Custom model name...
                      </option>
                    </select>
                    <ChevronDown className="absolute right-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground pointer-events-none" />
                  </div>
                ) : (
                  <input
                    type="text"
                    value={formState.model}
                    onChange={(e) => setFormState((prev) => ({ ...prev, model: e.target.value }))}
                    placeholder="Enter model name (e.g., gpt-4o-mini)"
                    className="w-full h-10 rounded-lg border border-white/10 bg-white/5 px-3 text-sm focus:outline-none focus:ring-2 focus:ring-primary/50"
                  />
                )}
                {showCustomModelInput && (
                  <input
                    type="text"
                    value={customModel}
                    onChange={(e) => {
                      setCustomModel(e.target.value);
                      setFormState((prev) => ({ ...prev, model: e.target.value }));
                    }}
                    placeholder="Enter custom model name"
                    className="w-full h-10 rounded-lg border border-white/10 bg-white/5 px-3 text-sm focus:outline-none focus:ring-2 focus:ring-primary/50"
                  />
                )}
              </div>
            </div>

            {providerInfo.requiresBaseUrl && (
              <div className="space-y-2">
                <label className="text-xs font-bold uppercase tracking-wider text-muted-foreground">
                  Base URL <span className="text-amber-500">*</span>
                </label>
                <input
                  type="text"
                  value={formState.baseUrl || ""}
                  onChange={(e) => setFormState((prev) => ({ ...prev, baseUrl: e.target.value }))}
                  placeholder="https://api.example.com/v1"
                  className="w-full h-10 rounded-lg border border-white/10 bg-white/5 px-3 text-sm focus:outline-none focus:ring-2 focus:ring-primary/50"
                />
              </div>
            )}

            <div className="space-y-2">
              <label className="text-xs font-bold uppercase tracking-wider text-muted-foreground">
                API Key{" "}
                {settings?.hasApiKey && (
                  <Badge variant="outline" className="ml-2 text-[9px]">
                    Set
                  </Badge>
                )}
              </label>
              <div className="relative">
                <input
                  type={showApiKey ? "text" : "password"}
                  value={apiKey}
                  onChange={(e) => setApiKey(e.target.value)}
                  placeholder={settings?.hasApiKey ? "••••••••••••••••" : "Enter your API key"}
                  className="w-full h-10 rounded-lg border border-white/10 bg-white/5 px-3 pr-10 text-sm focus:outline-none focus:ring-2 focus:ring-primary/50"
                />
                <button
                  type="button"
                  onClick={() => setShowApiKey(!showApiKey)}
                  className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
                >
                  {showApiKey ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                </button>
              </div>
              <p className="text-[10px] text-muted-foreground">
                API keys are stored securely in your system's credential manager.
              </p>
            </div>

            <div className="flex flex-wrap gap-2 pt-2">
              <Button
                variant="outline"
                className="glass"
                onClick={handleTest}
                disabled={isTesting || !settings?.hasApiKey}
              >
                {isTesting ? (
                  <>
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                    Testing...
                  </>
                ) : (
                  <>
                    <RefreshCw className="mr-2 h-4 w-4" />
                    Test Connection
                  </>
                )}
              </Button>

              <Button
                onClick={handleSave}
                disabled={isSaving || !hasChanges}
                className="glow-primary"
              >
                {isSaving ? (
                  <>
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                    Saving...
                  </>
                ) : (
                  <>
                    <Check className="mr-2 h-4 w-4" />
                    Save Settings
                  </>
                )}
              </Button>
            </div>

            <details className="group">
              <summary className="cursor-pointer text-xs font-bold uppercase tracking-wider text-muted-foreground hover:text-foreground flex items-center gap-2 pt-4">
                <ChevronDown className="h-3 w-3 transition-transform group-open:rotate-180" />
                Custom Prompts (Advanced)
              </summary>
              <div className="space-y-4 pt-4">
                <div className="space-y-2">
                  <label className="text-xs font-bold uppercase tracking-wider text-muted-foreground">
                    Improvement Prompt
                  </label>
                  <textarea
                    value={formState.improvementPrompt || ""}
                    onChange={(e) =>
                      setFormState((prev) => ({ ...prev, improvementPrompt: e.target.value }))
                    }
                    placeholder="Leave empty to use the default prompt..."
                    rows={4}
                    className="w-full rounded-lg border border-white/10 bg-white/5 px-3 py-2 text-sm font-mono focus:outline-none focus:ring-2 focus:ring-primary/50 resize-none"
                  />
                </div>
                <div className="space-y-2">
                  <label className="text-xs font-bold uppercase tracking-wider text-muted-foreground">
                    Generation Prompt
                  </label>
                  <textarea
                    value={formState.generationPrompt || ""}
                    onChange={(e) =>
                      setFormState((prev) => ({ ...prev, generationPrompt: e.target.value }))
                    }
                    placeholder="Leave empty to use the default prompt..."
                    rows={4}
                    className="w-full rounded-lg border border-white/10 bg-white/5 px-3 py-2 text-sm font-mono focus:outline-none focus:ring-2 focus:ring-primary/50 resize-none"
                  />
                </div>
                <Button
                  variant="outline"
                  size="sm"
                  className="glass text-xs"
                  onClick={async () => {
                    try {
                      const [improvement, generation] = await api.ai.getDefaultPrompts();
                      setFormState((prev) => ({
                        ...prev,
                        improvementPrompt: improvement,
                        generationPrompt: generation,
                      }));
                    } catch {
                      toast.error(addToast, {
                        title: "Failed to load default prompts",
                        description: "Could not retrieve default prompts from backend",
                      });
                    }
                  }}
                >
                  Reset to Defaults
                </Button>
              </div>
            </details>
          </>
        )}
      </CardContent>
    </Card>
  );
}
