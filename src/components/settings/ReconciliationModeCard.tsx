import { useEffect, useState } from "react";
import { Switch } from "@/components/ui/switch";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { api } from "@/lib/tauri";
import { useToast } from "@/components/ui/toast";
import { SETTINGS_KEYS, RECONCILIATION_MODES } from "@/lib/constants";

export function ReconciliationModeCard() {
  const { addToast } = useToast();
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [isAutomatic, setIsAutomatic] = useState(false);

  useEffect(() => {
    loadSetting();
  }, []);

  const loadSetting = async () => {
    setIsLoading(true);
    try {
      const value = await api.app.getSetting(SETTINGS_KEYS.RECONCILIATION_MODE);
      setIsAutomatic(value === RECONCILIATION_MODES.AUTOMATIC);
    } catch {
      setIsAutomatic(false);
    } finally {
      setIsLoading(false);
    }
  };

  const handleToggle = async (checked: boolean) => {
    const newMode = checked ? RECONCILIATION_MODES.AUTOMATIC : RECONCILIATION_MODES.INTERACTIVE;
    setIsAutomatic(checked);
    setIsSaving(true);

    try {
      await api.app.setSetting(SETTINGS_KEYS.RECONCILIATION_MODE, newMode);
      addToast({
        title: "Setting Saved",
        description: `Reconciliation mode set to ${checked ? "automatic" : "interactive"}`,
        variant: "success",
      });
    } catch {
      addToast({
        title: "Error",
        description: "Failed to save setting",
        variant: "error",
      });
      setIsAutomatic(!checked);
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <Card className="glass-card premium-shadow border-none overflow-hidden">
      <CardHeader className="bg-white/5 pb-4">
        <CardTitle className="text-sm font-semibold tracking-wide uppercase text-muted-foreground/80">
          Reconciliation Mode
        </CardTitle>
        <CardDescription>Control how file changes are reconciled after edits</CardDescription>
      </CardHeader>
      <CardContent className="pt-6">
        <div className="flex items-center justify-between p-4 rounded-xl border border-white/5 bg-white/5">
          <div>
            <div className="font-bold">Automatic Reconciliation</div>
            <div className="text-xs text-muted-foreground mt-1">
              {isAutomatic
                ? "Changes are reconciled automatically after saves"
                : "Manual reconciliation required via Status page"}
            </div>
          </div>
          <Switch
            checked={isAutomatic}
            onCheckedChange={handleToggle}
            disabled={isLoading || isSaving}
          />
        </div>
      </CardContent>
    </Card>
  );
}
