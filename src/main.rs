use std::{
    collections::HashSet,
    sync::Arc,
    time::Duration,
};

use btleplug::{
    api::{Central, CentralEvent, Manager as _, Peripheral as _, ScanFilter},
    platform::{Manager, Peripheral},
};
use clap::Parser;
use futures::stream::StreamExt;
use tokio::task::JoinSet;

use args::{GetStateType, Opts, Subcommand, SubcommandSet};
use device::BaseStationDevice;
use error::Error;

mod args;
mod constants;
mod device;
mod error;

#[derive(Debug, thiserror::Error)]
enum MainError {
    // Top level errors returned by '?'
    #[error("{0}")]
    Unprinted(#[from] error::Error),
    // Errors from spawned tasks, which are already eprintln'd
    #[error("")]
    AlreadyPrinted,
}

impl From<btleplug::Error> for MainError {
    fn from(error: btleplug::Error) -> Self {
        Error::from(error).into()
    }
}

async fn process_device(
    peripheral: Peripheral,
    opts: Arc<Opts>,
) -> Result<(), error::Error> {

    let Some(name) = peripheral.properties().await?
        .and_then(|p| p.local_name) else { return Err(Error::UnknownDevice) };

    // Filter out non-base-stations based on the name to avoid unnecessary connections
    if !BaseStationDevice::is_likely_base_station(&name) {
        return Err(Error::UnknownDevice);
    }

    let addr = peripheral.address();
    let device = BaseStationDevice::connect(peripheral).await?;

    match &opts.subcommand {
        Subcommand::Discover => {
            println!("{addr}={name}");
        }
        Subcommand::Get(args) => {
            match args.state_type {
                GetStateType::Channel => {
                    let channel = device.get_channel().await?;
                    println!("{addr}={channel}");
                }
                GetStateType::Power => {
                    let state = device.get_power_state().await?;
                    println!("{addr}={state}");
                }
            }
        }
        Subcommand::Set(args) => {
            match &args.subcommand {
                SubcommandSet::Channel(sargs) => {
                    device.set_channel(sargs.channel).await?;
                }
                SubcommandSet::Identify(sargs) => {
                    device.set_identify(sargs.state.into()).await?;
                }
                SubcommandSet::Power(sargs) => {
                    device.set_power_state(sargs.state.into()).await?;
                }
            }
        }
    }

    Ok(())
}

async fn main_wrapper() -> Result<(), MainError> {
    let opts = Arc::new(Opts::parse());
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
    }.map(Vec::as_slice);

    let manager = Manager::new().await?;
    let adapters = manager.adapters().await?;
    let adapter = adapters.into_iter().next()
        .ok_or(Error::NoBluetoothAdapter)?;

    let mut events = adapter.events().await?;
    let filter = ScanFilter {
        services: vec![constants::UUID_SERVICE],
    };
    adapter.start_scan(filter).await?;

    let mut tasks = JoinSet::new();
    let mut remaining = limit.map(|s| s.iter().copied().collect::<HashSet<_>>());
    let discovery_timer = tokio::time::sleep(timeout);
    tokio::pin!(discovery_timer);

    // Start discovery and trigger actions
    loop {
        tokio::select! {
            // Discovery timeout exceeded
            _ = &mut discovery_timer => {
                break;
            }

            // Received bluetooth event
            e = events.next() => {
                match e {
                    Some(CentralEvent::DeviceDiscovered(id)) => {
                        let peripheral = adapter.peripheral(&id).await
                            .expect("Can't get peripheral for discovered device");
                        let addr = peripheral.address();

                        if let Some(r) = &mut remaining {
                            if !r.remove(&addr) {
                                // Not in the user-provided addresses
                                continue;
                            }
                        }

                        let opts = opts.clone();

                        tasks.spawn(async move {
                            let is_discovery = matches!(opts.subcommand, Subcommand::Discover);

                            // Returns a simple bool because btleplug::Error is not Send + Sync
                            match process_device(peripheral, opts).await {
                                Ok(()) => true,
                                Err(e) => {
                                    // We want to fail if the user specifies a valid, but
                                    // non-base-station address
                                    if matches!(e, Error::UnknownDevice) && is_discovery {
                                        true
                                    } else {
                                        eprintln!("[{addr}] {e}");
                                        false
                                    }
                                }
                            }
                        });

                        // All user-provided addresses have been discovered
                        if let Some(r) = &mut remaining {
                            if r.is_empty() {
                                break;
                            }
                        }
                    }
                    Some(_) => {}
                    // No bluetooth events left
                    None => break,
                }
            }
        }
    }

    adapter.stop_scan().await?;

    let mut error_occurred = false;

    while let Some(r) = tasks.join_next().await {
        match r {
            Ok(true) => {}
            Ok(false) => error_occurred = true,
            Err(e) => {
                eprintln!("Unexpected panic: {e}");
                error_occurred = true;
            }
        }
    }

    if let Some(addrs) = remaining {
        if !addrs.is_empty() {
            for addr in addrs {
                eprintln!("[{addr}] Could not find device");
            }
            error_occurred = true;
        }
    }

    if error_occurred {
        Err(MainError::AlreadyPrinted)
    } else {
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    match main_wrapper().await {
        Ok(_) => {}
        Err(e) => {
            match e {
                MainError::Unprinted(e) => eprintln!("{e}"),
                MainError::AlreadyPrinted => {}
            }
            std::process::exit(1);
        }
    }
}
