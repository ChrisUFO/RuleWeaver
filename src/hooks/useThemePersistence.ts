import { useState, useEffect, useCallback } from "react";
import { api } from "@/lib/tauri";
import { featureManager, FEATURE_FLAGS } from "@/lib/featureManager";

export type Theme = "light" | "dark" | "system";

const THEME_SETTING_KEY = "ui_theme";
const VALID_THEMES: Theme[] = ["light", "dark", "system"];
const THEMES_CYCLE: Theme[] = ["light", "dark", "system"];

function isValidTheme(value: string | null | undefined): value is Theme {
  return VALID_THEMES.includes(value as Theme);
}

export function useThemePersistence() {
  const [theme, setThemeState] = useState<Theme>("system");
  const [isLoaded, setIsLoaded] = useState(false);
  const persistenceEnabled = featureManager.isEnabled(FEATURE_FLAGS.THEME_PERSISTENCE);

  useEffect(() => {
    if (!persistenceEnabled) {
      setIsLoaded(true);
      return;
    }

    api.settings
      .get(THEME_SETTING_KEY)
      .then((stored) => {
        if (isValidTheme(stored)) {
          setThemeState(stored);
        }
        // Invalid or missing → stays "system"
      })
      .catch((err) => {
        console.error("Failed to load theme setting", err);
      })
      .finally(() => {
        setIsLoaded(true);
      });
  }, [persistenceEnabled]);

  const setTheme = useCallback(
    (newTheme: Theme) => {
      setThemeState(newTheme);
      if (persistenceEnabled) {
        api.settings.set(THEME_SETTING_KEY, newTheme).catch((err) => {
          console.error("Failed to persist theme setting", err);
        });
      }
    },
    [persistenceEnabled]
  );

  const cycleTheme = useCallback(() => {
    setThemeState((prev) => {
      const idx = THEMES_CYCLE.indexOf(prev);
      const next = THEMES_CYCLE[(idx + 1) % THEMES_CYCLE.length];
      if (persistenceEnabled) {
        api.settings.set(THEME_SETTING_KEY, next).catch((err) => {
          console.error("Failed to persist theme setting", err);
        });
      }
      return next;
    });
  }, [persistenceEnabled]);

  return { theme, setTheme, cycleTheme, isLoaded };
}
