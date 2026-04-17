import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useRecordingState } from "../../hooks/useRecordingState";
import { useAppConfig } from "../../hooks/useAppConfig";
import { useTheme } from "../../hooks/useTheme";
import { StatusTab } from "../tabs/StatusTab";
import { SettingsTab } from "../tabs/SettingsTab";
import { HistoryTab } from "../tabs/HistoryTab";
import { RecordingIndicator } from "../StatusBar/RecordingIndicator";

type Tab = "status" | "settings" | "history";

export function MainWindow() {
  const { state } = useRecordingState();
  const { theme, darkMode, setTheme, setDarkMode } = useTheme();
  const {
    config,
    setConfig,
    provider,
    lastText,
    lastError,
    history,
    setHistory,
    updateLastText,
    vocab,
    setVocab,
    vocabDirty,
    setVocabDirty,
    handleApiKeyChange,
  } = useAppConfig();

  const [tab, setTab] = useState<Tab>("status");

  if (!config)
    return <div className="p-6 text-text-tertiary">Loading...</div>;

  const stateLabel =
    state === "RECORDING"
      ? "Recording..."
      : state === "TRANSCRIBING"
        ? "Transcribing..."
        : state === "STOPPING"
          ? "Stopping..."
          : "Ready";

  return (
    <div className="flex flex-col h-screen bg-surface select-none">
      {/* Header */}
      <div className="glass-panel-elevated rounded-none px-4 py-3 flex items-center gap-3">
        <div className="relative">
          <div
            className="state-dot shrink-0"
            data-state={
              state === "RECORDING"
                ? "recording"
                : state === "TRANSCRIBING"
                  ? "transcribing"
                  : state === "STOPPING"
                    ? "stopping"
                    : "idle"
            }
          />
          {state === "RECORDING" && (
            <div className="absolute -inset-1.5">
              <RecordingIndicator />
            </div>
          )}
        </div>
        <span className="font-medium text-sm text-text-primary">
          {stateLabel}
        </span>
        <div className="flex-1" />
        {provider && (
          <span className="bg-brand-100 text-brand-700 text-xs px-2 py-0.5 rounded dark:bg-brand-900/30 dark:text-brand-300">
            {provider}
          </span>
        )}
      </div>

      {/* Tabs */}
      <div className="flex border-b border-border text-sm">
        {(["status", "settings", "history"] as Tab[]).map((t) => (
          <button
            key={t}
            onClick={() => setTab(t)}
            className="tab-item flex-1 text-center"
            data-active={tab === t}
          >
            {t === "status"
              ? "Status"
              : t === "settings"
                ? "Settings"
                : `History (${history.length})`}
          </button>
        ))}
      </div>

      {/* Content */}
      <div key={tab} className="flex-1 overflow-y-auto animate-warble-tab-enter">
        {tab === "status" && (
          <StatusTab
            state={state}
            lastText={lastText}
            lastError={lastError}
            apiKeySet={!!config.apiKey}
            onUpdateLastText={updateLastText}
          />
        )}
        {tab === "settings" && (
          <SettingsTab
            config={config}
            setConfig={setConfig}
            provider={provider}
            vocab={vocab}
            vocabDirty={vocabDirty}
            onApiKeyChange={handleApiKeyChange}
            onVocabChange={(v) => {
              setVocab(v);
              setVocabDirty(true);
            }}
            onSaveVocab={() => {
              invoke("set_vocabulary", { content: vocab });
              setVocabDirty(false);
            }}
            theme={theme}
            darkMode={darkMode}
            onThemeChange={setTheme}
            onDarkModeChange={setDarkMode}
          />
        )}
        {tab === "history" && (
          <HistoryTab
            entries={history}
            setEntries={setHistory}
            onClear={() => {
              invoke("clear_history");
              setHistory([]);
            }}
            onUpdateEntry={(idx, text) => {
              if (idx === 0) updateLastText(text);
            }}
          />
        )}
      </div>
    </div>
  );
}
