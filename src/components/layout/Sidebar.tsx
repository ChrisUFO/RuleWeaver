import { cn } from "@/lib/utils";
import { LayoutDashboard, FileText, Settings, ChevronLeft, ChevronRight } from "lucide-react";

interface SidebarProps {
  collapsed: boolean;
  onCollapsedChange: (collapsed: boolean) => void;
  activeView: string;
  onViewChange: (view: string) => void;
}

const navItems = [
  { id: "dashboard", label: "Dashboard", icon: LayoutDashboard },
  { id: "rules", label: "Rules", icon: FileText },
  { id: "settings", label: "Settings", icon: Settings },
];

export function Sidebar({ collapsed, onCollapsedChange, activeView, onViewChange }: SidebarProps) {
  return (
    <aside
      className={cn(
        "flex h-full flex-col border-r bg-card transition-all duration-300",
        collapsed ? "w-16" : "w-56"
      )}
      aria-label="Main navigation"
      aria-expanded={!collapsed}
    >
      <div className="flex h-14 items-center justify-between border-b px-4">
        {!collapsed && <span className="font-semibold text-lg">RuleWeaver</span>}
        <button
          onClick={() => onCollapsedChange(!collapsed)}
          className="p-1.5 rounded-md hover:bg-accent focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2"
          aria-label={collapsed ? "Expand sidebar" : "Collapse sidebar"}
          aria-expanded={!collapsed}
        >
          {collapsed ? (
            <ChevronRight className="h-4 w-4" aria-hidden="true" />
          ) : (
            <ChevronLeft className="h-4 w-4" aria-hidden="true" />
          )}
        </button>
      </div>

      <nav className="flex-1 space-y-1 p-2" role="navigation" aria-label="Primary">
        {navItems.map((item) => (
          <button
            key={item.id}
            onClick={() => onViewChange(item.id)}
            className={cn(
              "flex w-full items-center gap-3 rounded-md px-3 py-2 text-sm font-medium transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2",
              activeView === item.id
                ? "bg-primary text-primary-foreground"
                : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
            )}
            aria-current={activeView === item.id ? "page" : undefined}
            aria-label={collapsed ? item.label : undefined}
          >
            <item.icon className="h-4 w-4 shrink-0" aria-hidden="true" />
            {!collapsed && <span>{item.label}</span>}
          </button>
        ))}
      </nav>

      <div className="border-t p-2">
        {!collapsed && <div className="px-3 py-2 text-xs text-muted-foreground">Version 0.1.0</div>}
      </div>
    </aside>
  );
}
