import { Download, ChevronLeft, Loader2, AlertCircle } from "lucide-react";
import { motion } from "framer-motion";
import { DialogHeader, DialogTitle, DialogDescription } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { api } from "@/lib/tauri";
import type { TemplateCommand } from "@/types/command";
import { BaseTemplateBrowser } from "../Templates/BaseTemplateBrowser";

interface CommandTemplateBrowserProps {
  onInstalled: () => void;
}

export function CommandTemplateBrowser({ onInstalled }: CommandTemplateBrowserProps) {
  return (
    <BaseTemplateBrowser<TemplateCommand>
      title="Command Templates"
      description="Browse built-in command templates to automate your repetitive tasks."
      onInstalled={onInstalled}
      getTemplates={() => api.commands.getTemplates()}
      installTemplate={(id) => api.commands.installTemplate(id)}
      getName={(t) => t.metadata.name}
      getDescription={(t) => t.metadata.description}
      getTheme={(t) => t.theme}
      getTemplateId={(t) => t.templateId}
      getSearchableContent={(t) => [t.metadata.script]}
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
              Review the details of this command before installation.
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-4 rounded-xl border border-white/5 bg-white/5 p-5 backdrop-blur-sm">
            <div>
              <h3 className="text-lg font-bold text-primary tracking-tight">
                {selectedTemplate.metadata.name}
              </h3>
              <div className="mt-2 flex items-center gap-2">
                <span className="px-2 py-0.5 rounded-full bg-primary/10 border border-primary/20 text-[10px] font-bold uppercase tracking-wider text-primary">
                  {selectedTemplate.theme}
                </span>
                {selectedTemplate.metadata.isPlaceholder && (
                  <span className="flex items-center gap-1 px-2 py-0.5 rounded-full bg-amber-500/10 border border-amber-500/20 text-[10px] font-bold uppercase tracking-wider text-amber-500">
                    <AlertCircle className="h-2.5 w-2.5" />
                    Placeholder
                  </span>
                )}
              </div>
              <p className="mt-3 text-sm text-muted-foreground/90 leading-relaxed border-l-2 border-primary/20 pl-4 py-1 italic">
                {selectedTemplate.metadata.description}
              </p>
            </div>

            {selectedTemplate.metadata.isPlaceholder && (
              <div className="p-3 rounded-lg bg-amber-500/5 border border-amber-500/10 text-[11px] text-amber-500/90 leading-relaxed">
                <span className="font-bold">Note:</span> This is a logic-only template. You will
                need to provide your own functional script after installation.
              </div>
            )}

            {selectedTemplate.metadata.script && (
              <div className="space-y-2 pt-4 border-t border-white/5">
                <p className="text-[10px] uppercase font-bold tracking-widest text-muted-foreground/60">
                  Executable Script
                </p>
                <div className="relative group">
                  <pre className="text-xs text-muted-foreground bg-black/40 p-3 rounded-lg font-mono overflow-x-auto border border-white/5 shadow-inner">
                    {selectedTemplate.metadata.script}
                  </pre>
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
                  Install Command
                </>
              )}
            </Button>
          </div>
        </motion.div>
      )}
    />
  );
}
