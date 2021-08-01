#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("No bluetooth adapters found")]
    NoBluetoothAdapter,
    #[error("Unknown device")]
    UnknownDevice,
    #[error("No data in response")]
    EmptyResponse,
    #[error("Unknown channel {0:#x}")]
    UnknownChannel(u8),
    #[error("Unknown power state {0:#x}")]
    UnknownPowerState(u8),
    #[error("Bluetooth error: {0}")]
    Bluetooth(#[from] btleplug::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
