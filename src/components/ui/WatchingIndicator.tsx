import { Eye } from "lucide-react";
import { cn } from "@/lib/utils";

interface WatchingIndicatorProps {
  path?: string;
  paths?: string[];
  justRefreshed: boolean;
  className?: string;
}

export function WatchingIndicator({
  path,
  paths,
  justRefreshed,
  className,
}: WatchingIndicatorProps) {
  const tooltipContent = paths
    ? `MCP is watching local paths for this artifact\n${paths.join("\n")}`
    : path
      ? `MCP is watching this artifact directory\n${path}`
      : "MCP is watching for changes";

  return (
    <span title={tooltipContent} className={className}>
      <Eye
        className={cn(
          "h-3.5 w-3.5 text-blue-500 transition-all duration-500",
          justRefreshed ? "text-emerald-400 scale-125 glow-active drop-shadow-md" : "animate-pulse"
        )}
      />
    </span>
  );
}
