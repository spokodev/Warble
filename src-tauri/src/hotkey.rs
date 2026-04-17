use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc;

static PAUSED: AtomicBool = AtomicBool::new(false);
/// Timestamp (ms since epoch) when the watcher was last resumed — used for grace period.
static RESUMED_AT: AtomicU64 = AtomicU64::new(0);
/// Grace period after resume: ignore taps for 1 second to prevent the "selecting" tap from triggering.
const RESUME_GRACE_MS: u128 = 1000;

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Pause/resume the Right Command watcher.
pub fn set_paused(paused: bool) {
    PAUSED.store(paused, Ordering::SeqCst);
    if !paused {
        RESUMED_AT.store(now_ms(), Ordering::SeqCst);
    }
    crate::log(&format!("Hotkey watcher: {}", if paused { "PAUSED" } else { "RESUMED" }));
}

fn is_active() -> bool {
    if PAUSED.load(Ordering::SeqCst) {
        return false;
    }
    // Grace period after resume
    let resumed = RESUMED_AT.load(Ordering::SeqCst) as u128;
    let elapsed = (now_ms() as u128).saturating_sub(resumed);
    elapsed >= RESUME_GRACE_MS
}

/// Start a platform-specific modifier key watcher.
/// macOS: Right Command tap. Other platforms: no-op (use global shortcut plugin instead).
#[cfg(target_os = "macos")]
pub fn start_right_cmd_watcher(tx: mpsc::Sender<()>) {
    use std::sync::{Arc, Mutex};
    use std::time::Instant;

    const RIGHT_COMMAND_KEYCODE: u16 = 54;
    const DEBOUNCE_MS: u128 = 600;

    struct HotkeyState {
        right_cmd_down: bool,
        used_as_modifier: bool,
        down_time: Instant,
        last_toggle_time: Instant,
    }

    fn handle_event(
        event: cocoa::base::id,
        tx: &mpsc::Sender<()>,
        state: &Arc<Mutex<HotkeyState>>,
    ) {
        unsafe {
            let event_type: u64 = msg_send![event, type];
            let keycode: u16 = msg_send![event, keyCode];

            if event_type == 12 {
                // FlagsChanged
                if keycode == RIGHT_COMMAND_KEYCODE {
                    let modifier_flags: u64 = msg_send![event, modifierFlags];
                    let cmd_pressed = (modifier_flags & (1 << 20)) != 0;

                    let mut s = state.lock().unwrap();
                    if cmd_pressed {
                        s.right_cmd_down = true;
                        s.used_as_modifier = false;
                        s.down_time = Instant::now();
                    } else if s.right_cmd_down {
                        let since_last = s.last_toggle_time.elapsed().as_millis();
                        if !s.used_as_modifier
                            && s.down_time.elapsed().as_secs_f64() < 2.0
                            && since_last >= DEBOUNCE_MS
                            && is_active()
                        {
                            crate::log("Hotkey: Right ⌘ tap → toggle");
                            let _ = tx.send(());
                            s.last_toggle_time = Instant::now();
                        }
                        s.right_cmd_down = false;
                        s.used_as_modifier = false;
                    }
                }
            } else if event_type == 10 {
                // KeyDown
                let mut s = state.lock().unwrap();
                if s.right_cmd_down {
                    s.used_as_modifier = true;
                }
            }
        }
    }

    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(500));

        let state = Arc::new(Mutex::new(HotkeyState {
            right_cmd_down: false,
            used_as_modifier: false,
            down_time: Instant::now(),
            last_toggle_time: Instant::now() - std::time::Duration::from_secs(10),
        }));

        unsafe {
            use cocoa::base::{id, nil};
            use cocoa::foundation::NSUInteger;

            // NSEventMaskFlagsChanged | NSEventMaskKeyDown
            let mask: NSUInteger = (1 << 12) | (1 << 10);

            // Global monitor (fires for events in OTHER apps)
            let tx_g = tx.clone();
            let state_g = state.clone();
            let global_block = block::ConcreteBlock::new(move |event: id| {
                handle_event(event, &tx_g, &state_g);
            });
            let global_block = global_block.copy();
            let _: id = msg_send![
                class!(NSEvent),
                addGlobalMonitorForEventsMatchingMask: mask
                handler: &*global_block
            ];

            // Local monitor (fires for events in OUR app)
            let tx_l = tx;
            let state_l = state;
            let local_block = block::ConcreteBlock::new(move |event: id| -> id {
                handle_event(event, &tx_l, &state_l);
                event
            });
            let local_block = local_block.copy();
            let _: id = msg_send![
                class!(NSEvent),
                addLocalMonitorForEventsMatchingMask: mask
                handler: &*local_block
            ];

            crate::log("Hotkey: NSEvent monitors registered");

            // Prevent thread from exiting (monitors are attached to main run loop)
            let _ = nil; // suppress unused warning
            loop {
                std::thread::sleep(std::time::Duration::from_secs(3600));
            }
        }
    });
}

/// No-op on non-macOS: Right Command key doesn't exist.
/// Users rely on the global shortcut plugin (F5, etc.) instead.
#[cfg(not(target_os = "macos"))]
pub fn start_right_cmd_watcher(_tx: mpsc::Sender<()>) {
    crate::log("Hotkey: Right ⌘ watcher not available on this platform");
}
