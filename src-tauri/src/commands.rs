use crate::config::{self, AppConfig, HistoryEntry};
use crate::state::RecordingState;
use tauri::State;

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
pub fn detect_provider(api_key: String) -> Option<String> {
    crate::providers::detect_provider(&api_key).map(|p| p.name)
}

pub fn load_config() -> AppConfig {
    config::load_config_from_file()
}
