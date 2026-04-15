use crate::state::RecordingState;
use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    AppHandle, Emitter, Manager,
};

const TRAY_IDLE: &[u8] = include_bytes!("../icons/tray-idle.png");
const TRAY_RECORDING: &[u8] = include_bytes!("../icons/tray-recording.png");
const TRAY_TRANSCRIBING: &[u8] = include_bytes!("../icons/tray-transcribing.png");

pub fn create_tray(app: &AppHandle) -> Result<tauri::tray::TrayIcon, Box<dyn std::error::Error>> {
    let toggle = MenuItem::with_id(app, "toggle", "Start Recording (F5 / Right ⌘)", true, None::<&str>)?;
    let separator1 = MenuItem::with_id(app, "sep1", "", false, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Settings...", true, None::<&str>)?;
    let history = MenuItem::with_id(app, "history", "History", true, None::<&str>)?;
    let separator2 = MenuItem::with_id(app, "sep2", "", false, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Spoko Whisper", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&toggle, &separator1, &settings, &history, &separator2, &quit])?;

    let tray = TrayIconBuilder::new()
        .icon(Image::from_bytes(TRAY_IDLE)?)
        .icon_as_template(false)
        .tooltip("Spoko Whisper — Idle")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(move |app, event| match event.id().as_ref() {
            "toggle" => {
                let _ = app.emit("toggle-recording", ());
            }
            "settings" => {
                open_window(app, "settings", "Settings", 400.0, 600.0);
            }
            "history" => {
                open_window(app, "history", "History", 500.0, 700.0);
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;

    Ok(tray)
}

pub fn update_tray_state(
    tray: &tauri::tray::TrayIcon,
    state: RecordingState,
) -> Result<(), Box<dyn std::error::Error>> {
    let (icon_data, tooltip, is_template) = match state {
        RecordingState::Idle => (TRAY_IDLE, "Spoko Whisper — Idle", true),
        RecordingState::Recording => (TRAY_RECORDING, "Spoko Whisper — Recording...", false),
        RecordingState::Stopping => (TRAY_RECORDING, "Spoko Whisper — Stopping...", false),
        RecordingState::Transcribing => (TRAY_TRANSCRIBING, "Spoko Whisper — Transcribing...", true),
    };

    tray.set_icon(Some(Image::from_bytes(icon_data)?))?;
    tray.set_icon_as_template(is_template)?;
    tray.set_tooltip(Some(tooltip))?;

    Ok(())
}

fn open_window(app: &AppHandle, label: &str, title: &str, width: f64, height: f64) {
    if let Some(window) = app.get_webview_window(label) {
        let _ = window.show();
        let _ = window.set_focus();
    } else {
        let _ = tauri::WebviewWindowBuilder::new(app, label, tauri::WebviewUrl::App("index.html".into()))
            .title(title)
            .inner_size(width, height)
            .resizable(label == "history")
            .build();
    }
}
