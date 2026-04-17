import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { LANGUAGES } from "../../lib/languages";
import { WHISPER_MODELS } from "../../lib/tauri-commands";
import type { AppConfig, ThemeId, DarkMode } from "../../lib/tauri-commands";
import { Toggle } from "../shared/Toggle";
import { ThemePicker } from "../shared/ThemePicker";

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

      {/* Hotkey */}
      <div>
        <label className="block font-medium mb-1 text-text-primary">
          Recording Hotkey
        </label>
        <select
          value={config.hotkey}
          onChange={(e) => {
            setConfig({ ...config, hotkey: e.target.value });
            invoke("set_hotkey", { hotkey: e.target.value });
          }}
          className="select w-full"
        >
          <option value="F5">F5</option>
          <option value="F6">F6</option>
          <option value="F7">F7</option>
          <option value="F8">F8</option>
          <option value="F9">F9</option>
          <option value="CmdOrCtrl+Shift+R">Cmd/Ctrl + Shift + R</option>
          <option value="CmdOrCtrl+Shift+S">Cmd/Ctrl + Shift + S</option>
          <option value="Alt+Space">Alt + Space</option>
        </select>
        <p className="text-xs text-text-tertiary mt-1">
          Right ⌘ tap also works on macOS (requires Accessibility)
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
        <Toggle
          checked={config.datasetCollectionEnabled}
          onChange={(checked) => {
            setConfig({ ...config, datasetCollectionEnabled: checked });
            invoke("set_dataset_collection", { enabled: checked });
          }}
          label="Save audio+text for fine-tuning"
        />
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
