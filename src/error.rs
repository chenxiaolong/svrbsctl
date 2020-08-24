use thiserror::Error;
use winrt::Guid;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to connect to bluetooth device")]
    ConnectionFailed,
    #[error("Could not find GATT service {0:?}")]
    MissingService(Guid),
    #[error("Could not find GATT characteristic {1:?} in service {0:?}")]
    MissingCharacteristic(Guid, Guid),
    #[error("Unknown channel {0:#x}")]
    UnknownChannel(u8),
    #[error("Unknown power state {0:#x}")]
    UnknownPowerState(u8),
    #[error("WinRT error: {0:?}")]
    WinRT(winrt::Error),
    #[error("Error: {0}")]
    Other(#[from] std::io::Error),
}

impl From<winrt::Error> for Error {
    fn from(error: winrt::Error) -> Self {
        Self::WinRT(error)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
