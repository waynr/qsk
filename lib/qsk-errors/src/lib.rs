use thiserror;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("io error")]
    IO(#[from] std::io::Error),

    #[error("io error")]
    NoSupportedKeys,

    #[error("io error")]
    NoEvents,

    #[error("time error")]
    SystemTimeError(#[from] std::time::SystemTimeError),
}
