import { Button } from "@/components/ui/button";

interface BulkActionsBarProps {
  selectedCount: number;
  onEnableAll: () => void;
  onDisableAll: () => void;
  onDeleteAll: () => void;
  onCancel: () => void;
}

export function BulkActionsBar({
  selectedCount,
  onEnableAll,
  onDisableAll,
  onDeleteAll,
  onCancel,
}: BulkActionsBarProps) {
  if (selectedCount === 0) return null;

  return (
    <div
      className="flex items-center gap-3 p-3 bg-accent/50 rounded-md border"
      role="toolbar"
      aria-label="Bulk actions"
    >
      <span className="text-sm text-muted-foreground">{selectedCount} selected</span>
      <Button variant="outline" size="sm" onClick={onEnableAll}>
        Enable All
      </Button>
      <Button variant="outline" size="sm" onClick={onDisableAll}>
        Disable All
      </Button>
      <Button variant="destructive" size="sm" onClick={onDeleteAll}>
        Delete All
      </Button>
      <Button variant="ghost" size="sm" onClick={onCancel}>
        Cancel
      </Button>
    </div>
  );
}
