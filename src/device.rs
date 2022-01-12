use std::{
    convert::{TryFrom, TryInto},
    fmt,
};

use btleplug::{
    api::{Characteristic, Peripheral as _, WriteType},
    platform::Peripheral,
};

use crate::error::{Error, Result};

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum PowerState {
    Sleeping,
    Standby,
    Awake,
}

impl fmt::Display for PowerState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Sleeping => "sleeping",
            Self::Standby => "standby",
            Self::Awake => "awake",
        };
        write!(f, "{}", s)
    }
}

// Outputs state to be sent to device
impl From<PowerState> for &[u8] {
    fn from(value: PowerState) -> Self {
        match value {
            PowerState::Sleeping => &[0x01, 0x00],
            PowerState::Standby => &[0x02],
            PowerState::Awake => &[0x01],
        }
    }
}

// Parses state returned by device
impl TryFrom<u8> for PowerState {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        let result = match value {
            0x00 => Self::Sleeping,
            0x02 => Self::Standby,
            // Awake, but previously sleeping
            0x09 => Self::Awake,
            // Awake, but previously in standby
            0x0b => Self::Awake,
            // Unknown, but also awake
            0x01 => Self::Awake,
            s => return Err(Error::UnknownPowerState(s)),
        };

        Ok(result)
    }
}

#[derive(Debug)]
pub struct BaseStationDevice {
    peripheral: Peripheral,
    c_identify: Characteristic,
    c_mode: Characteristic,
    c_power: Characteristic,
}

impl BaseStationDevice {
    // A fast heuristic to determine if a peripheral is likely a base station
    // based on the device name. There may be false positives, but there will
    // never be false negatives.
    pub fn is_likely_base_station(name: &str) -> bool {
        name.starts_with(crate::constants::NAME_PREFIX)
    }

    pub async fn connect(peripheral: Peripheral) -> Result<Self> {
        peripheral.connect().await?;

        peripheral.discover_services().await?;

        let mut c_identify = None;
        let mut c_mode = None;
        let mut c_power = None;

        for c in peripheral.characteristics() {
            if c.uuid == crate::constants::UUID_CHARACTERISTIC_IDENTIFY {
                c_identify = Some(c);
            } else if c.uuid == crate::constants::UUID_CHARACTERISTIC_MODE {
                c_mode = Some(c);
            } else if c.uuid == crate::constants::UUID_CHARACTERISTIC_POWER {
                c_power = Some(c);
            }
        }

        let c_identify = c_identify.ok_or(Error::UnknownDevice)?;
        let c_mode = c_mode.ok_or(Error::UnknownDevice)?;
        let c_power = c_power.ok_or(Error::UnknownDevice)?;

        Ok(Self {
            peripheral,
            c_identify,
            c_mode,
            c_power,
        })
    }

    async fn read_value(&self, characteristic: &Characteristic) -> Result<u8> {
        let buf = self.peripheral.read(characteristic).await?;

        buf.into_iter().next().ok_or(Error::EmptyResponse)
    }

    async fn write_values(&self, characteristic: &Characteristic, values: &[u8]) -> Result<()> {
        // Each byte is sent as a separate command
        for value in values {
            self.peripheral.write(characteristic, &[*value], WriteType::WithoutResponse).await?;
        }

        Ok(())
    }

    pub async fn set_identify(&self, state: bool) -> Result<()> {
        self.write_values(&self.c_identify, &[state as u8]).await
    }

    pub async fn get_channel(&self) -> Result<u8> {
        let value = self.read_value(&self.c_mode).await?;

        if value >= 16 {
            return Err(Error::UnknownChannel(value));
        }

        Ok(value)
    }

    pub async fn set_channel(&self, channel: u8) -> Result<()> {
        if channel >= 16 {
            return Err(Error::UnknownChannel(channel));
        }

        self.write_values(&self.c_mode, &[channel]).await
    }

    pub async fn get_power_state(&self) -> Result<PowerState> {
        let value = self.read_value(&self.c_power).await?;

        Ok(value.try_into()?)
    }

    pub async fn set_power_state(&self, state: PowerState) -> Result<()> {
        self.write_values(&self.c_power, state.into()).await
    }
}
