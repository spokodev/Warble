use chrono::Local;
use std::path::{Path, PathBuf};

fn dataset_dir() -> PathBuf {
    let home = dirs::home_dir().expect("No home directory");
    home.join(".warble").join("dataset")
}

/// Save an audio+text pair to the dataset directory.
/// Returns the dataset_id (timestamp stem) for linking to history.
pub fn save_pair(audio_path: &Path, text: &str) -> Result<String, String> {
    let dir = dataset_dir();
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create dataset dir: {}", e))?;

    let dataset_id = Local::now().format("%Y%m%d-%H%M%S").to_string();
    let wav_dest = dir.join(format!("{}.wav", dataset_id));
    let txt_dest = dir.join(format!("{}.txt", dataset_id));

    std::fs::copy(audio_path, &wav_dest)
        .map_err(|e| format!("Failed to copy audio to dataset: {}", e))?;

    std::fs::write(&txt_dest, text)
        .map_err(|e| format!("Failed to write text to dataset: {}", e))?;

    Ok(dataset_id)
}

/// Update the text file for an existing dataset entry (used when user corrects transcription).
pub fn update_text(dataset_id: &str, new_text: &str) {
    let txt_path = dataset_dir().join(format!("{}.txt", dataset_id));
    if txt_path.exists() {
        let _ = std::fs::write(&txt_path, new_text);
    }
}
