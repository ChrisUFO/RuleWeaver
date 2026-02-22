import { Sun, Moon, Monitor } from "lucide-react";
import { Button } from "@/components/ui/button";

type Theme = "light" | "dark" | "system";

interface HeaderProps {
  theme: Theme;
  onThemeChange: (theme: Theme) => void;
}

export function Header({ theme, onThemeChange }: HeaderProps) {
  const cycleTheme = () => {
    const themes: Theme[] = ["light", "dark", "system"];
    const currentIndex = themes.indexOf(theme);
    const nextIndex = (currentIndex + 1) % themes.length;
    onThemeChange(themes[nextIndex]);
  };

  const ThemeIcon = theme === "light" ? Sun : theme === "dark" ? Moon : Monitor;

  return (
    <header className="flex h-14 items-center justify-end border-b bg-card px-4">
      <div className="flex items-center gap-2">
        <Button variant="ghost" size="icon" onClick={cycleTheme} aria-label="Toggle theme">
          <ThemeIcon className="h-4 w-4" />
        </Button>
      </div>
    </header>
  );
}
