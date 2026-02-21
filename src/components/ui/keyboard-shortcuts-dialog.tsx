import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog";

interface KeyboardShortcutsDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

interface ShortcutItem {
  key: string;
  ctrl?: boolean;
  shift?: boolean;
  alt?: boolean;
  label: string;
}

const shortcutList: ShortcutItem[] = [
  { key: "n", ctrl: true, label: "New Rule" },
  { key: "s", ctrl: true, label: "Save" },
  { key: "s", ctrl: true, shift: true, label: "Sync All" },
  { key: "f", ctrl: true, label: "Search" },
  { key: ",", ctrl: true, label: "Settings" },
  { key: "Escape", label: "Close/Cancel" },
];

function formatShortcut(shortcut: ShortcutItem): string {
  const parts: string[] = [];
  if (shortcut.ctrl) parts.push("Ctrl");
  if (shortcut.shift) parts.push("Shift");
  if (shortcut.alt) parts.push("Alt");
  parts.push(shortcut.key === "Escape" ? "Esc" : shortcut.key.toUpperCase());
  return parts.join(" + ");
}

export function KeyboardShortcutsDialog({ open, onOpenChange }: KeyboardShortcutsDialogProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle>Keyboard Shortcuts</DialogTitle>
        </DialogHeader>
        <div className="space-y-2 py-4">
          {shortcutList.map((shortcut, index) => (
            <div key={index} className="flex items-center justify-between py-2">
              <span className="text-sm">{shortcut.label}</span>
              <kbd className="px-2 py-1 text-xs font-mono bg-muted rounded border">
                {formatShortcut(shortcut)}
              </kbd>
            </div>
          ))}
        </div>
      </DialogContent>
    </Dialog>
  );
}
