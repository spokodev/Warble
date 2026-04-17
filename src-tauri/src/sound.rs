// Embed sound files at compile time (WAV format — cross-platform)
const SOUND_START: &[u8] = include_bytes!("../sounds/start.wav");
const SOUND_STOP: &[u8] = include_bytes!("../sounds/stop.wav");
const SOUND_SUCCESS: &[u8] = include_bytes!("../sounds/success.wav");
const SOUND_ERROR: &[u8] = include_bytes!("../sounds/error.wav");

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

    play_sound_bytes(data, sound_type)
}

#[cfg(target_os = "macos")]
fn play_sound_bytes(data: &[u8], sound_type: SoundType) -> Result<(), String> {
    let temp_path = std::env::temp_dir().join(format!("warble-sound-{:?}.wav", sound_type));
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

#[cfg(not(target_os = "macos"))]
fn play_sound_bytes(data: &[u8], _sound_type: SoundType) -> Result<(), String> {
    use rodio::{Decoder, OutputStream, Sink};
    use std::io::Cursor;

    let (_stream, handle) = OutputStream::try_default()
        .map_err(|e| format!("No audio output device: {}", e))?;
    let sink = Sink::try_new(&handle)
        .map_err(|e| format!("Failed to create audio sink: {}", e))?;
    let source = Decoder::new(Cursor::new(data.to_vec()))
        .map_err(|e| format!("Failed to decode sound: {}", e))?;
    sink.append(source);
    sink.sleep_until_end();

    Ok(())
}
