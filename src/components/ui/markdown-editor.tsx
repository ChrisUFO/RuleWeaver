import { useState, useCallback, useRef, useEffect, ReactNode } from "react";
import MDEditor from "@uiw/react-md-editor";
import { Maximize2, Minimize2, WrapText, Save, Loader2, AlertCircle } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

type EditorMode = "edit" | "preview" | "split";

export interface FullscreenSaveState {
  saving: boolean;
  hasUnsavedChanges: boolean;
  lastSaved: Date | null;
  autoSaveError: string | null;
  onSave: () => Promise<void>;
}

interface MarkdownEditorProps {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  className?: string;
  defaultMode?: EditorMode;
  onFullscreenChange?: (isFullscreen: boolean) => void;
  isFullscreen?: boolean;
  fullscreenSaveState?: FullscreenSaveState;
  fullscreenTitle?: string;
  fullscreenSaveStatus?: ReactNode;
}

export function MarkdownEditor({
  value,
  onChange,
  placeholder = "Write your content in Markdown...",
  className,
  defaultMode = "edit",
  onFullscreenChange,
  isFullscreen = false,
  fullscreenSaveState,
  fullscreenTitle,
  fullscreenSaveStatus,
}: MarkdownEditorProps) {
  const [mode, setMode] = useState<EditorMode>(defaultMode);
  const [wordWrap, setWordWrap] = useState(true);
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === "Escape" && isFullscreen && onFullscreenChange) {
        onFullscreenChange(false);
      }
    };
    document.addEventListener("keydown", handleEscape);
    return () => document.removeEventListener("keydown", handleEscape);
  }, [isFullscreen, onFullscreenChange]);

  const handleModeChange = useCallback((newMode: EditorMode) => {
    setMode(newMode);
  }, []);

  const toggleWordWrap = useCallback(() => {
    setWordWrap((prev) => !prev);
  }, []);

  const toggleFullscreen = useCallback(() => {
    onFullscreenChange?.(!isFullscreen);
  }, [isFullscreen, onFullscreenChange]);

  const lineCount = value.split("\n").length;
  const wordCount = value.trim() ? value.trim().split(/\s+/).length : 0;
  const charCount = value.length;

  const toolbar = (
    <div className="flex items-center gap-1 px-2 py-1.5 border-b border-white/10 bg-muted/30">
      {isFullscreen && fullscreenTitle && (
        <>
          <span className="text-sm font-medium truncate max-w-[200px]" title={fullscreenTitle}>
            {fullscreenTitle}
          </span>
          <div className="w-px h-5 bg-border mx-1" />
        </>
      )}
      <div className="flex items-center gap-0.5 p-0.5 glass border border-white/5 rounded-md">
        <Button
          variant={mode === "edit" ? "default" : "ghost"}
          size="sm"
          onClick={() => handleModeChange("edit")}
          className="h-7 px-2 text-xs"
          type="button"
          aria-pressed={mode === "edit"}
        >
          &lt;/&gt; MD
        </Button>
        <Button
          variant={mode === "split" ? "default" : "ghost"}
          size="sm"
          onClick={() => handleModeChange("split")}
          className="h-7 px-2 text-xs"
          type="button"
          aria-pressed={mode === "split"}
        >
          Split
        </Button>
        <Button
          variant={mode === "preview" ? "default" : "ghost"}
          size="sm"
          onClick={() => handleModeChange("preview")}
          className="h-7 px-2 text-xs"
          type="button"
          aria-pressed={mode === "preview"}
        >
          Preview
        </Button>
      </div>

      <div className="w-px h-5 bg-border mx-1" />

      <Button
        variant={wordWrap ? "default" : "ghost"}
        size="icon"
        onClick={toggleWordWrap}
        title={wordWrap ? "Disable word wrap" : "Enable word wrap"}
        className="h-7 w-7"
        type="button"
        aria-pressed={wordWrap}
      >
        <WrapText className="h-3.5 w-3.5" />
      </Button>

      <div className="flex-1" />

      {isFullscreen && fullscreenSaveState && (
        <>
          {fullscreenSaveState.autoSaveError && (
            <span className="flex items-center gap-1 text-xs text-destructive mr-2">
              <AlertCircle className="h-3 w-3" />
              Auto-save failed
            </span>
          )}
          {fullscreenSaveStatus}
          <Button
            variant="outline"
            size="sm"
            onClick={fullscreenSaveState.onSave}
            disabled={fullscreenSaveState.saving}
            className="h-7 px-2 text-xs mr-1"
            type="button"
          >
            {fullscreenSaveState.saving ? (
              <Loader2 className="h-3 w-3 animate-spin mr-1" />
            ) : (
              <Save className="h-3 w-3 mr-1" />
            )}
            {fullscreenSaveState.saving ? "Saving..." : "Save"}
          </Button>
        </>
      )}

      <Button
        variant="ghost"
        size="icon"
        onClick={toggleFullscreen}
        title={isFullscreen ? "Exit fullscreen" : "Fullscreen"}
        className="h-7 w-7"
        type="button"
      >
        {isFullscreen ? (
          <Minimize2 className="h-3.5 w-3.5" />
        ) : (
          <Maximize2 className="h-3.5 w-3.5" />
        )}
      </Button>
    </div>
  );

  const statsFooter = (
    <div className="flex items-center gap-4 px-3 py-2 border-t border-white/10 bg-muted/20 text-[10px] font-medium uppercase tracking-wider text-muted-foreground/60">
      <span>{wordCount} words</span>
      <span>{charCount} chars</span>
      <span>{lineCount} lines</span>
    </div>
  );

  const getPreviewMode = () => {
    switch (mode) {
      case "edit":
        return "edit" as const;
      case "preview":
        return "preview" as const;
      case "split":
        return "live" as const;
    }
  };

  return (
    <div
      ref={containerRef}
      className={cn(
        "flex flex-col h-full bg-background",
        isFullscreen && "fixed inset-0 z-50",
        className
      )}
    >
      {toolbar}

      <div
        className="flex-1 overflow-hidden"
        data-color-mode={document.documentElement.classList.contains("dark") ? "dark" : "light"}
      >
        <MDEditor
          value={value}
          onChange={(val) => onChange(val || "")}
          preview={getPreviewMode()}
          hideToolbar
          visibleDragbar={false}
          height={isFullscreen ? "100%" : undefined}
          textareaProps={{
            placeholder,
            style: {
              fontFamily: "ui-monospace, monospace",
              fontSize: "13px",
              lineHeight: "1.6",
              whiteSpace: wordWrap ? "pre-wrap" : "pre",
              overflowWrap: wordWrap ? "break-word" : "normal",
            },
          }}
          previewOptions={{
            style: {
              fontSize: "13px",
              lineHeight: "1.6",
              padding: "12px",
            },
          }}
          style={{
            height: "100%",
            border: "none",
            borderRadius: 0,
          }}
        />
      </div>

      {statsFooter}
    </div>
  );
}
