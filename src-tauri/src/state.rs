use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RecordingState {
    Idle,
    Recording,
    Stopping,
    Transcribing,
}

impl std::fmt::Display for RecordingState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecordingState::Idle => write!(f, "IDLE"),
            RecordingState::Recording => write!(f, "RECORDING"),
            RecordingState::Stopping => write!(f, "STOPPING"),
            RecordingState::Transcribing => write!(f, "TRANSCRIBING"),
        }
    }
}

pub struct AppState {
    pub state: RecordingState,
    pub last_toggle_time: Instant,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            state: RecordingState::Idle,
            last_toggle_time: Instant::now() - std::time::Duration::from_secs(10),
        }
    }

    pub fn can_toggle(&self) -> bool {
        let debounce = std::time::Duration::from_millis(600);
        self.last_toggle_time.elapsed() >= debounce
            && (self.state == RecordingState::Idle || self.state == RecordingState::Recording)
    }

    pub fn set_state(&mut self, new_state: RecordingState) -> RecordingState {
        let old = self.state;
        self.state = new_state;
        if new_state == RecordingState::Recording || new_state == RecordingState::Idle {
            self.last_toggle_time = Instant::now();
        }
        old
    }
}

pub type SharedState = Arc<Mutex<AppState>>;

pub fn new_shared_state() -> SharedState {
    Arc::new(Mutex::new(AppState::new()))
}
