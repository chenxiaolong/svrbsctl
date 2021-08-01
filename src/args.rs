use btleplug::api::BDAddr;
use clap::Clap;

use crate::device::PowerState;

#[derive(Clap, Clone, Copy, Eq, PartialEq)]
pub enum ArgOnOffState {
    Off,
    On,
}

impl From<ArgOnOffState> for bool {
    fn from(state: ArgOnOffState) -> Self {
        state == ArgOnOffState::On
    }
}

#[derive(Clap, Clone, Copy, Eq, PartialEq)]
pub enum ArgPowerState {
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
pub struct Opts {
    #[clap(subcommand)]
    pub subcommand: Subcommand,
    /// Bluetooth scanning timeout (in seconds)
    #[clap(short, long, default_value = "3")]
    pub timeout: u64,
}

#[derive(Clap)]
pub enum Subcommand {
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
pub enum GetStateType {
    Channel,
    Power,
}

#[derive(Clap)]
pub struct CommandGet {
    /// The type of state to query
    #[clap(arg_enum)]
    pub state_type: GetStateType,
    /// Bluetooth addresses
    #[clap(name = "addr", required = true, min_values = 1)]
    pub addrs: Vec<BDAddr>,
}

#[derive(Clap)]
pub struct CommandSet {
    #[clap(subcommand)]
    pub subcommand: SubcommandSet,
}

#[derive(Clap)]
pub enum SubcommandSet {
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
pub struct CommandSetChannel {
    /// Channel number [valid range: 0..15 inclusive]
    pub channel: u8,
    /// Bluetooth addresses
    #[clap(name = "addr", required = true, min_values = 1)]
    pub addrs: Vec<BDAddr>,
}

#[derive(Clap)]
pub struct CommandSetIdentify {
    /// 'identify' mode state
    #[clap(arg_enum)]
    pub state: ArgOnOffState,
    /// Bluetooth addresses
    #[clap(name = "addr", required = true, min_values = 1)]
    pub addrs: Vec<BDAddr>,
}

#[derive(Clap)]
pub struct CommandSetPower {
    /// Power state
    #[clap(arg_enum)]
    pub state: ArgPowerState,
    /// Bluetooth addresses
    #[clap(name = "addr", required = true, min_values = 1)]
    pub addrs: Vec<BDAddr>,
}
