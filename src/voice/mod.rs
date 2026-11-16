//! Local English speech. Native inference runs on a bounded blocking worker.
mod models;
mod recognition;
mod synthesis;

pub use models::check_models;
pub use recognition::{detect_wake, listen, transcribe};
pub use synthesis::{speak, Speaker};

pub enum VoiceEvent {
    Ready,
    Awake,
    Partial(String),
    Final(String),
}
