#[macro_use]
extern crate objc;

mod audio;
mod clipboard_paste;
mod commands;
mod config;
mod dataset;
mod hotkey;
mod postprocess;
mod providers;
mod sound;
mod state;
mod transcription;
mod tray;

use state::RecordingState;
use std::sync::Mutex;
use tauri::{Emitter, Listener, Manager};

/// Log to ~/.whisperspoon/spoko-whisper.log and stderr
fn log(msg: &str) {
    let line = format!(
        "[{}] {}\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
        msg
    );
    eprint!("{}", line);
    if let Some(home) = dirs::home_dir() {
        let log_path = home.join(".whisperspoon").join("spoko-whisper.log");
        let _ = std::fs::create_dir_all(log_path.parent().unwrap());
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
        {
            let _ = f.write_all(line.as_bytes());
        }
    }
}

pub fn run() {
    log("Starting Spoko Whisper...");

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            log("Setup: loading config...");
            let config = commands::load_config();
            log(&format!("Setup: config loaded, apiKey={}, model={}, history={} entries",
                if config.api_key.is_empty() { "(empty)" } else { "(set)" },
                config.model,
                config.history.len(),
            ));
            app.manage(Mutex::new(config));

            // Initialize shared state
            let shared_state = state::new_shared_state();
            app.manage(shared_state.clone());

            // Create system tray (non-fatal if fails)
            log("Setup: creating tray icon...");
            match tray::create_tray(&app.handle()) {
                Ok(tray_icon) => {
                    log("Setup: tray icon created OK");
                    app.manage(Mutex::new(Some(tray_icon)));
                }
                Err(e) => {
                    log(&format!("Setup: tray icon FAILED (non-fatal): {}", e));
                    app.manage(Mutex::new(None::<tauri::tray::TrayIcon>));
                }
            };

            // Store recorder reference
            let recorder: Mutex<Option<audio::AudioRecorder>> = Mutex::new(None);
            app.manage(recorder);

            // Listen for toggle events (from tray menu or frontend)
            let handle = app.handle().clone();
            let state_for_tray = shared_state.clone();
            app.handle().listen("toggle-recording", move |_event| {
                toggle_recording(&handle, &state_for_tray);
            });

            // Register F5 global shortcut
            log("Setup: registering F5 shortcut...");
            let handle2 = app.handle().clone();
            let state_for_f5 = shared_state.clone();
            use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
            match app.global_shortcut().on_shortcut("F5", move |_app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    log("F5 pressed!");
                    toggle_recording(&handle2, &state_for_f5);
                }
            }) {
                Ok(_) => log("Setup: F5 shortcut registered OK"),
                Err(e) => log(&format!("Setup: F5 shortcut FAILED: {}", e)),
            }

            // Start Right Command key watcher
            log("Setup: starting Right Command watcher...");
            let (tx, rx) = std::sync::mpsc::channel();
            hotkey::start_right_cmd_watcher(tx);

            let handle3 = app.handle().clone();
            let state_for_cmd = shared_state.clone();
            std::thread::spawn(move || {
                while let Ok(()) = rx.recv() {
                    toggle_recording(&handle3, &state_for_cmd);
                }
            });

            log("Setup: complete!");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_state,
            commands::get_config,
            commands::set_api_key,
            commands::set_language,
            commands::set_model,
            commands::set_sounds_enabled,
            commands::set_auto_paste,
            commands::set_dataset_collection,
            commands::get_history,
            commands::clear_history,
            commands::delete_history_entry,
            commands::update_history_entry,
            commands::get_vocabulary,
            commands::set_vocabulary,
            commands::detect_provider,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn toggle_recording(app: &tauri::AppHandle, shared_state: &state::SharedState) {
    let current_state = {
        let state = shared_state.lock().unwrap();
        if !state.can_toggle() {
            return;
        }
        state.state
    };

    match current_state {
        RecordingState::Idle => start_recording(app, shared_state),
        RecordingState::Recording => stop_recording(app, shared_state),
        _ => {}
    }
}

fn start_recording(app: &tauri::AppHandle, shared_state: &state::SharedState) {
    // Save which app is in foreground BEFORE we start (paste target)
    // Must run on main thread for NSWorkspace
    unsafe { dispatch::Queue::main().exec_sync(|| clipboard_paste::save_frontmost_app_pid()) };
    log("Recording: starting...");
    shared_state
        .lock()
        .unwrap()
        .set_state(RecordingState::Recording);
    update_tray(app, RecordingState::Recording);

    let config = app.state::<Mutex<config::AppConfig>>();
    let sounds_enabled = config.lock().unwrap().sounds_enabled;
    if sounds_enabled {
        sound::play_sound(sound::SoundType::Start);
    }

    let audio_dir = dirs::home_dir()
        .expect("No home dir")
        .join(".whisperspoon");
    let _ = std::fs::create_dir_all(&audio_dir);
    let audio_path = audio_dir.join("audio.wav");

    match audio::AudioRecorder::start(audio_path) {
        Ok(recorder) => {
            log("Recording: audio capture started");
            let rec_state = app.state::<Mutex<Option<audio::AudioRecorder>>>();
            *rec_state.lock().unwrap() = Some(recorder);
        }
        Err(e) => {
            log(&format!("Recording: FAILED to start: {}", e));
            shared_state
                .lock()
                .unwrap()
                .set_state(RecordingState::Idle);
            update_tray(app, RecordingState::Idle);
            if sounds_enabled {
                sound::play_sound(sound::SoundType::Error);
            }
        }
    }
}

