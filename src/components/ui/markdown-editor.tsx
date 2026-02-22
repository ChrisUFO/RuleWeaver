import { useState, useCallback } from "react";
import {
  Bold,
  Italic,
  Heading1,
  Heading2,
  List,
  ListOrdered,
  Code,
  Quote,
  Link,
  Undo,
  Redo,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

interface MarkdownEditorProps {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  className?: string;
}

export function MarkdownEditor({
  value,
  onChange,
  placeholder = "Write your content in Markdown...",
  className,
}: MarkdownEditorProps) {
  const [history, setHistory] = useState<string[]>([value]);
  const [historyIndex, setHistoryIndex] = useState(0);

  const updateValue = useCallback(
    (newValue: string, addToHistory = true) => {
      onChange(newValue);
      if (addToHistory) {
        const newHistory = history.slice(0, historyIndex + 1);
        newHistory.push(newValue);
        if (newHistory.length > 50) {
          newHistory.shift();
        }
        setHistory(newHistory);
        setHistoryIndex(newHistory.length - 1);
      }
    },
    [history, historyIndex, onChange]
  );

  const undo = useCallback(() => {
    if (historyIndex > 0) {
      const newIndex = historyIndex - 1;
      setHistoryIndex(newIndex);
      onChange(history[newIndex]);
    }
  }, [history, historyIndex, onChange]);

  const redo = useCallback(() => {
    if (historyIndex < history.length - 1) {
      const newIndex = historyIndex + 1;
      setHistoryIndex(newIndex);
      onChange(history[newIndex]);
    }
  }, [history, historyIndex, onChange]);

  const insertText = useCallback(
    (before: string, after = "") => {
      const textarea = document.querySelector(".markdown-editor-textarea") as HTMLTextAreaElement;
      if (!textarea) return;

      const start = textarea.selectionStart;
      const end = textarea.selectionEnd;
      const selectedText = value.substring(start, end);
      const newText =
        value.substring(0, start) + before + selectedText + after + value.substring(end);

      updateValue(newText);

      setTimeout(() => {
        textarea.focus();
        const newCursorPos = start + before.length + selectedText.length;
        textarea.setSelectionRange(newCursorPos, newCursorPos);
      }, 0);
    },
    [value, updateValue]
  );

  const insertAtLineStart = useCallback(
    (prefix: string) => {
      const textarea = document.querySelector(".markdown-editor-textarea") as HTMLTextAreaElement;
      if (!textarea) return;

      const start = textarea.selectionStart;
      const lineStart = value.lastIndexOf("\n", start - 1) + 1;
      const newText = value.substring(0, lineStart) + prefix + value.substring(lineStart);

      updateValue(newText);

      setTimeout(() => {
        textarea.focus();
        textarea.setSelectionRange(start + prefix.length, start + prefix.length);
      }, 0);
    },
    [value, updateValue]
  );

  const toolbarItems = [
    { icon: Bold, action: () => insertText("**", "**"), title: "Bold" },
    { icon: Italic, action: () => insertText("*", "*"), title: "Italic" },
    {
      icon: Heading1,
      action: () => insertAtLineStart("# "),
      title: "Heading 1",
    },
    {
      icon: Heading2,
      action: () => insertAtLineStart("## "),
      title: "Heading 2",
    },
    {
      icon: List,
      action: () => insertAtLineStart("- "),
      title: "Bullet List",
    },
    {
      icon: ListOrdered,
      action: () => insertAtLineStart("1. "),
      title: "Numbered List",
    },
    { icon: Code, action: () => insertText("`", "`"), title: "Inline Code" },
    {
      icon: Quote,
      action: () => insertAtLineStart("> "),
      title: "Quote",
    },
    {
      icon: Link,
      action: () => insertText("[", "](url)"),
      title: "Link",
    },
  ];

  return (
    <div className={cn("flex flex-col h-full", className)}>
      <div className="flex items-center gap-1 p-2 border-b bg-muted/30">
        {toolbarItems.map((item) => (
          <Button
            key={item.title}
            variant="ghost"
            size="icon"
            className="h-8 w-8"
            onClick={item.action}
            title={item.title}
            aria-label={item.title}
            type="button"
          >
            <item.icon className="h-4 w-4" aria-hidden="true" />
          </Button>
        ))}
        <div className="w-px h-6 bg-border mx-1" />
        <Button
          variant="ghost"
          size="icon"
          className="h-8 w-8"
          onClick={undo}
          disabled={historyIndex <= 0}
          title="Undo"
          aria-label="Undo"
          type="button"
        >
          <Undo className="h-4 w-4" aria-hidden="true" />
        </Button>
        <Button
          variant="ghost"
          size="icon"
          className="h-8 w-8"
          onClick={redo}
          disabled={historyIndex >= history.length - 1}
          title="Redo"
          aria-label="Redo"
          type="button"
        >
          <Redo className="h-4 w-4" />
        </Button>
      </div>
      <textarea
        className="markdown-editor-textarea flex-1 w-full resize-none p-3 text-sm bg-transparent focus:outline-none font-mono"
        value={value}
        onChange={(e) => updateValue(e.target.value)}
        placeholder={placeholder}
        spellCheck={false}
      />
    </div>
  );
}
