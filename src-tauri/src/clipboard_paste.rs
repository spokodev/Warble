use std::sync::atomic::{AtomicI32, Ordering};

static TARGET_PID: AtomicI32 = AtomicI32::new(0);

/// Save the frontmost app's PID before recording.
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

/// Copy text to clipboard and simulate Cmd+V.
pub fn copy_and_paste(text: &str) -> Result<(), String> {
    crate::log(&format!("Paste: {} chars", text.len()));

    // Set clipboard
    set_clipboard(text);

    // Post Cmd+V from main thread
    unsafe {
        dispatch::Queue::main().exec_async(|| {
            post_cmd_v();
        });
    }

    Ok(())
}

fn set_clipboard(text: &str) {
    let mut child = std::process::Command::new("pbcopy")
        .env("LANG", "en_US.UTF-8")
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
}

fn post_cmd_v() {
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

    crate::log("Paste: Cmd+V posted");
}
