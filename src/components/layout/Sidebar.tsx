import { useEffect, useState } from "react";
import { cn } from "@/lib/utils";
import { motion, AnimatePresence } from "framer-motion";
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
  const [appVersion, setAppVersion] = useState<string>("0.0.0");

  useEffect(() => {
    fetch("/version.json")
      .then((res) => res.json())
      .then((data) => {
        setAppVersion(data.version || "0.0.0");
      })
      .catch(() => {
        setAppVersion("dev"); // Fallback for development
      });
  }, []);

  return (
    <motion.aside
      initial={false}
      animate={{ width: collapsed ? 64 : 256 }}
      transition={{ type: "spring", stiffness: 300, damping: 30 }}
      className={cn(
        "flex h-full flex-col border-r relative z-10",
        "glass border-white/5 shadow-2xl overflow-hidden"
      )}
      aria-label="Main navigation"
    >
      {/* Header */}
      <div className="flex h-14 items-center justify-between border-b border-white/10 px-4">
        <div className="flex items-center gap-2 overflow-hidden">
          <div className="relative shrink-0">
            <img
              src="/logo.svg"
              alt="RuleWeaver"
              className="h-8 w-8 rounded-lg shadow-lg glow-primary relative z-10"
            />
            <div className="absolute inset-0 bg-primary/20 blur-md rounded-full animate-pulse" />
          </div>
          <AnimatePresence>
            {!collapsed && (
              <motion.span
                initial={{ opacity: 0, x: -10 }}
                animate={{ opacity: 1, x: 0 }}
                exit={{ opacity: 0, x: -10 }}
                className="font-bold text-lg tracking-tight truncate luminescent-text"
              >
                RuleWeaver
              </motion.span>
            )}
          </AnimatePresence>
        </div>
        <button
          onClick={() => onCollapsedChange(!collapsed)}
          className="p-1.5 rounded-md hover:bg-white/5 text-muted-foreground hover:text-foreground transition-colors"
          aria-label={collapsed ? "Expand sidebar" : "Collapse sidebar"}
          aria-expanded={!collapsed}
        >
          {collapsed ? <ChevronRight className="h-4 w-4" /> : <ChevronLeft className="h-4 w-4" />}
        </button>
      </div>

      {/* Navigation */}
      <nav className="flex-1 space-y-1 p-3">
        {navItems.map((item) => {
          const isActive = activeView === item.id;
          return (
            <button
              key={item.id}
              onClick={() => onViewChange(item.id)}
              className={cn(
                "flex w-full items-center gap-3 rounded-xl px-3 py-2.5 text-sm font-medium transition-all duration-300 relative group",
                isActive
                  ? "text-primary"
                  : "text-muted-foreground hover:bg-white/5 hover:text-foreground"
              )}
              aria-current={isActive ? "page" : undefined}
              aria-label={collapsed ? item.label : undefined}
            >
              {isActive && (
                <motion.div
                  layoutId="active-nav"
                  className="absolute inset-0 bg-primary/10 rounded-xl border border-primary/20 shadow-glow-active"
                  transition={{ type: "spring", stiffness: 400, damping: 35 }}
                />
              )}

              <item.icon
                className={cn(
                  "h-4 w-4 shrink-0 relative z-10 transition-transform duration-300 group-hover:scale-110",
                  isActive ? "text-primary" : "text-muted-foreground/80"
                )}
              />

              <AnimatePresence>
                {!collapsed && (
                  <motion.span
                    initial={{ opacity: 0, x: -5 }}
                    animate={{ opacity: 1, x: 0 }}
                    exit={{ opacity: 0, x: -5 }}
                    className="relative z-10 truncate"
                  >
                    {item.label}
                  </motion.span>
                )}
              </AnimatePresence>

              {collapsed && (
                <div className="absolute left-14 px-2 py-1 bg-popover text-popover-foreground text-xs rounded opacity-0 group-hover:opacity-100 pointer-events-none transition-opacity shadow-lg border border-white/5 whitespace-nowrap z-50">
                  {item.label}
                </div>
              )}
            </button>
          );
        })}
      </nav>

      {/* Footer Version & Help */}
      <motion.div
        layout
        className={cn(
          "mt-auto border-t border-white/5 p-4 space-y-2",
          collapsed ? "items-center" : ""
        )}
      >
        <div className="flex flex-col gap-1">
          <AnimatePresence mode="wait">
            {!collapsed && (
              <motion.span
                initial={{ opacity: 0, y: 5 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: 5 }}
                className="text-[10px] font-bold text-muted-foreground/40 uppercase tracking-widest block"
              >
                Press ? for help
              </motion.span>
            )}
          </AnimatePresence>
          <motion.div
            initial={false}
            animate={{ opacity: 1 }}
            className="flex items-center gap-2 text-xs text-muted-foreground/60"
          >
            <div className="h-2 w-2 rounded-full bg-emerald-500/50 animate-pulse" />
            {!collapsed && <span>v{appVersion}</span>}
          </motion.div>
        </div>
      </motion.div>
    </motion.aside>
  );
}
