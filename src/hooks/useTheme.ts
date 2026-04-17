import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { ThemeId, DarkMode } from "../lib/tauri-commands";

function applyToDOM(theme: ThemeId, isDark: boolean) {
  const html = document.documentElement;
  html.classList.remove("theme-amber", "theme-teal", "theme-violet");
  html.classList.add(`theme-${theme}`);
  html.classList.toggle("dark", isDark);
}

function resolveIsDark(mode: DarkMode): boolean {
  if (mode === "dark") return true;
  if (mode === "light") return false;
  return window.matchMedia("(prefers-color-scheme: dark)").matches;
}

export function useTheme() {
  const [theme, setThemeState] = useState<ThemeId>("amber");
  const [darkMode, setDarkModeState] = useState<DarkMode>("system");
  const [isDark, setIsDark] = useState(false);

  // On mount, read config and apply
  useEffect(() => {
    invoke<{ theme: ThemeId; darkMode: DarkMode }>("get_config").then(
      (config) => {
        const t = config.theme || "amber";
        const dm = config.darkMode || "system";
        setThemeState(t);
        setDarkModeState(dm);
        const dark = resolveIsDark(dm);
        setIsDark(dark);
        applyToDOM(t, dark);
      },
    );
  }, []);

  // Listen for system preference changes when in "system" mode
  useEffect(() => {
    if (darkMode !== "system") return;

    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const handler = (e: MediaQueryListEvent) => {
      setIsDark(e.matches);
      applyToDOM(theme, e.matches);
    };
    mq.addEventListener("change", handler);
    return () => mq.removeEventListener("change", handler);
  }, [darkMode, theme]);

  const setTheme = useCallback(
    (t: ThemeId) => {
      setThemeState(t);
      applyToDOM(t, isDark);
      invoke("set_theme", { theme: t });
    },
    [isDark],
  );

  const setDarkModeChoice = useCallback(
    (mode: DarkMode) => {
      setDarkModeState(mode);
      const dark = resolveIsDark(mode);
      setIsDark(dark);
      applyToDOM(theme, dark);
      invoke("set_dark_mode", { mode });
    },
    [theme],
  );

  return { theme, darkMode, isDark, setTheme, setDarkMode: setDarkModeChoice };
}
