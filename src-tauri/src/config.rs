use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub text: String,
    pub timestamp: String,
    /// Filename stem in dataset dir (e.g. "20260414-193500"), if saved
    #[serde(default)]
    pub dataset_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AppConfig {
    pub api_key: String,
    pub language: String,
    pub model: String,
    pub sounds_enabled: bool,
    pub auto_paste: bool,
    pub dataset_collection_enabled: bool,
    pub history: Vec<HistoryEntry>,
    /// Global shortcut for recording toggle (e.g. "F5", "CmdOrCtrl+Shift+R")
    pub hotkey: String,
    /// Color theme: "amber", "teal", or "violet"
    pub theme: String,
    /// Dark mode: "light", "dark", or "system"
    pub dark_mode: String,
    /// Status bar visibility: "always", "recording", or "never"
    pub status_bar_visibility: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            language: "uk".to_string(),
            model: "Systran/faster-whisper-small".to_string(),
            sounds_enabled: true,
            auto_paste: true,
            dataset_collection_enabled: false,
            history: Vec::new(),
            hotkey: "RightCommand".to_string(),
            theme: "amber".to_string(),
            dark_mode: "system".to_string(),
            status_bar_visibility: "always".to_string(),
        }
    }
}

impl AppConfig {
    pub fn add_history(&mut self, text: String, dataset_id: Option<String>) {
        let entry = HistoryEntry {
            text,
            timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            dataset_id,
        };
        self.history.insert(0, entry);
        if self.history.len() > 50 {
            self.history.truncate(50);
        }
    }
}

// --- Config directory migration ---

/// Migrate config from ~/.whisperspoon to ~/.warble (one-time, on app startup).
pub fn migrate_config_dir() {
    let home = dirs::home_dir().expect("No home directory");
    let old_dir = home.join(".whisperspoon");
    let new_dir = home.join(".warble");

    if old_dir.exists() && !new_dir.exists() {
        if let Err(e) = std::fs::rename(&old_dir, &new_dir) {
            eprintln!(
                "Failed to migrate config dir from .whisperspoon to .warble: {}",
                e
            );
        }
    }
}

// --- File-based config persistence (replaces tauri-plugin-store) ---

fn config_path() -> PathBuf {
    let home = dirs::home_dir().expect("No home directory");
    home.join(".warble").join("app-config.json")
}

pub fn load_config_from_file() -> AppConfig {
    let path = config_path();
    if let Ok(content) = std::fs::read_to_string(&path) {
        if let Ok(config) = serde_json::from_str::<AppConfig>(&content) {
            return config;
        }
    }

    // Fall back to Hammerspoon import on first run
    if let Some(config) = import_from_hammerspoon() {
        save_config_to_file(&config);
        return config;
    }

    AppConfig::default()
}

pub fn save_config_to_file(config: &AppConfig) {
    let path = config_path();
    let _ = std::fs::create_dir_all(path.parent().unwrap());
    if let Ok(json) = serde_json::to_string_pretty(config) {
        let _ = std::fs::write(&path, json);
    }
}

/// Load vocabulary from the user's vocabulary file.
/// Supports comment lines (starting with #) and joins lines as sentence context.
pub fn load_vocabulary() -> String {
    let path = vocabulary_path();
    match std::fs::read_to_string(&path) {
        Ok(content) => content
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .collect::<Vec<_>>()
            .join(". "),
        Err(_) => String::new(),
    }
}

pub fn vocabulary_path() -> PathBuf {
    let home = dirs::home_dir().expect("No home directory");
    home.join(".warble").join("vocabulary.txt")
}

/// Try to import settings from existing Hammerspoon config.
fn import_from_hammerspoon() -> Option<AppConfig> {
    let home = dirs::home_dir()?;
    let config_path = home.join(".warble").join("config.json");
    let content = std::fs::read_to_string(config_path).ok()?;

    #[derive(Deserialize)]
    struct HammerspoonConfig {
        #[serde(rename = "apiKey")]
        api_key: Option<String>,
        language: Option<String>,
        history: Option<Vec<String>>,
    }

    let hs_config: HammerspoonConfig = serde_json::from_str(&content).ok()?;

    let mut config = AppConfig::default();
    if let Some(key) = hs_config.api_key {
        config.api_key = key;
    }
    if let Some(lang) = hs_config.language {
        config.language = lang;
    }
    if let Some(history) = hs_config.history {
        for text in history {
            config.add_history(text, None);
        }
    }

    Some(config)
}
