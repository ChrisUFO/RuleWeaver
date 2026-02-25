import { useState, useEffect } from "react";
import { Download, Library, Loader2, ChevronLeft, Search, SearchX } from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
import { Input } from "@/components/ui/input";
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
  const [searchQuery, setSearchQuery] = useState("");
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
                    onClick={() => setSelectedTemplate(null)}
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
                <Button
                  variant="outline"
                  className="flex-1 h-11"
                  onClick={() => setSelectedTemplate(null)}
                >
                  Back to List
                </Button>
                <Button
                  className="flex-1 h-11 glow-primary relative overflow-hidden group"
                  onClick={() => install(selectedTemplate.templateId)}
                  disabled={installingId !== null}
                >
                  <motion.div
                    className="absolute inset-0 bg-white/10 opacity-0 group-hover:opacity-100 transition-opacity"
                    animate={{ x: ["-100%", "100%"] }}
                    transition={{ repeat: Infinity, duration: 2, ease: "linear" }}
                  />
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
            </motion.div>
          ) : (
            <>
              <DialogHeader>
                <DialogTitle>Skill Templates</DialogTitle>
                <DialogDescription>
                  Browse built-in skill templates to quickly add new workflows to your toolkit.
                </DialogDescription>
                <div className="relative mt-4">
                  <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                  <Input
                    placeholder="Search templates..."
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                    className="pl-9 bg-white/5 border-white/10"
                  />
                </div>
              </DialogHeader>
              <div className="flex flex-col gap-3 py-4 max-h-[60vh] overflow-y-auto pr-1 custom-scrollbar">
                <AnimatePresence mode="popLayout" initial={false}>
                  {isLoading ? (
                    <motion.div
                      key="loading"
                      initial={{ opacity: 0 }}
                      animate={{ opacity: 1 }}
                      exit={{ opacity: 0 }}
                      className="space-y-3"
                    >
                      {Array.from({ length: 3 }).map((_, i) => (
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
                      ))}
                    </motion.div>
                  ) : templates.length === 0 ? (
                    <motion.div
                      key="empty"
                      initial={{ opacity: 0, y: 10 }}
                      animate={{ opacity: 1, y: 0 }}
                      className="flex flex-col items-center justify-center text-center text-sm text-muted-foreground py-16 px-4 border border-dashed border-white/10 rounded-2xl bg-white/[0.02]"
                    >
                      <div className="h-12 w-12 rounded-full bg-white/5 flex items-center justify-center mb-4">
                        <Library className="h-6 w-6 text-muted-foreground/40" />
                      </div>
                      <p className="font-medium text-muted-foreground">No templates available</p>
                      <p className="text-xs text-muted-foreground/60 mt-1">
                        Check back later for new presets.
                      </p>
                    </motion.div>
                  ) : (
                    (() => {
                      const filtered = templates.filter(
                        (t) =>
                          t.metadata.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
                          t.metadata.description
                            .toLowerCase()
                            .includes(searchQuery.toLowerCase()) ||
                          t.theme.toLowerCase().includes(searchQuery.toLowerCase())
                      );

                      const groups = filtered.reduce(
                        (acc, t) => {
                          const theme = t.theme || "General";
                          if (!acc[theme]) acc[theme] = [];
                          acc[theme].push(t);
                          return acc;
                        },
                        {} as Record<string, TemplateSkill[]>
                      );

                      const sortedThemes = Object.keys(groups).sort();

                      if (filtered.length === 0) {
                        return (
                          <motion.div
                            key="no-results"
                            initial={{ opacity: 0, y: 10 }}
                            animate={{ opacity: 1, y: 0 }}
                            className="flex flex-col items-center justify-center text-center text-sm text-muted-foreground py-16 px-4 border border-dashed border-white/10 rounded-2xl bg-white/[0.02]"
                          >
                            <div className="h-12 w-12 rounded-full bg-white/5 flex items-center justify-center mb-4">
                              <SearchX className="h-6 w-6 text-muted-foreground/40" />
                            </div>
                            <p className="font-medium text-muted-foreground">
                              No matching templates found
                            </p>
                            <p className="text-xs text-muted-foreground/60 mt-1 max-w-[200px]">
                              Try adjusting your search or browse a different theme.
                            </p>
                          </motion.div>
                        );
                      }

                      return (
                        <motion.div
                          key="results"
                          initial={{ opacity: 0 }}
                          animate={{ opacity: 1 }}
                          exit={{ opacity: 0 }}
                          className="space-y-6"
                        >
                          {sortedThemes.map((theme) => (
                            <div key={theme} className="space-y-3">
                              <h5 className="text-[10px] uppercase font-bold tracking-widest text-primary/70 px-1 pt-2 flex items-center gap-2">
                                {theme}
                                <div className="h-px flex-1 bg-gradient-to-r from-primary/20 to-transparent" />
                              </h5>
                              <div className="flex flex-col gap-3">
                                {groups[theme].map((t) => (
                                  <motion.div
                                    layout
                                    key={t.templateId}
                                    initial={{ opacity: 0, scale: 0.98 }}
                                    animate={{ opacity: 1, scale: 1 }}
                                    whileHover={{
                                      scale: 1.01,
                                      backgroundColor: "rgba(255,255,255,0.08)",
                                    }}
                                    whileTap={{ scale: 0.99 }}
                                    className={cn(
                                      "group flex items-center justify-between p-4 border border-white/5 rounded-xl bg-white/[0.04] transition-colors cursor-pointer"
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
                                      className="h-8 px-3 opacity-0 group-hover:opacity-100 transition-all bg-white/5 hover:bg-primary hover:text-primary-foreground"
                                      onClick={(e) => {
                                        e.stopPropagation();
                                        setSelectedTemplate(t);
                                      }}
                                    >
                                      Details
                                    </Button>
                                  </motion.div>
                                ))}
                              </div>
                            </div>
                          ))}
                        </motion.div>
                      );
                    })()
                  )}
                </AnimatePresence>
              </div>
            </>
          )}
        </DialogContent>
      </Dialog>
    </>
  );
}
