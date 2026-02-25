import * as React from "react";
import { Sidebar } from "./Sidebar";
import { Header } from "./Header";

type Theme = "light" | "dark" | "system";

interface MainLayoutProps {
  children: React.ReactNode;
  activeView: string;
  onViewChange: (view: string) => void;
}

export function MainLayout({ children, activeView, onViewChange }: MainLayoutProps) {
  const [sidebarCollapsed, setSidebarCollapsed] = React.useState(false);
  const [theme, setTheme] = React.useState<Theme>("system");

  React.useEffect(() => {
    const root = window.document.documentElement;
    root.classList.remove("light", "dark");

    if (theme === "system") {
      const systemTheme = window.matchMedia("(prefers-color-scheme: dark)").matches
        ? "dark"
        : "light";
      root.classList.add(systemTheme);
    } else {
      root.classList.add(theme);
    }
  }, [theme]);

  return (
    <div className="flex h-screen w-full overflow-hidden bg-background relative">
      {/* Luminescent Breeze Background */}
      <div className="absolute inset-0 overflow-hidden pointer-events-none z-0">
        <div className="absolute top-[-10%] left-[-10%] w-[120%] h-[120%] bg-[radial-gradient(circle_at_50%_0%,rgba(59,130,246,0.08),transparent_50%),radial-gradient(circle_at_100%_100%,rgba(147,51,234,0.05),transparent_50%)] animate-breeze" />
      </div>

      <Sidebar
        collapsed={sidebarCollapsed}
        onCollapsedChange={setSidebarCollapsed}
        activeView={activeView}
        onViewChange={onViewChange}
      />
      <div className="flex flex-1 flex-col overflow-hidden relative z-10">
        <Header theme={theme} onThemeChange={setTheme} />
        <main className="flex-1 overflow-auto p-6">{children}</main>
      </div>
    </div>
  );
}
