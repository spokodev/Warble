import { invoke } from "@tauri-apps/api/core";

export type RecordingState = "IDLE" | "RECORDING" | "STOPPING" | "TRANSCRIBING";

export interface HistoryEntry {
  text: string;
  timestamp: string;
  datasetId: string | null;
}

export type ThemeId = "amber" | "teal" | "violet";
export type DarkMode = "light" | "dark" | "system";

export interface AppConfig {
  apiKey: string;
  language: string;
  model: string;
  soundsEnabled: boolean;
  autoPaste: boolean;
  datasetCollectionEnabled: boolean;
  history: HistoryEntry[];
  theme: ThemeId;
  darkMode: DarkMode;
}

export const getState = () => invoke<RecordingState>("get_state");
export const getConfig = () => invoke<AppConfig>("get_config");
export const setApiKey = (key: string) => invoke("set_api_key", { key });
export const setLanguage = (language: string) => invoke("set_language", { language });
export const setModel = (model: string) => invoke("set_model", { model });
export const setSoundsEnabled = (enabled: boolean) => invoke("set_sounds_enabled", { enabled });
export const setDatasetCollection = (enabled: boolean) => invoke("set_dataset_collection", { enabled });
export const getHistory = () => invoke<HistoryEntry[]>("get_history");
export const clearHistory = () => invoke("clear_history");
export const updateHistoryEntry = (index: number, newText: string) => invoke("update_history_entry", { index, newText });
export const getVocabulary = () => invoke<string>("get_vocabulary");
export const setVocabulary = (content: string) => invoke("set_vocabulary", { content });
export const detectProvider = (apiKey: string) => invoke<string | null>("detect_provider", { apiKey });
export const setTheme = (theme: ThemeId) => invoke("set_theme", { theme });
export const setDarkMode = (mode: DarkMode) => invoke("set_dark_mode", { mode });

export const WHISPER_MODELS = [
  { id: "Systran/faster-whisper-small", label: "Small (fast)", description: "~1s per recording" },
  { id: "Systran/faster-whisper-medium", label: "Medium (balanced)", description: "~3s per recording" },
  { id: "Systran/faster-whisper-large-v3", label: "Large v3 (best)", description: "~6s, highest accuracy" },
] as const;
