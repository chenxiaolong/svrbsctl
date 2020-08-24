use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
    fmt,
    time::Duration,
};

use cancellable_timer::Timer;
use winrt::{ComInterface, Guid};

use crate::{
    bluetooth::BtAddr,
    error::{Error, Result},
    wrap::{
        ReceivedHandler,
        StoppedHandler,
        windows::{
            devices::bluetooth::{
                advertisement::{
                    BluetoothLEAdvertisementWatcher,
                    BluetoothLEScanningMode,
                },
                BluetoothLEDevice,
                generic_attribute_profile::GattCharacteristic,
            },
            storage::streams::{
                DataReader,
                DataWriter,
            },
        },
    },
};

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
    device: BluetoothLEDevice,
}

// TODO: Make everything async once winrt-rs has a stable release with async support
impl BaseStationDevice {
    pub fn discover(timeout: Duration) -> Result<HashMap<BtAddr, String>> {
        // Allow ending the scan early if an error occurs
        let (mut timer, canceller) = Timer::new2()?;
        let (tx, rx) = std::sync::mpsc::channel();

        {
            let watcher = BluetoothLEAdvertisementWatcher::new()?;
            watcher.set_scanning_mode(BluetoothLEScanningMode::Active)?;
            watcher.received(ReceivedHandler::new(move |_, event| {
                let name = event.advertisement()?.local_name()?.to_string();
                if !name.starts_with("LHB-") {
                    return Ok(());
                }

                tx.send((event.bluetooth_address()?, name)).unwrap();
                Ok(())
            }))?;
            watcher.stopped(StoppedHandler::new(move |_, _| {
                canceller.cancel().unwrap();
                Ok(())
            }))?;

            watcher.start()?;

            match timer.sleep(timeout) {
                Err(e) if e.kind() != std::io::ErrorKind::Interrupted => {
                    return Err(e.into());
                }
                _ => {}
            }

            watcher.stop()?;
        }

        // tx is dropped when watcher is dropped

        let addrs: HashMap<u64, String> = rx.iter().collect();
        let mut result = HashMap::new();

        for (addr, name) in addrs {
            let device = BluetoothLEDevice::from_bluetooth_address_async(addr)?.get()?;
            if device.is_null() {
                continue;
            }

            let services = device.get_gatt_services_for_uuid_async(
                crate::constants::SERVICE_GUID.clone())?.get()?;
            if services.services()?.size()? == 0 {
                continue;
            }

            result.insert(addr.into(), name);
        }

        Ok(result)
    }

    pub fn connect(addr: BtAddr) -> Result<Self> {
        let device = BluetoothLEDevice::from_bluetooth_address_async(addr.into())?.get()?;
        if device.is_null() {
            return Err(Error::ConnectionFailed);
        }

        Ok(Self { device })
    }

    fn get_characteristic(&self, service: &Guid, characteristic: &Guid) -> Result<GattCharacteristic> {
        let services = self.device.get_gatt_services_for_uuid_async(
            service.clone())?.get()?.services()?;
        if services.size()? == 0 {
            return Err(Error::MissingService(service.clone()));
        }

        let characteristics = services.get_at(0)?.get_characteristics_for_uuid_async(
            characteristic.clone())?.get()?.characteristics()?;
        if characteristics.size()? == 0 {
            return Err(Error::MissingCharacteristic(
                service.clone(), characteristic.clone()));
        }

        Ok(characteristics.get_at(0)?)
    }

    fn read_value(&self, service: &Guid, characteristic: &Guid) -> Result<u8> {
        let c = self.get_characteristic(service, characteristic)?;
        let buffer = c.read_value_async()?.get()?.value()?;
        let reader = DataReader::from_buffer(buffer)?;

        Ok(reader.read_byte()?)
    }

    fn write_values(&self, service: &Guid, characteristic: &Guid, values: &[u8]) -> Result<()> {
        let c = self.get_characteristic(service, characteristic)?;

        // Each byte is sent as a separate command
        for value in values {
            let writer = DataWriter::new()?;
            writer.write_byte(*value)?;

            let buffer = writer.detach_buffer()?;
            c.write_value_async(buffer)?.get()?;
        }

        Ok(())
    }

    pub fn set_identify(&self, state: bool) -> Result<()> {
        self.write_values(
            &*crate::constants::SERVICE_GUID,
            &*crate::constants::IDENTIFY_CHARACTERISTIC_GUID,
            &[state as u8]
        )
    }

    pub fn get_channel(&self) -> Result<u8> {
        let value = self.read_value(
            &*crate::constants::SERVICE_GUID,
            &*crate::constants::MODE_CHARACTERISTIC_GUID
        )?;

        if value >= 16 {
            return Err(Error::UnknownChannel(value));
        }

        Ok(value)
    }

    pub fn set_channel(&self, channel: u8) -> Result<()> {
        if channel >= 16 {
            return Err(Error::UnknownChannel(channel));
        }

        self.write_values(
            &*crate::constants::SERVICE_GUID,
            &*crate::constants::MODE_CHARACTERISTIC_GUID,
            &[channel]
        )
    }

    pub fn get_power_state(&self) -> Result<PowerState> {
        let value = self.read_value(
            &*crate::constants::SERVICE_GUID,
            &*crate::constants::POWER_CHARACTERISTIC_GUID
        )?;

        Ok(value.try_into()?)
    }

    pub fn set_power_state(&self, state: PowerState) -> Result<()> {
        self.write_values(
            &*crate::constants::SERVICE_GUID,
            &*crate::constants::POWER_CHARACTERISTIC_GUID,
            state.into()
        )
    }
}
