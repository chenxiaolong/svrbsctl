use std::time::Duration;

use clap::Clap;
use thiserror::Error;

use bluetooth::BtAddr;
use device::{BaseStationDevice, PowerState};

mod bluetooth;
mod constants;
mod device;
mod error;
#[allow(dead_code, clippy::all)]
mod wrap;

#[derive(Clap, Clone, Copy, Eq, PartialEq)]
enum ArgOnOffState {
    Off,
    On,
}

impl From<ArgOnOffState> for bool {
    fn from(state: ArgOnOffState) -> Self {
        state == ArgOnOffState::On
    }
}

#[derive(Clap, Clone, Copy, Eq, PartialEq)]
enum ArgPowerState {
    Sleeping,
    Standby,
    Awake,
}

impl From<ArgPowerState> for PowerState {
    fn from(value: ArgPowerState) -> Self {
        match value {
            ArgPowerState::Sleeping => Self::Sleeping,
            ArgPowerState::Standby => Self::Standby,
            ArgPowerState::Awake => Self::Awake,
        }
    }
}

/// A simple tool for querying and setting the state of SteamVR base stations.
#[derive(Clap)]
#[clap(author, about, version)]
struct Opts {
    #[clap(subcommand)]
    subcommand: Subcommand,
    /// Bluetooth scanning timeout (in seconds)
    #[clap(short, long, default_value = "3")]
    timeout: u64,
}

#[derive(Clap)]
enum Subcommand {
    /// Discover SteamVR base stations
    Discover,
    /// Get state of base station
    ///
    /// The current channel and power state can be queried, but the 'identify'
    /// mode state cannot.
    Get(CommandGet),
    /// Set state of base station
    Set(CommandSet),
}

#[derive(Clap)]
enum GetStateType {
    Channel,
    Power,
}

#[derive(Clap)]
struct CommandGet {
    /// The type of state to query
    #[clap(arg_enum)]
    state_type: GetStateType,
    /// Bluetooth addresses
    #[clap(name = "addr", required = true, min_values = 1)]
    addrs: Vec<BtAddr>,
}

#[derive(Clap)]
struct CommandSet {
    #[clap(subcommand)]
    subcommand: SubcommandSet,
}

#[derive(Clap)]
enum SubcommandSet {
    /// Set the RF channel used by the base stations.
    Channel(CommandSetChannel),
    /// Set the state of the 'identify' mode.
    ///
    /// Note that this will turn on the base station if it is not awake.
    Identify(CommandSetIdentify),
    /// Set the power state of the base stations.
    Power(CommandSetPower),
}

#[derive(Clap)]
struct CommandSetChannel {
    /// Channel number [valid range: 0..15 inclusive]
    channel: u8,
    /// Bluetooth addresses
    #[clap(name = "addr", required = true, min_values = 1)]
    addrs: Vec<BtAddr>,
}

#[derive(Clap)]
struct CommandSetIdentify {
    /// 'identify' mode state
    #[clap(arg_enum)]
    state: ArgOnOffState,
    /// Bluetooth addresses
    #[clap(name = "addr", required = true, min_values = 1)]
    addrs: Vec<BtAddr>,
}

#[derive(Clap)]
struct CommandSetPower {
    /// Power state
    #[clap(arg_enum)]
    state: ArgPowerState,
    /// Bluetooth addresses
    #[clap(name = "addr", required = true, min_values = 1)]
    addrs: Vec<BtAddr>,
}

#[derive(Error, Debug)]
enum MainError {
    #[error("{0}")]
    Global(error::Error),
    #[error("[{0}] {1}")]
    Device(BtAddr, error::Error),
    #[error("Some devices were not found")]
    MissingDevices,
}

fn main_wrapper() -> Result<(), MainError> {
    let opts = Opts::parse();
    let timeout = Duration::from_secs(opts.timeout);
    let limit = match &opts.subcommand {
        Subcommand::Discover => None,
        Subcommand::Get(args) => Some(&args.addrs),
        Subcommand::Set(args) => {
            match &args.subcommand {
                SubcommandSet::Channel(sargs) => Some(&sargs.addrs),
                SubcommandSet::Identify(sargs) => Some(&sargs.addrs),
                SubcommandSet::Power(sargs) => Some(&sargs.addrs),
            }
        },
    }.map(|l| l.as_slice());
    let devices = BaseStationDevice::discover(timeout, limit)
        .map_err(MainError::Global)?;
    let mut missing = false;

    if let Some(l) = limit {
        if devices.len() < l.len() {
            for addr in l.iter().filter(|d| !devices.contains_key(d)) {
                eprintln!("[{}] Could not find device", addr);
            }
            missing = true;
        }
    }

    let device_error = move |addr| {
        move |e| MainError::Device(addr, e)
    };

    match &opts.subcommand {
        Subcommand::Discover => {
            for (addr, name) in devices {
                println!("{}={}", addr, name);
            }
        }
        Subcommand::Get(args) => {
            for addr in devices.keys() {
                let device = BaseStationDevice::connect(*addr)
                    .map_err(device_error(*addr))?;

                match args.state_type {
                    GetStateType::Channel => {
                        let channel = device.get_channel()
                            .map_err(device_error(*addr))?;
                        println!("{}={}", addr, channel);
                    }
                    GetStateType::Power => {
                        let state = device.get_power_state()
                            .map_err(device_error(*addr))?;
                        println!("{}={}", addr, state);
                    }
                }
            }
        }
        Subcommand::Set(args) => {
            for addr in devices.keys() {
                let device = BaseStationDevice::connect(*addr)
                    .map_err(device_error(*addr))?;

                match &args.subcommand {
                    SubcommandSet::Channel(sargs) => {
                        device.set_channel(sargs.channel)
                    }
                    SubcommandSet::Identify(sargs) => {
                        device.set_identify(sargs.state.into())
                    }
                    SubcommandSet::Power(sargs) => {
                        device.set_power_state(sargs.state.into())
                    }
                }.map_err(device_error(*addr))?;
            }
        }
    }

    if missing {
        Err(MainError::MissingDevices)
    } else {
        Ok(())
    }
}

fn main() {
    match main_wrapper() {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}
