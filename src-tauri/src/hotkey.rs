use std::sync::mpsc;
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

/// Watches for Right Command key taps using NSEvent global monitor.
pub fn start_right_cmd_watcher(tx: mpsc::Sender<()>) {
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

            // Global monitor only (fires for events in OTHER apps)
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
            loop {
                std::thread::sleep(std::time::Duration::from_secs(3600));
            }
        }
    });
}

fn handle_event(event: cocoa::base::id, tx: &mpsc::Sender<()>, state: &Arc<Mutex<HotkeyState>>) {
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