fn stop_recording(app: &tauri::AppHandle, shared_state: &state::SharedState) {
    log("Recording: stopping...");
    shared_state
        .lock()
        .unwrap()
        .set_state(RecordingState::Stopping);
    update_tray(app, RecordingState::Stopping);

    let config_state = app.state::<Mutex<config::AppConfig>>();
    let sounds_enabled = config_state.lock().unwrap().sounds_enabled;
    if sounds_enabled {
        sound::play_sound(sound::SoundType::Stop);
    }

    let recorder = {
        let rec_state = app.state::<Mutex<Option<audio::AudioRecorder>>>();
        let mut guard = rec_state.lock().unwrap();
        guard.take()
    };

    let Some(recorder) = recorder else {
        log("Recording: no recorder found");
        shared_state
            .lock()
            .unwrap()
            .set_state(RecordingState::Idle);
        update_tray(app, RecordingState::Idle);
        return;
    };

    let app_handle = app.clone();
    let state_clone = shared_state.clone();

    tauri::async_runtime::spawn(async move {
        let audio_path = match tokio::task::spawn_blocking(move || recorder.stop()).await {
            Ok(Ok(path)) => {
                log("Recording: audio file saved");
                path
            }
            Ok(Err(e)) => {
                log(&format!("Recording: stop FAILED: {}", e));
                handle_error(&app_handle, &state_clone);
                return;
            }
            Err(e) => {
                log(&format!("Recording: task panicked: {}", e));
                handle_error(&app_handle, &state_clone);
                return;
            }
        };

        state_clone
            .lock()
            .unwrap()
            .set_state(RecordingState::Transcribing);
        update_tray(&app_handle, RecordingState::Transcribing);

        let (api_key, language, model, sounds_enabled, auto_paste, dataset_enabled) = {
            let cfg = app_handle.state::<Mutex<config::AppConfig>>();
            let c = cfg.lock().unwrap();
            (
                c.api_key.clone(),
                c.language.clone(),
                c.model.clone(),
                c.sounds_enabled,
                c.auto_paste,
                c.dataset_collection_enabled,
            )
        };

        if api_key.is_empty() {
            log("Transcription: no API key configured!");
            if sounds_enabled {
                sound::play_sound(sound::SoundType::Error);
            }
            state_clone
                .lock()
                .unwrap()
                .set_state(RecordingState::Idle);
            update_tray(&app_handle, RecordingState::Idle);
            return;
        }

        let vocabulary = config::load_vocabulary();
        log("Transcription: sending to API...");

        match transcription::transcribe(&audio_path, &api_key, &language, &vocabulary, &model).await {
            Ok(raw_text) => {
                let text = postprocess::cleanup(&raw_text);
                log(&format!("Transcription: OK, {} chars (raw {})", text.len(), raw_text.len()));

                // Save to dataset and get ID for linking
                let dataset_id = if dataset_enabled {
                    match dataset::save_pair(&audio_path, &text) {
                        Ok(id) => Some(id),
                        Err(e) => {
                            log(&format!("Dataset: save FAILED: {}", e));
                            None
                        }
                    }
                } else {
                    None
                };

                // Add to history and save config to file
                {
                    let cfg = app_handle.state::<Mutex<config::AppConfig>>();
                    let mut c = cfg.lock().unwrap();
                    c.add_history(text.clone(), dataset_id);
                    config::save_config_to_file(&c);
                }

                if auto_paste {
                    if let Err(e) = clipboard_paste::copy_and_paste(&text) {
                        log(&format!("Paste: FAILED: {}", e));
                    }
                } else {
                    // Just copy to clipboard, don't paste
                    let mut child = std::process::Command::new("pbcopy")
                        .stdin(std::process::Stdio::piped())
                        .spawn()
                        .ok();
                    if let Some(ref mut c) = child {
                        use std::io::Write;
                        if let Some(ref mut stdin) = c.stdin {
                            let _ = stdin.write_all(text.as_bytes());
                        }
                        let _ = c.wait();
                    }
                    log("Clipboard: text copied (auto-paste off)");
                }

                if sounds_enabled {
                    sound::play_sound(sound::SoundType::Success);
                }

                // Notify frontend
                let _ = app_handle.emit("transcription-done", text);
            }
            Err(e) => {
                log(&format!("Transcription: FAILED: {}", e));
                if sounds_enabled {
                    sound::play_sound(sound::SoundType::Error);
                }
                let _ = app_handle.emit("transcription-error", e);
            }
        }

        state_clone
            .lock()
            .unwrap()
            .set_state(RecordingState::Idle);
        update_tray(&app_handle, RecordingState::Idle);
    });
}

fn handle_error(app: &tauri::AppHandle, shared_state: &state::SharedState) {
    let config = app.state::<Mutex<config::AppConfig>>();
    let sounds_enabled = config.lock().unwrap().sounds_enabled;
    if sounds_enabled {
        sound::play_sound(sound::SoundType::Error);
    }
    shared_state
        .lock()
        .unwrap()
        .set_state(RecordingState::Idle);
    update_tray(app, RecordingState::Idle);
}

fn update_tray(app: &tauri::AppHandle, state: RecordingState) {
    let tray_opt = app.state::<Mutex<Option<tauri::tray::TrayIcon>>>();
    if let Ok(guard) = tray_opt.lock() {
        if let Some(ref t) = *guard {
            let _ = tray::update_tray_state(t, state);
        }
    }
    let _ = app.emit("state-changed", state);
}
