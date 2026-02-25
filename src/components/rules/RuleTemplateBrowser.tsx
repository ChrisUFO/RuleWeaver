import { Download, ChevronLeft, Loader2 } from "lucide-react";
import { motion } from "framer-motion";
import { DialogHeader, DialogTitle, DialogDescription } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { api } from "@/lib/tauri";
import type { TemplateRule } from "@/types/rule";
import { BaseTemplateBrowser } from "../Templates/BaseTemplateBrowser";

interface RuleTemplateBrowserProps {
  onInstalled: () => void;
}

export function RuleTemplateBrowser({ onInstalled }: RuleTemplateBrowserProps) {
  return (
    <BaseTemplateBrowser<TemplateRule>
      title="Rule Templates"
      description="Browse built-in rule templates to quickly set up standards and personas."
      onInstalled={onInstalled}
      getTemplates={() => api.rules.getTemplates()}
      installTemplate={(id) => api.rules.installTemplate(id)}
      getName={(t) => t.metadata.name}
      getDescription={(t) => t.metadata.description}
      getTheme={(t) => t.theme}
      getTemplateId={(t) => t.templateId}
      getSearchableContent={(t) => [t.metadata.content]}
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
              Review the details of this rule before installation.
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
              </div>
            </div>

            {selectedTemplate.metadata.content && (
              <div className="space-y-2 pt-4 border-t border-white/5">
                <p className="text-[10px] uppercase font-bold tracking-widest text-muted-foreground/60">
                  Rule Content
                </p>
                <div className="relative group">
                  <pre className="text-xs text-muted-foreground bg-black/40 p-3 rounded-lg font-mono overflow-x-auto border border-white/5 max-h-[200px] custom-scrollbar">
                    {selectedTemplate.metadata.content}
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
                  Install Rule
                </>
              )}
            </Button>
          </div>
        </motion.div>
      )}
    />
  );
}
