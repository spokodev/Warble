import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
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

function loadAndApplyTheme(
  setThemeState: (t: ThemeId) => void,
  setDarkModeState: (m: DarkMode) => void,
  setIsDark: (d: boolean) => void,
) {
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
}

export function useTheme() {
  const [theme, setThemeState] = useState<ThemeId>("amber");
  const [darkMode, setDarkModeState] = useState<DarkMode>("system");
  const [isDark, setIsDark] = useState(false);

  // On mount, read config and apply
  useEffect(() => {
    loadAndApplyTheme(setThemeState, setDarkModeState, setIsDark);
  }, []);

  // Listen for theme changes from other windows (e.g. settings → status bar)
  useEffect(() => {
    let unsub: (() => void) | undefined;
    listen("theme-changed", () => {
      loadAndApplyTheme(setThemeState, setDarkModeState, setIsDark);
    }).then((u) => { unsub = u; });
    return () => unsub?.();
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
