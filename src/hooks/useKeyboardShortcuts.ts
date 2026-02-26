import { useEffect, useCallback } from "react";

type KeyboardShortcut = {
  key: string;
  ctrl?: boolean;
  shift?: boolean;
  alt?: boolean;
  action: () => void;
  description?: string;
};

interface UseKeyboardShortcutsOptions {
  shortcuts: KeyboardShortcut[];
  enabled?: boolean;
}

export function useKeyboardShortcuts({ shortcuts, enabled = true }: UseKeyboardShortcutsOptions) {
  const handleKeyDown = useCallback(
    (event: KeyboardEvent) => {
      if (!enabled) return;

      if (event.target instanceof HTMLInputElement || event.target instanceof HTMLTextAreaElement) {
        const isInputShortcut = shortcuts.some(
          (s) => s.key.toLowerCase() === event.key.toLowerCase() && (s.ctrl || s.shift || s.alt)
        );
        if (!isInputShortcut) return;
      }

      for (const shortcut of shortcuts) {
        const keyMatch = shortcut.key.toLowerCase() === event.key.toLowerCase();
        const ctrlMatch = shortcut.ctrl ? event.ctrlKey || event.metaKey : true;
        const shiftMatch = shortcut.shift ? event.shiftKey : !event.shiftKey;
        const altMatch = shortcut.alt ? event.altKey : !event.altKey;

        if (keyMatch && ctrlMatch && shiftMatch && altMatch) {
          event.preventDefault();
          shortcut.action();
          return;
        }
      }
    },
    [shortcuts, enabled]
  );

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [handleKeyDown]);
}

export const SHORTCUTS = {
  NEW_RULE: { key: "n", ctrl: true, description: "Create new rule" },
  NEW_COMMAND: { key: "n", ctrl: true, shift: true, description: "Create new command" },
  SAVE: { key: "s", ctrl: true, description: "Save current rule" },
  SYNC: { key: "s", ctrl: true, shift: true, description: "Sync all rules" },
  SEARCH: { key: "f", ctrl: true, description: "Focus search" },
  SETTINGS: { key: ",", ctrl: true, description: "Open settings" },
  DASHBOARD: { key: "1", ctrl: true, description: "Go to dashboard" },
  RULES: { key: "2", ctrl: true, description: "Go to rules" },
  COMMANDS: { key: "3", ctrl: true, description: "Go to commands" },
  SKILLS: { key: "4", ctrl: true, description: "Go to skills" },
  STATUS: { key: "5", ctrl: true, description: "Go to status" },
  HELP: { key: "?", shift: true, description: "Show keyboard shortcuts" },
  ESCAPE: { key: "Escape", description: "Close dialog/cancel" },
} as const;
