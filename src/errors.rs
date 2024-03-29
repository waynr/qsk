use async_std;
use thiserror;

use evdev;
use evdev_rs;

use crate::events;
use crate::recorder;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("io error")]
    IO(#[from] std::io::Error),

    #[error("io error")]
    NoSupportedKeys,

    #[error("io error")]
    NoEvents,

    #[error("async error")]
    AsyncSendInputEvent(#[from] async_std::channel::SendError<events::InputEvent>),

    #[error("async error")]
    AsyncSendLog(#[from] async_std::channel::SendError<recorder::Log>),

    #[error("unrecognized InputEvent\n time: {:?}, code: {:?}, value: {:?}", .e.time, .e.code, .e.state)]
    UnrecognizedInputEvent { e: events::InputEvent },

    #[error("unrecognized evdev::InputEvent:\n time: {:?}, code: {:?}, value: {:?}", .e.timestamp(), .e.code(), .e.value())]
    UnrecognizedEvdevInputEvent { e: evdev::InputEvent },

    #[error("unrecognized evdev_rs::InputEvent:\n time: {:?}, type: {:?}, code: {:?}, value: {:?}", .e.time, .e.event_type(), .e.event_code, .e.value)]
    UnrecognizedEvdevRSInputEvent { e: evdev_rs::InputEvent },

    #[error("time error")]
    SystemTimeError(#[from] std::time::SystemTimeError),
}
