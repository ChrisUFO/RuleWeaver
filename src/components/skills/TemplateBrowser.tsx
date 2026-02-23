import { useState, useEffect } from "react";
import { Download, Library, Loader2 } from "lucide-react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { api } from "@/lib/tauri";
import { useToast } from "@/components/ui/toast";
import type { TemplateSkill } from "@/types/skill";

interface TemplateBrowserProps {
  onInstalled: () => void;
}

export function TemplateBrowser({ onInstalled }: TemplateBrowserProps) {
  const [templates, setTemplates] = useState<TemplateSkill[]>([]);
  const [isOpen, setIsOpen] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [installingId, setInstallingId] = useState<string | null>(null);
  const { addToast } = useToast();

  useEffect(() => {
    if (isOpen) {
      setIsLoading(true);
      api.skills
        .getTemplates()
        .then(setTemplates)
        .catch((e) => {
          console.error("Failed to load templates:", e);
          addToast({
            title: "Failed to load templates",
            description: String(e),
            variant: "error",
          });
        })
        .finally(() => setIsLoading(false));
    }
  }, [isOpen, addToast]);

  const install = async (id: string) => {
    setInstallingId(id);
    try {
      await api.skills.installTemplate(id);
      addToast({ title: "Template Installed", variant: "success" });
      onInstalled();
      setIsOpen(false);
    } catch (e) {
      addToast({
        title: "Install Failed",
        description: e instanceof Error ? e.message : String(e),
        variant: "error",
      });
    } finally {
      setInstallingId(null);
    }
  };

  return (
    <>
      <Button variant="outline" size="sm" onClick={() => setIsOpen(true)}>
        <Library className="mr-2 h-4 w-4" />
        Browse Templates
      </Button>
      <Dialog open={isOpen} onOpenChange={setIsOpen}>
        <DialogContent className="sm:max-w-[500px]" onClose={() => setIsOpen(false)}>
          <DialogHeader>
            <DialogTitle>Skill Templates</DialogTitle>
            <DialogDescription>
              Install built-in skill templates to quickly add new workflows. Wait for the skill to
              compile.
            </DialogDescription>
          </DialogHeader>
          <div className="flex flex-col gap-4 py-4 max-h-[60vh] overflow-y-auto">
            {isLoading ? (
              Array.from({ length: 3 }).map((_, i) => (
                <div
                  key={i}
                  className="flex items-center justify-between p-4 border rounded-md bg-card"
                >
                  <div className="flex-1 space-y-2">
                    <Skeleton className="h-5 w-32" />
                    <Skeleton className="h-4 w-full" />
                    <Skeleton className="h-4 w-20" />
                  </div>
                  <Skeleton className="h-9 w-20 ml-4" />
                </div>
              ))
            ) : templates.length === 0 ? (
              <div className="text-center text-sm text-muted-foreground py-8">
                No templates found.
              </div>
            ) : (
              templates.map((t) => (
                <div
                  key={t.template_id}
                  className="flex items-center justify-between p-4 border rounded-md bg-card"
                >
                  <div className="flex-1 pr-4">
                    <h4 className="font-semibold">{t.metadata.name}</h4>
                    <p className="text-sm text-muted-foreground mt-1">{t.metadata.description}</p>
                    {t.metadata.input_schema.length > 0 && (
                      <div className="text-xs text-muted-foreground mt-2">
                        Takes {t.metadata.input_schema.length} parameter
                        {t.metadata.input_schema.length !== 1 ? "s" : ""}
                      </div>
                    )}
                  </div>
                  <Button
                    size="sm"
                    onClick={() => install(t.template_id)}
                    disabled={installingId !== null}
                  >
                    {installingId === t.template_id ? (
                      <span className="flex items-center">
                        <Loader2 className="mr-2 h-4 w-4 animate-spin" /> Installing...
                      </span>
                    ) : (
                      <>
                        <Download className="mr-2 h-4 w-4" />
                        Install
                      </>
                    )}
                  </Button>
                </div>
              ))
            )}
          </div>
        </DialogContent>
      </Dialog>
    </>
  );
}
