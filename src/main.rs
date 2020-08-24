use std::time::Duration;

use clap::Clap;

use bluetooth::BtAddr;
use device::{BaseStationDevice, PowerState};
use error::Result;

mod bluetooth;
mod constants;
mod device;
mod error;
#[allow(dead_code)]
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
}

#[derive(Clap)]
enum Subcommand {
    /// Discover SteamVR base stations
    Discover(CommandDiscover),
    /// Get state of base station
    ///
    /// The current channel and power state can be queried, but the 'identify'
    /// mode state cannot.
    Get(CommandGet),
    /// Set state of base station
    Set(CommandSet),
}

#[derive(Clap)]
struct CommandDiscover {
    /// Bluetooth scanning timeout (in seconds)
    #[clap(short, long, default_value = "3")]
    timeout: u64,
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

fn main() {
    let opts = Opts::parse();
    let mut failed = false;
    let mut handle_result = |result, addr| {
        match result {
            Ok(_) => {},
            Err(e) => {
                if let Some(a) = addr {
                    eprint!("[{}] ", a);
                }
                eprintln!("{}", e);
                failed = true;
            },
        }
    };

    match opts.subcommand {
        Subcommand::Discover(args) => {
            let duration = Duration::from_secs(args.timeout);

            handle_result(|| -> Result<()> {
                let devices = BaseStationDevice::discover(duration)?;
                for (addr, name) in devices {
                    println!("{}={}", addr, name);
                }
                Ok(())
            }(), None);
        }
        Subcommand::Get(args) => {
            for addr in &args.addrs {
                handle_result(|| -> Result<()> {
                    let device = BaseStationDevice::connect(*addr)?;

                    match args.state_type {
                        GetStateType::Channel => {
                            println!("{}={}", addr, device.get_channel()?);
                        }
                        GetStateType::Power => {
                            println!("{}={}", addr, device.get_power_state()?);
                        }
                    }

                    Ok(())
                }(), Some(addr));
            }
        }
        Subcommand::Set(args) => {
            match args.subcommand {
                SubcommandSet::Channel(sargs) => {
                    for addr in &sargs.addrs {
                        handle_result(|| -> Result<()> {
                            let device = BaseStationDevice::connect(*addr)?;
                            device.set_channel(sargs.channel)?;
                            Ok(())
                        }(), Some(addr));
                    }
                }
                SubcommandSet::Identify(sargs) => {
                    for addr in &sargs.addrs {
                        handle_result(|| -> Result<()> {
                            let device = BaseStationDevice::connect(*addr)?;
                            device.set_identify(sargs.state.into())?;
                            Ok(())
                        }(), Some(addr));
                    }
                }
                SubcommandSet::Power(sargs) => {
                    for addr in &sargs.addrs {
                        handle_result(|| -> Result<()> {
                            let device = BaseStationDevice::connect(*addr)?;
                            device.set_power_state(sargs.state.into())?;
                            Ok(())
                        }(), Some(addr));
                    }
                }
            }
        }
    }

    if failed {
        std::process::exit(1);
    }
}
