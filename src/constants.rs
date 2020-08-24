use lazy_static::lazy_static;
use winrt::Guid;

// See: https://gist.github.com/BenWoodford/3a1e500a4ea2673525f5adb4120fd47c

lazy_static! {
    pub static ref SERVICE_GUID: Guid =
        Guid::from("00001523-1212-EFDE-1523-785FEABCD124");
    pub static ref IDENTIFY_CHARACTERISTIC_GUID: Guid =
        Guid::from("00008421-1212-EFDE-1523-785FEABCD124");
    pub static ref MODE_CHARACTERISTIC_GUID: Guid =
        Guid::from("00001524-1212-EFDE-1523-785FEABCD124");
    pub static ref POWER_CHARACTERISTIC_GUID: Guid =
        Guid::from("00001525-1212-EFDE-1523-785FEABCD124");
}
