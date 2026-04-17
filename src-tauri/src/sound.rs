use std::io::Write;

// Embed sound files at compile time
const SOUND_START: &[u8] = include_bytes!("../sounds/start.aiff");
const SOUND_STOP: &[u8] = include_bytes!("../sounds/stop.aiff");
const SOUND_SUCCESS: &[u8] = include_bytes!("../sounds/success.aiff");
const SOUND_ERROR: &[u8] = include_bytes!("../sounds/error.aiff");

#[derive(Debug, Clone, Copy)]
pub enum SoundType {
    Start,
    Stop,
    Success,
    Error,
}

pub fn play_sound(sound_type: SoundType) {
    std::thread::spawn(move || {
        if let Err(e) = play_sound_blocking(sound_type) {
            eprintln!("Failed to play sound {:?}: {}", sound_type, e);
        }
    });
}

fn play_sound_blocking(sound_type: SoundType) -> Result<(), String> {
    let data = match sound_type {
        SoundType::Start => SOUND_START,
        SoundType::Stop => SOUND_STOP,
        SoundType::Success => SOUND_SUCCESS,
        SoundType::Error => SOUND_ERROR,
    };

    // Write to a temp file and play with afplay (native macOS, supports AIFF)
    let temp_path = std::env::temp_dir().join(format!("warble-sound-{:?}.aiff", sound_type));
    std::fs::write(&temp_path, data)
        .map_err(|e| format!("Failed to write temp sound file: {}", e))?;

    let status = std::process::Command::new("afplay")
        .arg(&temp_path)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_err(|e| format!("Failed to run afplay: {}", e))?;

    if !status.success() {
        return Err(format!("afplay exited with status: {}", status));
    }

    Ok(())
}
