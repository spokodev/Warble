import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { AppConfig, HistoryEntry } from "../lib/tauri-commands";

export function useAppConfig() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [provider, setProvider] = useState<string | null>(null);
  const [lastText, setLastText] = useState("");
  const [lastError, setLastError] = useState("");
  const [history, setHistory] = useState<HistoryEntry[]>([]);
  const [vocab, setVocab] = useState("");
  const [vocabDirty, setVocabDirty] = useState(false);

  useEffect(() => {
    invoke<AppConfig>("get_config").then((c) => {
      setConfig(c);
      if (c.apiKey) {
        invoke<string | null>("detect_provider", { apiKey: c.apiKey }).then(
          setProvider,
        );
      }
    });
    invoke<string>("get_vocabulary").then(setVocab);
    invoke<HistoryEntry[]>("get_history").then(setHistory);
  }, []);

  useEffect(() => {
    const unsubs: (() => void)[] = [];
    listen<string>("state-changed", (e) => {
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
    listen("vocabulary-changed", () => {
      invoke<string>("get_vocabulary").then(setVocab);
    }).then((u) => unsubs.push(u));
    return () => unsubs.forEach((u) => u());
  }, []);

  const handleApiKeyChange = useCallback(
    (value: string) => {
      if (!config) return;
      setConfig({ ...config, apiKey: value });
      invoke("set_api_key", { key: value });
      invoke<string | null>("detect_provider", { apiKey: value }).then(
        setProvider,
      );
    },
    [config],
  );

  const updateLastText = useCallback(
    (text: string) => {
      setLastText(text);
      // Sync history[0] with the new text
      setHistory((prev) => {
        if (prev.length === 0) return prev;
        const updated = [...prev];
        updated[0] = { ...updated[0], text };
        return updated;
      });
    },
    [],
  );

  return {
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
  };
}
