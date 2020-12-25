use thiserror;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("io error")]
    IO(#[from] std::io::Error),

    #[error("time error")]
    SystemTimeError(#[from] std::time::SystemTimeError),
}
