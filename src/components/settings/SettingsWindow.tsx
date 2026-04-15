import { useEffect, useState } from "react";
import {
  getConfig,
  setApiKey,
  setLanguage,
  setSoundsEnabled,
  setDatasetCollection,
  getVocabulary,
  setVocabulary,
  detectProvider,
  type AppConfig,
} from "../../lib/tauri-commands";
import { LANGUAGES } from "../../lib/languages";

export default function SettingsWindow() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [provider, setProvider] = useState<string | null>(null);
  const [vocab, setVocab] = useState("");
  const [vocabDirty, setVocabDirty] = useState(false);

  useEffect(() => {
    getConfig().then(setConfig);
    getVocabulary().then(setVocab);
  }, []);

  useEffect(() => {
    if (config?.apiKey) {
      detectProvider(config.apiKey).then(setProvider);
    } else {
      setProvider(null);
    }
  }, [config?.apiKey]);

  if (!config) return <div className="p-6">Loading...</div>;

  const handleApiKeyChange = (value: string) => {
    setConfig({ ...config, apiKey: value });
    setApiKey(value);
  };

  const handleLanguageChange = (value: string) => {
    setConfig({ ...config, language: value });
    setLanguage(value);
  };

  const handleSaveVocab = () => {
    setVocabulary(vocab);
    setVocabDirty(false);
  };

  return (
    <div className="p-6 bg-white min-h-screen text-sm">
      <h1 className="text-lg font-semibold mb-5">Settings</h1>

      {/* API Key */}
      <section className="mb-5">
        <label className="block font-medium mb-1">API Key</label>
        <div className="flex gap-2 items-center">
          <input
            type="password"
            value={config.apiKey}
            onChange={(e) => handleApiKeyChange(e.target.value)}
            placeholder="wsk-..., sk_..., gsk..., sk-..."
            className="flex-1 border border-gray-300 rounded px-3 py-1.5 text-sm font-mono focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          {provider && (
            <span className="bg-blue-100 text-blue-800 text-xs px-2 py-1 rounded whitespace-nowrap">
              {provider}
            </span>
          )}
        </div>
        <p className="text-gray-400 text-xs mt-1">
          Supports: Self-Hosted (wsk-), ElevenLabs (sk_), Groq (gsk), OpenAI
          (sk-)
        </p>
      </section>

      {/* Language */}
      <section className="mb-5">
        <label className="block font-medium mb-1">Language</label>
        <select
          value={config.language}
          onChange={(e) => handleLanguageChange(e.target.value)}
          className="w-full border border-gray-300 rounded px-3 py-1.5 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
        >
          {LANGUAGES.map((lang) => (
            <option key={lang.code} value={lang.code}>
              {lang.label}
            </option>
          ))}
        </select>
      </section>

      {/* Toggles */}
      <section className="mb-5 space-y-3">
        <label className="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            checked={config.soundsEnabled}
            onChange={(e) => {
              setConfig({ ...config, soundsEnabled: e.target.checked });
              setSoundsEnabled(e.target.checked);
            }}
            className="rounded"
          />
          <span>Sound feedback</span>
        </label>
        <label className="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            checked={config.datasetCollectionEnabled}
            onChange={(e) => {
              setConfig({
                ...config,
                datasetCollectionEnabled: e.target.checked,
              });
              setDatasetCollection(e.target.checked);
            }}
            className="rounded"
          />
          <span>Collect audio+text for fine-tuning</span>
        </label>
      </section>

      {/* Hotkey Info */}
      <section className="mb-5">
        <label className="block font-medium mb-1">Hotkeys</label>
        <div className="bg-gray-50 rounded p-3 text-xs space-y-1">
          <div>
            <kbd className="bg-gray-200 px-1.5 py-0.5 rounded text-xs">
              Right ⌘
            </kbd>{" "}
            — tap to start, release to stop
          </div>
          <div>
            <kbd className="bg-gray-200 px-1.5 py-0.5 rounded text-xs">
              F5
            </kbd>{" "}
            — toggle recording (fallback)
          </div>
        </div>
      </section>

      {/* Vocabulary */}
      <section className="mb-5">
        <div className="flex items-center justify-between mb-1">
          <label className="font-medium">Vocabulary</label>
          {vocabDirty && (
            <button
              onClick={handleSaveVocab}
              className="bg-blue-500 text-white text-xs px-3 py-1 rounded hover:bg-blue-600"
            >
              Save
            </button>
          )}
        </div>
        <textarea
          value={vocab}
          onChange={(e) => {
            setVocab(e.target.value);
            setVocabDirty(true);
          }}
          rows={6}
          className="w-full border border-gray-300 rounded px-3 py-2 text-xs font-mono resize-y focus:outline-none focus:ring-2 focus:ring-blue-500"
          placeholder="Add words/phrases to improve recognition..."
        />
        <p className="text-gray-400 text-xs mt-1">
          Words and phrases sent as hints to the transcription model
        </p>
      </section>
    </div>
  );
}
