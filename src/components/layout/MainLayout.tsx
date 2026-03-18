import * as React from "react";
import { Sidebar } from "./Sidebar";
import { Header } from "./Header";
import { useThemePersistence } from "@/hooks/useThemePersistence";

interface MainLayoutProps {
  children: React.ReactNode;
  activeView: string;
  onViewChange: (view: string) => void;
}

export function MainLayout({ children, activeView, onViewChange }: MainLayoutProps) {
  const [sidebarCollapsed, setSidebarCollapsed] = React.useState(false);
  const { theme, setTheme, isLoaded } = useThemePersistence();

  React.useEffect(() => {
    const root = window.document.documentElement;

    const applyTheme = (t: typeof theme) => {
      root.classList.remove("light", "dark");
      if (t === "system") {
        const systemTheme = window.matchMedia("(prefers-color-scheme: dark)").matches
          ? "dark"
          : "light";
        root.classList.add(systemTheme);
      } else {
        root.classList.add(t);
      }
    };

    applyTheme(theme);

    if (theme !== "system") return;

    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const handleChange = () => applyTheme("system");
    mq.addEventListener("change", handleChange);
    return () => mq.removeEventListener("change", handleChange);
  }, [theme]);

  return (
    <div
      className="flex h-screen w-full overflow-hidden bg-background relative"
      style={isLoaded ? undefined : { visibility: "hidden" }}
    >
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
        <main className="flex-1 overflow-y-auto overflow-x-hidden p-6">{children}</main>
      </div>
    </div>
  );
}
