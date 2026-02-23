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
  { key: "n", ctrl: true, shift: true, label: "New Command" },
  { key: "s", ctrl: true, label: "Save" },
  { key: "s", ctrl: true, shift: true, label: "Sync All" },
  { key: "f", ctrl: true, label: "Search" },
  { key: "1", ctrl: true, label: "Dashboard" },
  { key: "2", ctrl: true, label: "Rules" },
  { key: "3", ctrl: true, label: "Commands" },
  { key: "4", ctrl: true, label: "Skills" },
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
      <DialogContent className="max-w-md glass-card bg-neutral-900/90 border-white/5 premium-shadow">
        <DialogHeader>
          <DialogTitle className="text-sm font-semibold tracking-wide uppercase text-muted-foreground/80">
            Keyboard Shortcuts
          </DialogTitle>
        </DialogHeader>
        <div className="space-y-1 py-4">
          {shortcutList.map((shortcut, index) => (
            <div
              key={index}
              className="flex items-center justify-between py-2.5 px-3 rounded-xl transition-colors hover:bg-white/5"
            >
              <span className="text-sm font-medium text-foreground/80">{shortcut.label}</span>
              <kbd className="px-2 py-1 text-[10px] font-mono bg-white/5 text-primary/80 rounded border border-white/10 uppercase tracking-widest shadow-inner">
                {formatShortcut(shortcut)}
              </kbd>
            </div>
          ))}
        </div>
      </DialogContent>
    </Dialog>
  );
}
