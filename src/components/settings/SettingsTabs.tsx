import * as React from "react";
import { motion } from "framer-motion";
import { cn } from "@/lib/utils";

interface Tab {
  id: string;
  label: string;
  icon: React.ElementType;
}

interface SettingsTabsProps {
  tabs: Tab[];
  activeTab: string;
  onTabChange: (id: string) => void;
}

export function SettingsTabs({ tabs, activeTab, onTabChange }: SettingsTabsProps) {
  return (
    <div className="flex items-center gap-1 p-1 rounded-2xl glass border-white/5 bg-white/5 mb-8 w-fit">
      {tabs.map((tab) => {
        const isActive = activeTab === tab.id;
        return (
          <button
            key={tab.id}
            onClick={() => onTabChange(tab.id)}
            className={cn(
              "relative flex items-center gap-2 px-4 py-2 text-xs font-bold uppercase tracking-wider transition-colors duration-300 rounded-xl outline-none",
              isActive ? "text-primary" : "text-muted-foreground hover:text-foreground"
            )}
          >
            {isActive && (
              <motion.div
                layoutId="settings-active-tab"
                className="absolute inset-0 bg-white/10 shadow-glow-active rounded-xl z-0"
                transition={{ type: "spring", stiffness: 350, damping: 30 }}
              />
            )}
            <tab.icon className={cn("h-3.5 w-3.5 relative z-10", isActive ? "text-primary" : "")} />
            <span className="relative z-10">{tab.label}</span>
          </button>
        );
      })}
    </div>
  );
}
