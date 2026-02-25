import { Download, ChevronLeft, Loader2 } from "lucide-react";
import { motion } from "framer-motion";
import { DialogHeader, DialogTitle, DialogDescription } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { api } from "@/lib/tauri";
import type { TemplateSkill } from "@/types/skill";
import { BaseTemplateBrowser } from "../Templates/BaseTemplateBrowser";

interface TemplateBrowserProps {
  onInstalled: () => void;
}

export function TemplateBrowser({ onInstalled }: TemplateBrowserProps) {
  return (
    <BaseTemplateBrowser<TemplateSkill>
      title="Skill Templates"
      description="Browse built-in skill templates to quickly add new workflows to your toolkit."
      onInstalled={onInstalled}
      getTemplates={() => api.skills.getTemplates()}
      installTemplate={(id) => api.skills.installTemplate(id)}
      getName={(t) => t.metadata.name}
      getDescription={(t) => t.metadata.description}
      getTheme={(t) => t.theme}
      getTemplateId={(t) => t.templateId}
      getSearchableContent={(t) => [t.metadata.instructions, t.metadata.entryPoint || ""]}
      renderDetail={(selectedTemplate, install, isInstalling, onBack) => (
        <motion.div
          initial={{ opacity: 0, x: 20 }}
          animate={{ opacity: 1, x: 0 }}
          exit={{ opacity: 0, x: -20 }}
          className="space-y-6"
        >
          <DialogHeader>
            <div className="flex items-center gap-2 mb-2">
              <Button
                variant="ghost"
                size="icon"
                className="h-8 w-8 -ml-2 hover:bg-white/10"
                onClick={onBack}
              >
                <ChevronLeft className="h-4 w-4" />
              </Button>
              <DialogTitle className="text-xl font-bold tracking-tight">
                Template Details
              </DialogTitle>
            </div>
            <DialogDescription className="text-muted-foreground/80">
              Review the details of this skill before installation.
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-4 rounded-xl border border-white/5 bg-white/5 p-5 backdrop-blur-sm">
            <div>
              <h3 className="text-lg font-bold text-primary tracking-tight">
                {selectedTemplate.metadata.name}
              </h3>
              <div className="mt-2 flex flex-wrap gap-2">
                <span className="px-2 py-0.5 rounded-full bg-primary/10 border border-primary/20 text-[10px] font-bold uppercase tracking-wider text-primary">
                  {selectedTemplate.theme}
                </span>
                <span className="px-2 py-0.5 rounded-full bg-white/5 border border-white/10 text-[10px] font-bold uppercase tracking-wider text-muted-foreground">
                  {selectedTemplate.metadata.inputSchema.length} Parameter
                  {selectedTemplate.metadata.inputSchema.length !== 1 ? "s" : ""}
                </span>
              </div>
              <p className="mt-3 text-sm text-muted-foreground/90 leading-relaxed border-l-2 border-primary/20 pl-4 py-1 italic">
                {selectedTemplate.metadata.description}
              </p>
            </div>

            <div className="flex flex-wrap gap-2 pt-2">
              {selectedTemplate.metadata.entryPoint && (
                <div className="w-full">
                  <p className="text-[10px] uppercase font-bold tracking-widest text-muted-foreground/60 mb-1.5 ml-1">
                    Entry Point
                  </p>
                  <div className="rounded-lg border border-white/5 bg-black/40 px-3 py-2 text-[11px] font-medium text-muted-foreground font-mono shadow-inner border-white/5">
                    {selectedTemplate.metadata.entryPoint}
                  </div>
                </div>
              )}
            </div>

            {selectedTemplate.metadata.instructions && (
              <div className="space-y-2 pt-4 border-t border-white/5">
                <p className="text-[10px] uppercase font-bold tracking-widest text-muted-foreground/60">
                  Instructions Template
                </p>
                <div className="relative group">
                  <p className="text-xs text-muted-foreground bg-black/20 p-3 rounded-lg border border-white/5 italic line-clamp-4">
                    {selectedTemplate.metadata.instructions}
                  </p>
                </div>
              </div>
            )}
          </div>

          <div className="flex gap-3 pt-2">
            <Button variant="outline" className="flex-1 h-11" onClick={onBack}>
              Back to List
            </Button>
            <Button
              className="flex-1 h-11 glow-primary relative overflow-hidden group"
              onClick={() => install(selectedTemplate.templateId)}
              disabled={isInstalling}
            >
              <motion.div
                className="absolute inset-0 bg-white/10 opacity-0 group-hover:opacity-100 transition-opacity"
                animate={{ x: ["-100%", "100%"] }}
                transition={{ repeat: Infinity, duration: 2, ease: "linear" }}
              />
              {isInstalling ? (
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
        </motion.div>
      )}
    />
  );
}
