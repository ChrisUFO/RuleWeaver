import { Eye, ExternalLink } from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { type AdapterType } from "@/types/rule";
import type { ToolEntry } from "@/types/rule";

interface RulePreviewCardProps {
  enabledAdapters: AdapterType[];
  previewAdapter: AdapterType;
  onSelectPreviewAdapter: (adapter: AdapterType) => void;
  previewText: string;
  targetPath: string;
  onOpenFolder: () => void;
  tools: ToolEntry[];
}

export function RulePreviewCard({
  enabledAdapters,
  previewAdapter,
  onSelectPreviewAdapter,
  previewText,
  targetPath,
  onOpenFolder,
  tools,
}: RulePreviewCardProps) {
  return (
    <Card className="glass-card premium-shadow border-none overflow-hidden">
      <CardHeader className="bg-white/5 pb-4">
        <CardTitle className="text-sm font-semibold tracking-wide uppercase text-muted-foreground/80 flex items-center gap-2">
          <Eye className="h-4 w-4 text-primary" />
          Preview
        </CardTitle>
      </CardHeader>
      <CardContent className="pt-6">
        {enabledAdapters.length > 0 && (
          <div className="flex items-center gap-1.5 mb-4 p-1 glass border border-white/5 rounded-lg w-fit">
            {enabledAdapters.map((adapter) => (
              <Button
                key={adapter}
                variant={previewAdapter === adapter ? "default" : "ghost"}
                size="sm"
                onClick={() => onSelectPreviewAdapter(adapter)}
                className={cn(
                  "h-8 px-3 rounded-md transition-all",
                  previewAdapter === adapter ? "glow-active shadow-sm" : "text-muted-foreground"
                )}
              >
                {tools.find((a) => a.id === adapter)?.name ?? adapter}
              </Button>
            ))}
          </div>
        )}
        <pre className="p-4 rounded-xl bg-black/40 border border-white/5 text-[11px] overflow-auto max-h-60 font-mono text-primary/80 selection:bg-primary/20">
          {previewText}
        </pre>
        <div className="flex items-center justify-between mt-4">
          <p className="text-[10px] uppercase font-bold tracking-wider text-muted-foreground/40">
            Target:{" "}
            <span className="text-muted-foreground/80 lowercase font-normal">{targetPath}</span>
          </p>
          {targetPath && (
            <Button
              variant="ghost"
              size="sm"
              onClick={onOpenFolder}
              className="h-7 text-[10px] uppercase font-bold tracking-widest text-primary/60 hover:text-primary hover:bg-primary/5"
            >
              <ExternalLink className="mr-1.5 h-3 w-3" />
              Explorer
            </Button>
          )}
        </div>
      </CardContent>
    </Card>
  );
}
