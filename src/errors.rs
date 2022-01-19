use thiserror;

use evdev;

use crate::events;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("io error")]
    IO(#[from] std::io::Error),

    #[error("io error")]
    NoSupportedKeys,

    #[error("io error")]
    NoEvents,

    #[error("unrecognized InputEvent\n code: {:?} value: {:?}", .e.code, .e.state)]
    UnrecognizedInputEvent{
        e: events::InputEvent,
    },

    #[error("unrecognized evdev::InputEvent:\n time: {:?}, type: {:?} code: {:?} value: {:?}", .e.timestamp(), .e.event_type(), .e.code(), .e.value())]
    UnrecognizedEvdevInputEvent{
        e: evdev::InputEvent,
    },

    #[error("time error")]
    SystemTimeError(#[from] std::time::SystemTimeError),
}
