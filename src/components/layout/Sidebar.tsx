import { cn } from "@/lib/utils";
import {
  LayoutDashboard,
  FileText,
  Settings,
  ChevronLeft,
  ChevronRight,
  Terminal,
  Brain,
} from "lucide-react";

interface SidebarProps {
  collapsed: boolean;
  onCollapsedChange: (collapsed: boolean) => void;
  activeView: string;
  onViewChange: (view: string) => void;
}

const navItems = [
  { id: "dashboard", label: "Dashboard", icon: LayoutDashboard },
  { id: "rules", label: "Rules", icon: FileText },
  { id: "commands", label: "Commands", icon: Terminal },
  { id: "skills", label: "Skills", icon: Brain },
  { id: "settings", label: "Settings", icon: Settings },
];

export function Sidebar({ collapsed, onCollapsedChange, activeView, onViewChange }: SidebarProps) {
  return (
    <aside
      className={cn(
        "flex h-full flex-col border-r transition-all duration-500 ease-[cubic-bezier(0.4,0,0.2,1)]",
        "glass border-white/5 premium-shadow",
        collapsed ? "w-16" : "w-64"
      )}
      aria-label="Main navigation"
      aria-expanded={!collapsed}
    >
      <div className="flex h-14 items-center justify-between border-b border-white/10 px-4">
        <div className="flex items-center gap-2 overflow-hidden">
          <img
            src="/logo.svg"
            alt="RuleWeaver Logo"
            className="h-8 w-8 shrink-0 rounded-lg shadow-lg glow-primary"
          />
          {!collapsed && (
            <span className="font-bold text-lg tracking-tight truncate bg-clip-text text-transparent bg-gradient-to-br from-foreground to-foreground/60">
              RuleWeaver
            </span>
          )}
        </div>
        <button
          onClick={() => onCollapsedChange(!collapsed)}
          className="p-1.5 rounded-md hover:bg-white/5 focus:outline-none focus:ring-2 focus:ring-primary/40 transition-all duration-200"
          aria-label={collapsed ? "Expand sidebar" : "Collapse sidebar"}
          aria-expanded={!collapsed}
        >
          {collapsed ? (
            <ChevronRight className="h-4 w-4 text-muted-foreground" aria-hidden="true" />
          ) : (
            <ChevronLeft className="h-4 w-4 text-muted-foreground" aria-hidden="true" />
          )}
        </button>
      </div>

      <nav className="flex-1 space-y-1 p-3" role="navigation" aria-label="Primary">
        {navItems.map((item) => (
          <button
            key={item.id}
            onClick={() => onViewChange(item.id)}
            className={cn(
              "flex w-full items-center gap-3 rounded-lg px-3 py-2.5 text-sm font-medium transition-all duration-200 group",
              activeView === item.id
                ? "bg-primary/10 text-primary glow-active border border-primary/20"
                : "text-muted-foreground hover:bg-white/5 hover:text-foreground border border-transparent"
            )}
            aria-current={activeView === item.id ? "page" : undefined}
            aria-label={collapsed ? item.label : undefined}
          >
            <item.icon
              className={cn(
                "h-4 w-4 shrink-0 transition-transform duration-200 group-hover:scale-110",
                activeView === item.id ? "text-primary" : "text-muted-foreground/80"
              )}
              aria-hidden="true"
            />
            {!collapsed && <span>{item.label}</span>}
          </button>
        ))}
      </nav>

      <div className="border-t border-white/5 p-4">
        {!collapsed && (
          <div className="px-3 py-2 text-[10px] font-bold uppercase tracking-widest text-muted-foreground/50">
            Version 0.1.0
          </div>
        )}
      </div>
    </aside>
  );
}
