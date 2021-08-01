# SteamVR Base Station Control

`svrbsctl` is a simple tool for controlling the operating state of a SteamVR 2.0 Base Station. It is currently able query and set the RF channel and power state. The 'identify' mode (blinking white LED) can also be set, but cannot be queried.

This project was built based on @BenWoodford's reverse engineered [protocol documentation](https://gist.github.com/BenWoodford/3a1e500a4ea2673525f5adb4120fd47c).

Note that a separate bluetooth adapter is required for `svrbsctl` to work. It is not possible to use the bluetooth functionality of the VR headset itself.

## Getting started

The project can be built by running:

```powershell
cargo build --release
```

On Linux, the development package for dbus needs to be installed.

To run `svrbsctl`:

```powershell
cargo run --release -- <args>
```

Alternatively, the executable can be directly run with:

```powershell
# Windows
.\svrbsctl.exe <args>
# Linux
./svrbsctl <args>
```

On Windows, the executable is located at `target\release\svrbsctl.exe`. It has no dependencies that aren't shipped with Windows so it can be copied and run from anywhere. On Linux, the executable is located at `target/release/svrbsctl` and only depends on the C library and dbus.

## Usage

To discover the available base stations, run the following command. It will print out the base station bluetooth addresses and names.

```powershell
svrbsctl discover
# Output:
# 00:11:22:33:44:55=LHB-ABCDEFGH
# AA:BB:CC:DD:EE:FF=LHB-IJKLMNOP
```

The bluetooth addresses can then be used to query the current state:

```powershell
# Get current channel
svrbsctl get channel 00:11:22:33:44:55 AA:BB:CC:DD:EE:FF
# Output:
# 00:11:22:33:44:55=1
# AA:BB:CC:DD:EE:FF=2

# Get power state
svrbsctl get power 00:11:22:33:44:55 AA:BB:CC:DD:EE:FF
# Output:
# 00:11:22:33:44:55=sleeping
# AA:BB:CC:DD:EE:FF=sleeping
```

or to set the new state:

```powershell
# Set the RF channels
svrbsctl set channel 1 00:11:22:33:44:55
svrbsctl set channel 2 AA:BB:CC:DD:EE:FF
# Turn on 'identify' mode (blinking white LED)
svrbsctl set identify on 00:11:22:33:44:55 AA:BB:CC:DD:EE:FF
# Set power mode to 'sleeping' (motors and lasers off)
svrbsctl set power sleeping 00:11:22:33:44:55 AA:BB:CC:DD:EE:FF
```

The `set` commands will not print any output unless an error occurs.

For more details, check `svrbsctl --help`.

## Caveats

* Updating the base station firmware is not supported. Please use the official SteamVR tool for that.
