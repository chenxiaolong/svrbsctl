use btleplug::api::BDAddr;
use clap::{Parser, ValueEnum};

use crate::device::PowerState;

#[derive(Clone, Copy, Eq, PartialEq, Parser, ValueEnum)]
pub enum ArgOnOffState {
    Off,
    On,
}

impl From<ArgOnOffState> for bool {
    fn from(state: ArgOnOffState) -> Self {
        state == ArgOnOffState::On
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Parser, ValueEnum)]
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
#[derive(Parser)]
#[clap(author, about, version)]
pub struct Opts {
    #[clap(subcommand)]
    pub subcommand: Subcommand,
    /// Bluetooth scanning timeout (in seconds)
    #[clap(short, long, default_value = "10", global = true)]
    pub timeout: u64,
}

#[derive(Parser)]
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

#[derive(Clone, Copy, Eq, PartialEq, Parser, ValueEnum)]
pub enum GetStateType {
    Channel,
    Power,
}

#[derive(Parser)]
pub struct CommandGet {
    /// The type of state to query
    #[clap(value_enum)]
    pub state_type: GetStateType,
    /// Bluetooth addresses
    #[clap(name = "addr", required = true, num_args = 1..)]
    pub addrs: Vec<BDAddr>,
}

#[derive(Parser)]
pub struct CommandSet {
    #[clap(subcommand)]
    pub subcommand: SubcommandSet,
}

#[derive(Parser)]
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

#[derive(Parser)]
pub struct CommandSetChannel {
    /// Channel number [valid range: 0..15 inclusive]
    pub channel: u8,
    /// Bluetooth addresses
    #[clap(name = "addr", required = true, num_args = 1..)]
    pub addrs: Vec<BDAddr>,
}

#[derive(Parser)]
pub struct CommandSetIdentify {
    /// 'identify' mode state
    #[clap(value_enum)]
    pub state: ArgOnOffState,
    /// Bluetooth addresses
    #[clap(name = "addr", required = true, num_args = 1..)]
    pub addrs: Vec<BDAddr>,
}

#[derive(Parser)]
pub struct CommandSetPower {
    /// Power state
    #[clap(value_enum)]
    pub state: ArgPowerState,
    /// Bluetooth addresses
    #[clap(name = "addr", required = true, num_args = 1..)]
    pub addrs: Vec<BDAddr>,
}
