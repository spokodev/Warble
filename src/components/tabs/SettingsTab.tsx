import { useState, useEffect, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { LANGUAGES } from "../../lib/languages";
import { WHISPER_MODELS } from "../../lib/tauri-commands";
import type { AppConfig, ThemeId, DarkMode, StatusBarVisibility } from "../../lib/tauri-commands";
import { Toggle } from "../shared/Toggle";
import { ThemePicker } from "../shared/ThemePicker";

// ── Hotkey recorder ──

const KEY_MAP: Record<string, string> = {
  Space: "Space", Enter: "Enter", Tab: "Tab",
  Backspace: "Backspace", Delete: "Delete",
  ArrowUp: "Up", ArrowDown: "Down", ArrowLeft: "Left", ArrowRight: "Right",
  Home: "Home", End: "End", PageUp: "PageUp", PageDown: "PageDown",
  Insert: "Insert", Minus: "-", Equal: "=",
  BracketLeft: "[", BracketRight: "]", Backslash: "\\",
  Semicolon: ";", Quote: "'", Comma: ",", Period: ".", Slash: "/", Backquote: "`",
};

function codeToKey(code: string, key: string): string | null {
  // Right Command as standalone hotkey (macOS)
  if (code === "MetaRight" && key === "Meta") return "RightCommand";
  if (["Meta", "Control", "Alt", "Shift"].includes(key)) return null;
  if (/^F\d{1,2}$/.test(code)) return code;
  if (code.startsWith("Key")) return code.slice(3);
  if (code.startsWith("Digit")) return code.slice(5);
  return KEY_MAP[code] ?? null;
}

function formatForDisplay(shortcut: string): string {
  if (shortcut === "RightCommand") return "Right ⌘";
  return shortcut
    .split("+")
    .map((p) => {
      if (p === "CmdOrCtrl") return "⌘";
      if (p === "Alt") return "⌥";
      if (p === "Shift") return "⇧";
      return p;
    })
    .join(" ");
}

function HotkeyRecorder({
  value,
  onChange,
}: {
  value: string;
  onChange: (hotkey: string) => void;
}) {
  const [recording, setRecording] = useState(false);
  const btnRef = useRef<HTMLButtonElement>(null);

  const startRecording = useCallback(() => {
    // Pause both global shortcut and Right ⌘ watcher
    invoke("unregister_hotkey");
    invoke("pause_hotkey_watcher", { paused: true });
    setRecording(true);
  }, []);

  const stopRecording = useCallback(
    (newHotkey?: string) => {
      setRecording(false);
      // Resume Right ⌘ watcher
      invoke("pause_hotkey_watcher", { paused: false });
      if (newHotkey) {
        // User selected a new hotkey — set_hotkey re-registers
        onChange(newHotkey);
      } else {
        // Cancelled — re-register the previous hotkey
        invoke("set_hotkey", { hotkey: value });
      }
    },
    [onChange, value],
  );

  const handleCapture = useCallback(
    (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();

      if (e.key === "Escape") {
        stopRecording();
        return;
      }

      const mainKey = codeToKey(e.code, e.key);
      if (!mainKey) return; // modifier-only, keep waiting

      const parts: string[] = [];
      if (e.metaKey || e.ctrlKey) parts.push("CmdOrCtrl");
      if (e.altKey) parts.push("Alt");
      if (e.shiftKey) parts.push("Shift");

      // RightCommand is a special standalone hotkey
      if (mainKey === "RightCommand") {
        stopRecording("RightCommand");
        return;
      }

      // Single non-function key without modifier — not a valid global shortcut
      const isFnKey = /^F\d{1,2}$/.test(mainKey);
      if (!isFnKey && parts.length === 0) return;

      parts.push(mainKey);
      stopRecording(parts.join("+"));
    },
    [stopRecording],
  );

  useEffect(() => {
    if (!recording) return;
    window.addEventListener("keydown", handleCapture, true);
    return () => window.removeEventListener("keydown", handleCapture, true);
  }, [recording, handleCapture]);

  return (
    <button
      ref={btnRef}
      type="button"
      onClick={() => startRecording()}
      onBlur={() => stopRecording()}
      className={`input w-full text-left font-mono text-sm transition-colors ${
        recording
          ? "ring-2 ring-brand-500 text-brand-600 dark:text-brand-400"
          : ""
      }`}
    >
      {recording ? "Press a shortcut…" : formatForDisplay(value)}
    </button>
  );
}

interface SettingsTabProps {
  config: AppConfig;
  setConfig: (c: AppConfig) => void;
  provider: string | null;
  vocab: string;
  vocabDirty: boolean;
  onApiKeyChange: (v: string) => void;
  onVocabChange: (v: string) => void;
  onSaveVocab: () => void;
  theme: ThemeId;
  darkMode: DarkMode;
  onThemeChange: (t: ThemeId) => void;
  onDarkModeChange: (m: DarkMode) => void;
}

export function SettingsTab({
  config,
  setConfig,
  provider,
  vocab,
  vocabDirty,
  onApiKeyChange,
  onVocabChange,
  onSaveVocab,
  theme,
  darkMode,
  onThemeChange,
  onDarkModeChange,
}: SettingsTabProps) {
  const [showKey, setShowKey] = useState(false);
  const maskedKey = config.apiKey ? "••••••••" + config.apiKey.slice(-4) : "";

  return (
    <div className="p-4 space-y-5 text-sm">
      {/* Appearance */}
      <ThemePicker
        theme={theme}
        darkMode={darkMode}
        onThemeChange={onThemeChange}
        onDarkModeChange={onDarkModeChange}
      />

      {/* Status Bar Visibility */}
      <div>
        <label className="block font-medium mb-1 text-text-primary">
          Status Bar
        </label>
        <select
          value={config.statusBarVisibility}
          onChange={(e) => {
            const v = e.target.value as StatusBarVisibility;
            setConfig({ ...config, statusBarVisibility: v });
            invoke("set_status_bar_visibility", { visibility: v });
          }}
          className="select w-full"
        >
          <option value="always">Always visible</option>
          <option value="recording">Recording only</option>
          <option value="never">Hidden</option>
        </select>
      </div>

      {/* Hotkey */}
      <div>
        <label className="block font-medium mb-1 text-text-primary">
          Recording Hotkey
        </label>
        <HotkeyRecorder
          value={config.hotkey}
          onChange={(hotkey) => {
            setConfig({ ...config, hotkey });
            invoke("set_hotkey", { hotkey });
          }}
        />
        <p className="text-xs text-text-tertiary mt-1">
          Click and press any key combination. Default: Right ⌘
        </p>
      </div>

      {/* API Key */}
      <div>
        <label className="block font-medium mb-1 text-text-primary">
          API Key
        </label>
        <div className="flex gap-2 items-center">
          {showKey ? (
            <input
              type="text"
              value={config.apiKey}
              onChange={(e) => onApiKeyChange(e.target.value)}
              placeholder="wsk-..., sk_..., gsk..., sk-..."
              className="input flex-1 text-xs font-mono"
              autoFocus
              onBlur={() => setShowKey(false)}
            />
          ) : (
            <div
              onClick={() => setShowKey(true)}
              className="input flex-1 text-xs font-mono text-text-tertiary cursor-pointer hover:bg-surface-inset"
            >
              {maskedKey || "Click to set API key..."}
            </div>
          )}
          {provider && (
            <span className="bg-brand-100 text-brand-700 text-xs px-2 py-0.5 rounded whitespace-nowrap dark:bg-brand-900/30 dark:text-brand-300">
              {provider}
            </span>
          )}
        </div>
      </div>

      {/* Model */}
      <div>
        <label className="block font-medium mb-1 text-text-primary">
          Model
        </label>
        <select
          value={config.model}
          onChange={(e) => {
            setConfig({ ...config, model: e.target.value });
            invoke("set_model", { model: e.target.value });
          }}
          className="select w-full"
        >
          {WHISPER_MODELS.map((m) => (
            <option key={m.id} value={m.id}>
              {m.label} — {m.description}
            </option>
          ))}
        </select>
        <p className="text-xs text-text-tertiary mt-1">
          Only applies to Self-Hosted provider
        </p>
      </div>

      {/* Language */}
      <div>
        <label className="block font-medium mb-1 text-text-primary">
          Language
        </label>
        <select
          value={config.language}
          onChange={(e) => {
            setConfig({ ...config, language: e.target.value });
            invoke("set_language", { language: e.target.value });
          }}
          className="select w-full"
        >
          {LANGUAGES.map((l) => (
            <option key={l.code} value={l.code}>
              {l.label}
            </option>
          ))}
        </select>
      </div>

      {/* Toggles */}
      <div className="space-y-3">
        <Toggle
          checked={config.soundsEnabled}
          onChange={(checked) => {
            setConfig({ ...config, soundsEnabled: checked });
            invoke("set_sounds_enabled", { enabled: checked });
          }}
          label="Sound feedback"
        />
        <Toggle
          checked={config.autoPaste}
          onChange={(checked) => {
            setConfig({ ...config, autoPaste: checked });
            invoke("set_auto_paste", { enabled: checked });
          }}
          label="Auto-paste after transcription"
        />
        {!config.autoPaste && (
          <p className="text-xs text-text-tertiary ml-12">
            Text is copied to clipboard. Press Cmd+V to paste manually.
          </p>
        )}
      </div>

      {/* Vocabulary */}
      <div>
        <div className="flex items-center justify-between mb-1">
          <label className="font-medium text-text-primary">Vocabulary</label>
          {vocabDirty && (
            <button onClick={onSaveVocab} className="btn-primary text-xs">
              Save
            </button>
          )}
        </div>
        <textarea
          value={vocab}
          onChange={(e) => onVocabChange(e.target.value)}
          rows={4}
          className="input w-full font-mono text-xs resize-y"
          placeholder={"# Technical terms\nTauri, TypeScript, PostgreSQL\n\n# Context phrases\nДавай зробимо deploy."}
        />
      </div>
    </div>
  );
}
