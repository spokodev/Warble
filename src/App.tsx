import { useEffect, useState, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { LANGUAGES } from "./lib/languages";
import { WHISPER_MODELS } from "./lib/tauri-commands";
import type {
  RecordingState,
  AppConfig,
  HistoryEntry,
} from "./lib/tauri-commands";

type Tab = "status" | "settings" | "history";

/* ── Floating Status Bar (always on top, visible from any app) ── */
function StatusBar() {
  const [state, setState] = useState<RecordingState>("IDLE");

  useEffect(() => {
    invoke<RecordingState>("get_state").then(setState);
    let unsub: (() => void) | null = null;
    listen<RecordingState>("state-changed", (e) => setState(e.payload)).then(
      (u) => (unsub = u)
    );
    return () => unsub?.();
  }, []);

  const bg =
    state === "RECORDING"
      ? "bg-red-500"
      : state === "TRANSCRIBING"
        ? "bg-blue-500"
        : state === "STOPPING"
          ? "bg-yellow-500"
          : "bg-green-500/80";
  const label =
    state === "RECORDING"
      ? "REC"
      : state === "TRANSCRIBING"
        ? "Transcribing..."
        : state === "STOPPING"
          ? "Stopping..."
          : "Ready";

  return (
    <div
      className={`${bg} h-full w-full rounded-full flex items-center justify-center gap-1.5 px-3 text-white text-xs font-medium shadow-lg cursor-default select-none`}
      style={{ WebkitAppRegion: "drag" } as React.CSSProperties}
    >
      {state === "RECORDING" && (
        <div className="w-2 h-2 rounded-full bg-white animate-pulse" />
      )}
      <span>{label}</span>
    </div>
  );
}

function AppRouter() {
  const [label, setLabel] = useState("");
  useEffect(() => {
    import("@tauri-apps/api/window").then(({ getCurrentWindow }) => {
      setLabel(getCurrentWindow().label);
    });
  }, []);
  if (label === "status-bar") return <StatusBar />;
  return <MainWindow />;
}

function MainWindow() {
  const [state, setState] = useState<RecordingState>("IDLE");
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [provider, setProvider] = useState<string | null>(null);
  const [lastText, setLastText] = useState("");
  const [lastError, setLastError] = useState("");
  const [history, setHistory] = useState<HistoryEntry[]>([]);
  const [vocab, setVocab] = useState("");
  const [vocabDirty, setVocabDirty] = useState(false);
  const [tab, setTab] = useState<Tab>("status");

  useEffect(() => {
    invoke<AppConfig>("get_config").then((c) => {
      setConfig(c);
      if (c.apiKey) {
        invoke<string | null>("detect_provider", { apiKey: c.apiKey }).then(
          setProvider
        );
      }
    });
    invoke<string>("get_vocabulary").then(setVocab);
    invoke<HistoryEntry[]>("get_history").then(setHistory);
    invoke<RecordingState>("get_state").then(setState);
  }, []);

  useEffect(() => {
    const unsubs: (() => void)[] = [];
    listen<RecordingState>("state-changed", (e) => {
      setState(e.payload);
      if (e.payload === "IDLE") {
        invoke<HistoryEntry[]>("get_history").then(setHistory);
      }
    }).then((u) => unsubs.push(u));
    listen<string>("transcription-done", (e) => {
      setLastText(e.payload);
      setLastError("");
    }).then((u) => unsubs.push(u));
    listen<string>("transcription-error", (e) => {
      setLastError(e.payload);
    }).then((u) => unsubs.push(u));
    return () => unsubs.forEach((u) => u());
  }, []);

  const handleApiKeyChange = useCallback(
    (value: string) => {
      if (!config) return;
      setConfig({ ...config, apiKey: value });
      invoke("set_api_key", { key: value });
      invoke<string | null>("detect_provider", { apiKey: value }).then(
        setProvider
      );
    },
    [config]
  );

  if (!config) return <div className="p-6 text-gray-400">Loading...</div>;

  const stateColor =
    state === "RECORDING"
      ? "bg-red-500"
      : state === "TRANSCRIBING"
        ? "bg-blue-500"
        : state === "STOPPING"
          ? "bg-yellow-500"
          : "bg-green-500";
  const stateLabel =
    state === "RECORDING"
      ? "Recording..."
      : state === "TRANSCRIBING"
        ? "Transcribing..."
        : state === "STOPPING"
          ? "Stopping..."
          : "Ready";

  return (
    <div className="flex flex-col h-screen bg-white select-none">
      {/* Header */}
      <div className="px-4 py-3 border-b border-gray-200 flex items-center gap-3">
        <div className={`w-3 h-3 rounded-full ${stateColor} shrink-0`} />
        <span className="font-medium text-sm">{stateLabel}</span>
        <div className="flex-1" />
        {provider && (
          <span className="bg-blue-100 text-blue-700 text-xs px-2 py-0.5 rounded">
            {provider}
          </span>
        )}
        <span className="text-xs text-gray-400">F5 / Right ⌘</span>
      </div>

      {/* Tabs */}
      <div className="flex border-b border-gray-200 text-sm">
        {(["status", "settings", "history"] as Tab[]).map((t) => (
          <button
            key={t}
            onClick={() => setTab(t)}
            className={`flex-1 py-2 text-center ${
              tab === t
                ? "border-b-2 border-blue-500 text-blue-600 font-medium"
                : "text-gray-500 hover:text-gray-800"
            }`}
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
      <div className="flex-1 overflow-y-auto">
        {tab === "status" && (
          <StatusTab
            state={state}
            lastText={lastText}
            lastError={lastError}
            apiKeySet={!!config.apiKey}
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
          />
        )}
      </div>
    </div>
  );
}

/* ── Status Tab ── */
function StatusTab({
  state,
  lastText,
  lastError,
  apiKeySet,
}: {
  state: RecordingState;
  lastText: string;
  lastError: string;
  apiKeySet: boolean;
}) {
  return (
    <div className="p-4 space-y-4 text-sm">
      {!apiKeySet && (
        <div className="bg-yellow-50 border border-yellow-200 rounded p-3 text-yellow-800">
          API key not set — go to Settings tab.
        </div>
      )}
      <div className="bg-gray-50 rounded p-3 space-y-1 text-xs text-gray-600">
        <p>
          <kbd className="bg-gray-200 px-1.5 py-0.5 rounded">Right ⌘</kbd> —
          tap to start, release to stop
        </p>
        <p>
          <kbd className="bg-gray-200 px-1.5 py-0.5 rounded">F5</kbd> — toggle
          recording
        </p>
        <p className="text-gray-400 mt-2">
          Requires Accessibility permission for Right ⌘
        </p>
      </div>
      {state === "RECORDING" && (
        <div className="bg-red-50 border border-red-200 rounded p-3 text-red-700 animate-pulse">
          Recording... Release Right ⌘ or press F5 to stop.
        </div>
      )}
      {state === "TRANSCRIBING" && (
        <div className="bg-blue-50 border border-blue-200 rounded p-3 text-blue-700">
          Transcribing audio...
        </div>
      )}
      {lastError && (
        <div className="bg-red-50 border border-red-200 rounded p-3 text-red-700 text-xs">
          {lastError}
        </div>
      )}
      {lastText && (
        <div>
          <div className="flex items-center justify-between mb-1">
            <span className="text-xs text-gray-400">Last transcription:</span>
            <button
              onClick={() => {
                writeText(lastText);
              }}
              className="text-xs text-blue-500 hover:text-blue-700"
            >
              Copy
            </button>
          </div>
          <div className="bg-gray-50 rounded p-3 text-gray-800 break-words select-text cursor-text">
            {lastText}
          </div>
        </div>
      )}
    </div>
  );
}

/* ── Settings Tab ── */
function SettingsTab({
  config,
  setConfig,
  provider,
  vocab,
  vocabDirty,
  onApiKeyChange,
  onVocabChange,
  onSaveVocab,
}: {
  config: AppConfig;
  setConfig: (c: AppConfig) => void;
  provider: string | null;
  vocab: string;
  vocabDirty: boolean;
  onApiKeyChange: (v: string) => void;
  onVocabChange: (v: string) => void;
  onSaveVocab: () => void;
}) {
  const [showKey, setShowKey] = useState(false);

  const maskedKey = config.apiKey
    ? "••••••••" + config.apiKey.slice(-4)
    : "";

  return (
    <div className="p-4 space-y-4 text-sm">
      {/* API Key */}
      <div>
        <label className="block font-medium mb-1">API Key</label>
        <div className="flex gap-2 items-center">
          {showKey ? (
            <input
              type="text"
              value={config.apiKey}
              onChange={(e) => onApiKeyChange(e.target.value)}
              placeholder="wsk-..., sk_..., gsk..., sk-..."
              className="flex-1 border border-gray-300 rounded px-2.5 py-1.5 text-xs font-mono focus:outline-none focus:ring-2 focus:ring-blue-500"
              autoFocus
              onBlur={() => setShowKey(false)}
            />
          ) : (
            <div
              onClick={() => setShowKey(true)}
              className="flex-1 border border-gray-300 rounded px-2.5 py-1.5 text-xs font-mono text-gray-500 cursor-pointer hover:bg-gray-50"
            >
              {maskedKey || "Click to set API key..."}
            </div>
          )}
          {provider && (
            <span className="bg-blue-100 text-blue-700 text-xs px-2 py-0.5 rounded whitespace-nowrap">
              {provider}
            </span>
          )}
        </div>
      </div>

      {/* Model */}
      <div>
        <label className="block font-medium mb-1">Model</label>
        <select
          value={config.model}
          onChange={(e) => {
            setConfig({ ...config, model: e.target.value });
            invoke("set_model", { model: e.target.value });
          }}
          className="w-full border border-gray-300 rounded px-2.5 py-1.5 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
        >
          {WHISPER_MODELS.map((m) => (
            <option key={m.id} value={m.id}>
              {m.label} — {m.description}
            </option>
          ))}
        </select>
        <p className="text-xs text-gray-400 mt-1">
          Only applies to Self-Hosted provider
        </p>
      </div>

      {/* Language */}
      <div>
        <label className="block font-medium mb-1">Language</label>
        <select
          value={config.language}
          onChange={(e) => {
            setConfig({ ...config, language: e.target.value });
            invoke("set_language", { language: e.target.value });
          }}
          className="w-full border border-gray-300 rounded px-2.5 py-1.5 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
        >
          {LANGUAGES.map((l) => (
            <option key={l.code} value={l.code}>
              {l.label}
            </option>
          ))}
        </select>
      </div>

      {/* Toggles */}
      <div className="space-y-2">
        <label className="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            checked={config.soundsEnabled}
            onChange={(e) => {
              setConfig({ ...config, soundsEnabled: e.target.checked });
              invoke("set_sounds_enabled", { enabled: e.target.checked });
            }}
          />
          <span>Sound feedback</span>
        </label>
        <label className="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            checked={config.autoPaste}
            onChange={(e) => {
              setConfig({ ...config, autoPaste: e.target.checked });
              invoke("set_auto_paste", { enabled: e.target.checked });
            }}
          />
          <span>Auto-paste after transcription</span>
        </label>
        {!config.autoPaste && (
          <p className="text-xs text-gray-400 ml-6">
            Text is copied to clipboard. Press Cmd+V to paste manually.
          </p>
        )}
        <label className="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            checked={config.datasetCollectionEnabled}
            onChange={(e) => {
              setConfig({
                ...config,
                datasetCollectionEnabled: e.target.checked,
              });
              invoke("set_dataset_collection", { enabled: e.target.checked });
            }}
          />
          <span>Save audio+text for fine-tuning</span>
        </label>
      </div>

      {/* Vocabulary */}
      <div>
        <div className="flex items-center justify-between mb-1">
          <label className="font-medium">Vocabulary</label>
          {vocabDirty && (
            <button
              onClick={onSaveVocab}
              className="bg-blue-500 text-white text-xs px-3 py-1 rounded hover:bg-blue-600"
            >
              Save
            </button>
          )}
        </div>
        <textarea
          value={vocab}
          onChange={(e) => onVocabChange(e.target.value)}
          rows={4}
          className="w-full border border-gray-300 rounded px-2.5 py-2 text-xs font-mono resize-y focus:outline-none focus:ring-2 focus:ring-blue-500"
          placeholder="Words/phrases to help recognition..."
        />
      </div>
    </div>
  );
}

/* ── History Tab ── */
function HistoryTab({
  entries,
  setEntries,
  onClear,
}: {
  entries: HistoryEntry[];
  setEntries: (e: HistoryEntry[]) => void;
  onClear: () => void;
}) {
  const [editingIdx, setEditingIdx] = useState<number | null>(null);
  const [editText, setEditText] = useState("");
  const [copiedIdx, setCopiedIdx] = useState<number | null>(null);

  const handleCopy = async (text: string, idx: number) => {
    await writeText(text);
    setCopiedIdx(idx);
    setTimeout(() => setCopiedIdx(null), 1500);
  };

  const handleStartEdit = (idx: number) => {
    setEditingIdx(idx);
    setEditText(entries[idx].text);
  };

  const handleSaveEdit = async (idx: number) => {
    await invoke("update_history_entry", { index: idx, newText: editText });
    const updated = [...entries];
    updated[idx] = { ...updated[idx], text: editText };
    setEntries(updated);
    setEditingIdx(null);
  };

  const handleCancelEdit = () => {
    setEditingIdx(null);
    setEditText("");
  };

  return (
    <div className="flex flex-col h-full">
      {entries.length === 0 ? (
        <div className="p-8 text-center text-gray-400 text-sm">
          No transcriptions yet. Press F5 or Right ⌘ to record.
        </div>
      ) : (
        <div className="flex-1 overflow-y-auto divide-y divide-gray-100">
          {entries.map((entry, idx) => (
            <div key={`${entry.timestamp}-${idx}`} className="px-4 py-3">
              {editingIdx === idx ? (
                /* Editing mode */
                <div className="space-y-2">
                  <textarea
                    value={editText}
                    onChange={(e) => setEditText(e.target.value)}
                    rows={3}
                    className="w-full border border-blue-300 rounded px-2 py-1.5 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                    autoFocus
                  />
                  <div className="flex gap-2">
                    <button
                      onClick={() => handleSaveEdit(idx)}
                      className="bg-blue-500 text-white text-xs px-3 py-1 rounded hover:bg-blue-600"
                    >
                      Save correction
                    </button>
                    <button
                      onClick={handleCancelEdit}
                      className="text-gray-500 text-xs px-3 py-1 rounded hover:text-gray-800"
                    >
                      Cancel
                    </button>
                    {entry.datasetId && (
                      <span className="text-xs text-green-600 self-center ml-auto">
                        Updates training data
                      </span>
                    )}
                  </div>
                </div>
              ) : (
                /* View mode */
                <div>
                  <p className="text-sm text-gray-800 break-words">
                    {entry.text}
                  </p>
                  <div className="flex items-center gap-3 mt-1.5">
                    <time className="text-xs text-gray-400">
                      {entry.timestamp}
                    </time>
                    <button
                      onClick={() => handleCopy(entry.text, idx)}
                      className="text-xs text-blue-500 hover:text-blue-700"
                    >
                      {copiedIdx === idx ? "Copied!" : "Copy"}
                    </button>
                    <button
                      onClick={() => handleStartEdit(idx)}
                      className="text-xs text-gray-500 hover:text-gray-800"
                    >
                      Edit
                    </button>
                    <button
                      onClick={async () => {
                        await invoke("delete_history_entry", { index: idx });
                        setEntries(entries.filter((_, i) => i !== idx));
                      }}
                      className="text-xs text-red-400 hover:text-red-600"
                    >
                      Delete
                    </button>
                    {entry.datasetId && (
                      <span className="text-xs text-gray-300" title="Linked to training data">
                        dataset
                      </span>
                    )}
                  </div>
                </div>
              )}
            </div>
          ))}
        </div>
      )}
      {entries.length > 0 && (
        <div className="p-3 border-t border-gray-200 shrink-0">
          <button
            onClick={onClear}
            className="text-xs text-red-500 hover:text-red-700"
          >
            Clear All
          </button>
        </div>
      )}
    </div>
  );
}

export default AppRouter;
