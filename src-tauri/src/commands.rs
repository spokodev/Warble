use crate::config::{self, AppConfig, HistoryEntry};
use crate::state::RecordingState;
use tauri::{Manager, State};

pub type ConfigMutex = std::sync::Mutex<AppConfig>;

fn save(config: &State<'_, ConfigMutex>) {
    let cfg = config.lock().unwrap().clone();
    config::save_config_to_file(&cfg);
}

#[tauri::command]
pub fn get_state(state: State<'_, crate::state::SharedState>) -> RecordingState {
    state.lock().unwrap().state
}

#[tauri::command]
pub fn get_config(config: State<'_, ConfigMutex>) -> AppConfig {
    config.lock().unwrap().clone()
}

#[tauri::command]
pub fn set_api_key(key: String, config: State<'_, ConfigMutex>) {
    config.lock().unwrap().api_key = key;
    save(&config);
}

#[tauri::command]
pub fn set_language(language: String, config: State<'_, ConfigMutex>) {
    config.lock().unwrap().language = language;
    save(&config);
}

#[tauri::command]
pub fn set_model(model: String, config: State<'_, ConfigMutex>) {
    config.lock().unwrap().model = model;
    save(&config);
}

#[tauri::command]
pub fn set_sounds_enabled(enabled: bool, config: State<'_, ConfigMutex>) {
    config.lock().unwrap().sounds_enabled = enabled;
    save(&config);
}

#[tauri::command]
pub fn set_auto_paste(enabled: bool, config: State<'_, ConfigMutex>) {
    config.lock().unwrap().auto_paste = enabled;
    save(&config);
}

#[tauri::command]
pub fn set_dataset_collection(enabled: bool, config: State<'_, ConfigMutex>) {
    config.lock().unwrap().dataset_collection_enabled = enabled;
    save(&config);
}

#[tauri::command]
pub fn get_history(config: State<'_, ConfigMutex>) -> Vec<HistoryEntry> {
    config.lock().unwrap().history.clone()
}

#[tauri::command]
pub fn clear_history(config: State<'_, ConfigMutex>) {
    config.lock().unwrap().history.clear();
    save(&config);
}

#[tauri::command]
pub fn delete_history_entry(index: usize, config: State<'_, ConfigMutex>) {
    let mut cfg = config.lock().unwrap();
    if index < cfg.history.len() {
        cfg.history.remove(index);
    }
    drop(cfg);
    save(&config);
}

#[tauri::command]
pub fn update_history_entry(index: usize, new_text: String, config: State<'_, ConfigMutex>) -> Result<(), String> {
    let mut cfg = config.lock().unwrap();
    let entry = cfg.history.get_mut(index).ok_or("Index out of bounds")?;
    let old_text = entry.text.clone();
    entry.text = new_text.clone();

    // Update the dataset .txt file if it exists
    if let Some(ref dataset_id) = entry.dataset_id {
        crate::dataset::update_text(dataset_id, &new_text);
    }

    crate::log(&format!(
        "History: entry {} updated ('{}...' → '{}...')",
        index,
        old_text.chars().take(30).collect::<String>(),
        new_text.chars().take(30).collect::<String>(),
    ));

    // Auto-learn: add new words from correction to vocabulary
    auto_learn_vocabulary(&old_text, &new_text);

    drop(cfg);
    save(&config);
    Ok(())
}

#[tauri::command]
pub fn get_vocabulary() -> String {
    config::load_vocabulary()
}

#[tauri::command]
pub fn set_vocabulary(content: String) -> Result<(), String> {
    let path = config::vocabulary_path();
    std::fs::write(&path, &content).map_err(|e| format!("Failed to save vocabulary: {}", e))
}

#[tauri::command]
pub fn set_hotkey(hotkey: String, app: tauri::AppHandle, config: State<'_, ConfigMutex>) {
    use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

    let old_hotkey = config.lock().unwrap().hotkey.clone();
    config.lock().unwrap().hotkey = hotkey.clone();
    save(&config);

    // Unregister old shortcut
    if let Ok(old) = old_hotkey.parse::<tauri_plugin_global_shortcut::Shortcut>() {
        let _ = app.global_shortcut().unregister(old);
    }

    // Register new shortcut
    let hotkey_for_closure = hotkey.clone();
    let state = app.state::<crate::state::SharedState>().inner().clone();
    let handle = app.clone();
    match hotkey.parse::<tauri_plugin_global_shortcut::Shortcut>() {
        Ok(shortcut) => {
            match app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    crate::log(&format!("{} pressed!", hotkey_for_closure));
                    crate::toggle_recording(&handle, &state);
                }
            }) {
                Ok(_) => crate::log(&format!("Hotkey registered: {}", hotkey)),
                Err(e) => crate::log(&format!("Hotkey registration FAILED: {}", e)),
            }
        }
        Err(e) => crate::log(&format!("Invalid hotkey '{}': {}", hotkey, e)),
    }
}

#[tauri::command]
pub fn set_theme(theme: String, config: State<'_, ConfigMutex>) {
    config.lock().unwrap().theme = theme;
    save(&config);
}

#[tauri::command]
pub fn set_dark_mode(mode: String, config: State<'_, ConfigMutex>) {
    config.lock().unwrap().dark_mode = mode;
    save(&config);
}

#[tauri::command]
pub fn detect_provider(api_key: String) -> Option<String> {
    crate::providers::detect_provider(&api_key).map(|p| p.name)
}

pub fn load_config() -> AppConfig {
    config::load_config_from_file()
}

/// Extract new words from a correction and append them to vocabulary.txt
fn auto_learn_vocabulary(old_text: &str, new_text: &str) {
    use std::collections::HashSet;

    let old_words: HashSet<&str> = old_text
        .split(|c: char| c.is_whitespace() || c == ',' || c == '.')
        .filter(|w| !w.is_empty())
        .collect();

    let new_words: Vec<&str> = new_text
        .split(|c: char| c.is_whitespace() || c == ',' || c == '.')
        .filter(|w| !w.is_empty() && !old_words.contains(w) && w.len() > 1)
        .collect();

    if new_words.is_empty() {
        return;
    }

    let vocab_path = config::vocabulary_path();
    let existing = std::fs::read_to_string(&vocab_path).unwrap_or_default();

    // Don't add words that are already in vocabulary
    let mut to_add: Vec<&str> = Vec::new();
    for word in &new_words {
        if !existing.contains(word) {
            to_add.push(word);
        }
    }

    if to_add.is_empty() {
        return;
    }

    // Append under "# Auto-learned" section
    let mut content = existing;
    if !content.contains("# Auto-learned") {
        content.push_str("\n\n# Auto-learned from corrections\n");
    }

    let learned_line = to_add.join(", ");
    content.push_str(&learned_line);
    content.push('\n');

    let _ = std::fs::write(&vocab_path, &content);
    crate::log(&format!("Vocabulary: auto-learned {} words: {}", to_add.len(), learned_line));
}
