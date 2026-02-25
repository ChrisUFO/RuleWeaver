import { useState, useEffect } from "react";
import { Library, Search, SearchX } from "lucide-react";
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
import { useToast } from "@/components/ui/toast";
import { cn } from "@/lib/utils";

interface BaseTemplateBrowserProps<T> {
  title: string;
  description: string;
  onInstalled: () => void;
  getTemplates: () => Promise<T[]>;
  installTemplate: (id: string) => Promise<unknown>;
  getName: (template: T) => string;
  getDescription: (template: T) => string;
  getTheme: (template: T) => string;
  getTemplateId: (template: T) => string;
  getSearchableContent: (template: T) => string[];
  renderDetail: (
    template: T,
    install: (id: string) => void,
    isInstalling: boolean,
    onBack: () => void
  ) => React.ReactNode;
}

export function BaseTemplateBrowser<T>({
  title,
  description,
  onInstalled,
  getTemplates,
  installTemplate,
  getName,
  getDescription,
  getTheme,
  getTemplateId,
  getSearchableContent,
  renderDetail,
}: BaseTemplateBrowserProps<T>) {
  const [templates, setTemplates] = useState<T[]>([]);
  const [isOpen, setIsOpen] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [installingId, setInstallingId] = useState<string | null>(null);
  const [selectedTemplate, setSelectedTemplate] = useState<T | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const { addToast } = useToast();

  useEffect(() => {
    if (isOpen) {
      setIsLoading(true);
      getTemplates()
        .then(setTemplates)
        .catch((e) => {
          console.error(`Failed to load ${title}:`, e);
          addToast({
            title: `Failed to load ${title}`,
            description: String(e),
            variant: "error",
          });
        })
        .finally(() => setIsLoading(false));
    } else {
      setSelectedTemplate(null);
    }
  }, [isOpen, addToast, getTemplates, title]);

  const install = async (id: string) => {
    setInstallingId(id);
    try {
      await installTemplate(id);
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
            renderDetail(
              selectedTemplate,
              install,
              installingId === getTemplateId(selectedTemplate),
              () => setSelectedTemplate(null)
            )
          ) : (
            <>
              <DialogHeader>
                <DialogTitle>{title}</DialogTitle>
                <DialogDescription>{description}</DialogDescription>
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
                  ) : (
                    (() => {
                      const query = searchQuery.toLowerCase();
                      const filtered = templates.filter((t) => {
                        const content = [
                          getName(t),
                          getDescription(t),
                          getTheme(t),
                          ...getSearchableContent(t),
                        ].map((s) => s.toLowerCase());
                        return content.some((s) => s.includes(query));
                      });

                      const groups = filtered.reduce(
                        (acc, t) => {
                          const theme = getTheme(t) || "General";
                          if (!acc[theme]) acc[theme] = [];
                          acc[theme].push(t);
                          return acc;
                        },
                        {} as Record<string, T[]>
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
                                    key={getTemplateId(t)}
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
                                        {getName(t)}
                                      </h4>
                                      <p className="text-xs text-muted-foreground mt-1 line-clamp-1">
                                        {getDescription(t)}
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
