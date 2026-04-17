import type { ThemeId, DarkMode } from "../../lib/tauri-commands";

interface ThemePickerProps {
  theme: ThemeId;
  darkMode: DarkMode;
  onThemeChange: (theme: ThemeId) => void;
  onDarkModeChange: (mode: DarkMode) => void;
}

const THEMES: { id: ThemeId; name: string; color: string }[] = [
  { id: "amber", name: "Dawn", color: "#f59e0b" },
  { id: "teal", name: "Ocean", color: "#14b8a6" },
  { id: "violet", name: "Nebula", color: "#a855f7" },
];

const MODES: { id: DarkMode; label: string; icon: string }[] = [
  { id: "light", label: "Light", icon: "sun" },
  { id: "system", label: "System", icon: "monitor" },
  { id: "dark", label: "Dark", icon: "moon" },
];

function SunIcon() {
  return (
    <svg
      width="14"
      height="14"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
    >
      <circle cx="12" cy="12" r="5" />
      <path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42" />
    </svg>
  );
}

function MonitorIcon() {
  return (
    <svg
      width="14"
      height="14"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <rect x="2" y="3" width="20" height="14" rx="2" />
      <path d="M8 21h8M12 17v4" />
    </svg>
  );
}

function MoonIcon() {
  return (
    <svg
      width="14"
      height="14"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
    </svg>
  );
}

const MODE_ICONS = { sun: SunIcon, monitor: MonitorIcon, moon: MoonIcon };

export function ThemePicker({
  theme,
  darkMode,
  onThemeChange,
  onDarkModeChange,
}: ThemePickerProps) {
  return (
    <div className="space-y-3">
      <label className="block font-medium text-sm text-text-primary">
        Appearance
      </label>

      {/* Theme swatches */}
      <div className="flex gap-2">
        {THEMES.map((t) => (
          <button
            key={t.id}
            onClick={() => onThemeChange(t.id)}
            className={`flex items-center gap-2 px-3 py-2 rounded-lg text-xs font-medium transition-all ${
              theme === t.id
                ? "bg-brand-100 text-brand-700 ring-2 ring-brand-500/30 dark:bg-brand-900/30 dark:text-brand-300"
                : "bg-surface-inset text-text-secondary hover:text-text-primary"
            }`}
          >
            <div
              className="w-3 h-3 rounded-full"
              style={{ background: t.color }}
            />
            {t.name}
          </button>
        ))}
      </div>

      {/* Dark mode toggle */}
      <div className="flex rounded-lg overflow-hidden border border-border">
        {MODES.map((m) => {
          const Icon = MODE_ICONS[m.icon as keyof typeof MODE_ICONS];
          return (
            <button
              key={m.id}
              onClick={() => onDarkModeChange(m.id)}
              className={`flex-1 flex items-center justify-center gap-1.5 py-1.5 text-xs transition-colors ${
                darkMode === m.id
                  ? "bg-brand-500 text-text-inverse"
                  : "bg-surface-elevated text-text-secondary hover:text-text-primary"
              }`}
            >
              <Icon />
              {m.label}
            </button>
          );
        })}
      </div>
    </div>
  );
}
