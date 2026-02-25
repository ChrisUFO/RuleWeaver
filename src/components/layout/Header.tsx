import { Sun, Moon, Monitor } from "lucide-react";
import { Button } from "@/components/ui/button";
import { motion, AnimatePresence } from "framer-motion";

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
    <header className="flex h-14 items-center justify-end border-b border-white/5 glass px-4 sticky top-0 z-50">
      <div className="flex items-center gap-2">
        <AnimatePresence mode="wait">
          <motion.div
            key={theme}
            initial={{ opacity: 0, rotate: -20, scale: 0.8 }}
            animate={{ opacity: 1, rotate: 0, scale: 1 }}
            exit={{ opacity: 0, rotate: 20, scale: 0.8 }}
            transition={{ duration: 0.2 }}
          >
            <Button
              variant="ghost"
              size="icon"
              onClick={cycleTheme}
              className="rounded-full hover:bg-primary/10 text-muted-foreground hover:text-primary transition-all shadow-glow-active"
              aria-label="Toggle theme"
            >
              <ThemeIcon className="h-4 w-4" />
            </Button>
          </motion.div>
        </AnimatePresence>
      </div>
    </header>
  );
}
