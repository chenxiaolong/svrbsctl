# SteamVR Base Station Control

`svrbsctl` is a simple tool for controlling the operating state of a SteamVR 2.0 Base Station. It is currently able query and set the RF channel and power state. The 'identify' mode (blinking white LED) can also be set, but cannot be queried.

This project was built based on @BenWoodford's reverse engineered [protocol documentation](https://gist.github.com/BenWoodford/3a1e500a4ea2673525f5adb4120fd47c).

Note that a separate bluetooth adapter is required for `svrbsctl` to work. It is not possible to use the bluetooth functionality of the VR headset itself.

## Getting started

The project can be built by running:

```powershell
cargo build --release
```

To run `svrbsctl`:

```powershell
cargo run --release -- <args>
```

Alternatively, the executable can be directly run with:

```powershell
.\svrbsctl.exe <args>
```

The executable is located at `target\release\svrbsctl.exe`. It has no dependencies that aren't shipped with Windows so it can be copied and run from anywhere.

## Usage

To discover the available base stations, run the following command. It will print out the base station bluetooth addresses and names.

```powershell
svrbsctl discover
# Output:
# 00:11:22:33:44:55=LHB-ABCDEFGH
# aa:bb:cc:dd:ee:ff=LHB-IJKLMNOP
```

The bluetooth addresses can then be used to query the current state:

```powershell
# Get current channel
svrbsctl get channel 00:11:22:33:44:55 aa:bb:cc:dd:ee:ff
# Output:
# 00:11:22:33:44:55=1
# aa:bb:cc:dd:ee:ff=2

# Get power state
svrbsctl get power 00:11:22:33:44:55 aa:bb:cc:dd:ee:ff
# Output:
# 00:11:22:33:44:55=sleeping
# aa:bb:cc:dd:ee:ff=sleeping
```

or to set the new state:

```powershell
# Set the RF channels
svrbsctl set channel 1 00:11:22:33:44:55
svrbsctl set channel 2 aa:bb:cc:dd:ee:ff
# Turn on 'identify' mode (blinking white LED)
svrbsctl set identify on 00:11:22:33:44:55 aa:bb:cc:dd:ee:ff
# Set power mode to 'sleeping' (motors and lasers off)
svrbsctl set power sleeping 00:11:22:33:44:55 aa:bb:cc:dd:ee:ff
```

The `set` commands will not print any output unless an error occurs.

For more details, check `svrbsctl --help`.

## Caveats

* This project relies heavily on the `winrt-rs` crate. As such, Linux/bluez is not supported. I currently don't plan on supporting Linux until there's a good cross-platform BLE crate.
* Updating the base station firmware is not supported. Please use the official SteamVR tool for that.
