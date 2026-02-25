import { useState, useEffect } from "react";
import { Download, Library, Loader2, Info, ChevronLeft } from "lucide-react";
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
import { cn } from "@/lib/utils";

interface TemplateBrowserProps {
  onInstalled: () => void;
}

export function TemplateBrowser({ onInstalled }: TemplateBrowserProps) {
  const [templates, setTemplates] = useState<TemplateSkill[]>([]);
  const [isOpen, setIsOpen] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [installingId, setInstallingId] = useState<string | null>(null);
  const [selectedTemplate, setSelectedTemplate] = useState<TemplateSkill | null>(null);
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
    } else {
      // Reset selection when closing
      setSelectedTemplate(null);
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
          {selectedTemplate ? (
            <div className="space-y-6">
              <DialogHeader>
                <div className="flex items-center gap-2 mb-2">
                  <Button
                    variant="ghost"
                    size="icon"
                    className="h-8 w-8 -ml-2"
                    onClick={() => setSelectedTemplate(null)}
                  >
                    <ChevronLeft className="h-4 w-4" />
                  </Button>
                  <DialogTitle>Template Details</DialogTitle>
                </div>
                <DialogDescription>
                  Review the details of this skill before installation.
                </DialogDescription>
              </DialogHeader>

              <div className="space-y-4 rounded-xl border border-white/5 bg-white/5 p-4">
                <div>
                  <h3 className="text-sm font-semibold text-primary">
                    {selectedTemplate.metadata.name}
                  </h3>
                  <p className="mt-1 text-sm text-muted-foreground leading-relaxed">
                    {selectedTemplate.metadata.description}
                  </p>
                </div>

                <div className="flex flex-wrap gap-2">
                  <div className="rounded-md border border-white/5 bg-white/5 px-2.5 py-1 text-[11px] font-medium text-muted-foreground">
                    {selectedTemplate.metadata.inputSchema.length} Parameter
                    {selectedTemplate.metadata.inputSchema.length !== 1 ? "s" : ""}
                  </div>
                  {selectedTemplate.metadata.entryPoint && (
                    <div className="rounded-md border border-white/5 bg-white/5 px-2.5 py-1 text-[11px] font-medium text-muted-foreground font-mono">
                      {selectedTemplate.metadata.entryPoint}
                    </div>
                  )}
                </div>

                {selectedTemplate.metadata.instructions && (
                  <div className="space-y-1.5 pt-2 border-t border-white/5">
                    <p className="text-[10px] uppercase font-bold tracking-widest text-muted-foreground/60">
                      Instructions Preview
                    </p>
                    <p className="text-xs text-muted-foreground italic line-clamp-3">
                      {selectedTemplate.metadata.instructions}
                    </p>
                  </div>
                )}
              </div>

              <div className="flex gap-3 pt-2">
                <Button
                  variant="outline"
                  className="flex-1"
                  onClick={() => setSelectedTemplate(null)}
                >
                  Cancel
                </Button>
                <Button
                  className="flex-1 glow-primary"
                  onClick={() => install(selectedTemplate.templateId)}
                  disabled={installingId !== null}
                >
                  {installingId === selectedTemplate.templateId ? (
                    <>
                      <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                      Installing...
                    </>
                  ) : (
                    <>
                      <Download className="mr-2 h-4 w-4" />
                      Install Skill
                    </>
                  )}
                </Button>
              </div>
            </div>
          ) : (
            <>
              <DialogHeader>
                <DialogTitle>Skill Templates</DialogTitle>
                <DialogDescription>
                  Browse built-in skill templates to quickly add new workflows to your toolkit.
                </DialogDescription>
              </DialogHeader>
              <div className="flex flex-col gap-3 py-4 max-h-[60vh] overflow-y-auto pr-1">
                {isLoading ? (
                  Array.from({ length: 3 }).map((_, i) => (
                    <div
                      key={i}
                      className="flex items-center justify-between p-4 border border-white/5 rounded-xl bg-white/5"
                    >
                      <div className="flex-1 space-y-2">
                        <Skeleton className="h-5 w-32 bg-white/10" />
                        <Skeleton className="h-4 w-full bg-white/5" />
                      </div>
                      <Skeleton className="h-9 w-20 ml-4 bg-white/10" />
                    </div>
                  ))
                ) : templates.length === 0 ? (
                  <div className="text-center text-sm text-muted-foreground py-12 border border-dashed border-white/5 rounded-xl">
                    No templates found.
                  </div>
                ) : (
                  templates.map((t) => (
                    <div
                      key={t.templateId}
                      className={cn(
                        "group flex items-center justify-between p-4 border border-white/5 rounded-xl bg-white/5 transition-all hover:bg-white/10 hover:border-white/10 cursor-pointer"
                      )}
                      onClick={() => setSelectedTemplate(t)}
                    >
                      <div className="flex-1 pr-4">
                        <h4 className="font-semibold text-sm group-hover:text-primary transition-colors">
                          {t.metadata.name}
                        </h4>
                        <p className="text-xs text-muted-foreground mt-1 line-clamp-1">
                          {t.metadata.description}
                        </p>
                      </div>
                      <Button
                        variant="ghost"
                        size="sm"
                        className="opacity-0 group-hover:opacity-100 transition-opacity"
                        onClick={(e) => {
                          e.stopPropagation();
                          setSelectedTemplate(t);
                        }}
                      >
                        <Info className="h-4 w-4 mr-2" />
                        Details
                      </Button>
                    </div>
                  ))
                )}
              </div>
            </>
          )}
        </DialogContent>
      </Dialog>
    </>
  );
}
