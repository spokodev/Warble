use std::sync::atomic::{AtomicI32, Ordering};

static TARGET_PID: AtomicI32 = AtomicI32::new(0);

/// Save the frontmost app's PID before recording (macOS only).
#[cfg(target_os = "macos")]
pub fn save_frontmost_app_pid() {
    unsafe {
        let workspace: cocoa::base::id = msg_send![class!(NSWorkspace), sharedWorkspace];
        let app: cocoa::base::id = msg_send![workspace, frontmostApplication];
        if !app.is_null() {
            let pid: i32 = msg_send![app, processIdentifier];
            TARGET_PID.store(pid, Ordering::SeqCst);
            crate::log(&format!("Paste: target pid={}", pid));
        }
    }
}

#[cfg(not(target_os = "macos"))]
pub fn save_frontmost_app_pid() {}

/// Copy text to clipboard and simulate paste.
pub fn copy_and_paste(text: &str) -> Result<(), String> {
    crate::log(&format!("Paste: {} chars", text.len()));
    set_clipboard(text)?;
    std::thread::sleep(std::time::Duration::from_millis(50));
    simulate_paste();
    Ok(())
}

// ── Clipboard ──

#[cfg(target_os = "macos")]
pub fn set_clipboard(text: &str) -> Result<(), String> {
    // pbcopy is thread-safe (subprocess) — works from any thread
    let mut child = std::process::Command::new("pbcopy")
        .env("LANG", "en_US.UTF-8")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to run pbcopy: {}", e))?;
    if let Some(ref mut stdin) = child.stdin {
        use std::io::Write;
        let _ = stdin.write_all(text.as_bytes());
    }
    let _ = child.wait();
    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn set_clipboard(text: &str) -> Result<(), String> {
    // arboard is cross-platform and doesn't need external tools
    let mut clipboard = arboard::Clipboard::new()
        .map_err(|e| format!("Failed to access clipboard: {}", e))?;
    clipboard.set_text(text)
        .map_err(|e| format!("Failed to set clipboard text: {}", e))?;
    Ok(())
}

// ── Paste simulation ──

#[cfg(target_os = "macos")]
fn simulate_paste() {
    // CoreGraphics CGEvent is proven reliable on macOS — use it directly
    dispatch::Queue::main().exec_async(|| {
        post_cmd_v_cg();
    });
}

#[cfg(not(target_os = "macos"))]
fn simulate_paste() {
    use enigo::{Direction, Enigo, Key, Keyboard, Settings};

    let result = (|| -> Result<(), String> {
        let mut enigo = Enigo::new(&Settings::default())
            .map_err(|e| format!("enigo init: {}", e))?;
        enigo.key(Key::Control, Direction::Press)
            .map_err(|e| format!("ctrl press: {}", e))?;
        enigo.key(Key::Unicode('v'), Direction::Click)
            .map_err(|e| format!("v click: {}", e))?;
        enigo.key(Key::Control, Direction::Release)
            .map_err(|e| format!("ctrl release: {}", e))?;
        Ok(())
    })();

    match result {
        Ok(()) => crate::log("Paste: Ctrl+V via enigo"),
        Err(e) => crate::log(&format!("Paste: failed: {}", e)),
    }
}

#[cfg(target_os = "macos")]
fn post_cmd_v_cg() {
    use core_graphics::event::{CGEvent, CGEventFlags, CGKeyCode};
    use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

    const KEY_V: CGKeyCode = 9;

    let Ok(source) = CGEventSource::new(CGEventSourceStateID::Private) else {
        crate::log("Paste: CGEventSource failed");
        return;
    };

    let Ok(down) = CGEvent::new_keyboard_event(source.clone(), KEY_V, true) else {
        crate::log("Paste: key down event failed");
        return;
    };
    down.set_flags(CGEventFlags::CGEventFlagCommand);
    down.post(core_graphics::event::CGEventTapLocation::AnnotatedSession);

    let Ok(up) = CGEvent::new_keyboard_event(source, KEY_V, false) else {
        crate::log("Paste: key up event failed");
        return;
    };
    up.set_flags(CGEventFlags::CGEventFlagCommand);
    up.post(core_graphics::event::CGEventTapLocation::AnnotatedSession);

    crate::log("Paste: Cmd+V via CoreGraphics");
}
