use rodio::{Decoder, OutputStream, Sink};
use std::io::Cursor;

// Embed macOS system sounds at compile time
// These will be bundled into the binary
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

    let (_stream, stream_handle) =
        OutputStream::try_default().map_err(|e| format!("No audio output: {}", e))?;

    let cursor = Cursor::new(data);
    let source =
        Decoder::new(cursor).map_err(|e| format!("Failed to decode sound: {}", e))?;

    let sink = Sink::try_new(&stream_handle).map_err(|e| format!("Failed to create sink: {}", e))?;
    sink.append(source);
    sink.sleep_until_end();

    Ok(())
}
