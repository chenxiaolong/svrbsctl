use winrt::import;

import!(
    dependencies
        os
    types
        windows::devices::bluetooth::*
        windows::devices::bluetooth::advertisement::*
        windows::devices::bluetooth::generic_attribute_profile::*
        windows::foundation::*
        windows::storage::streams::*
);

use windows::{
    devices::bluetooth::{
        advertisement::{
            BluetoothLEAdvertisementReceivedEventArgs,
            BluetoothLEAdvertisementWatcher,
            BluetoothLEAdvertisementWatcherStoppedEventArgs,
        },
    },
    foundation::TypedEventHandler,
};

pub type ReceivedHandler = TypedEventHandler<
    BluetoothLEAdvertisementWatcher,
    BluetoothLEAdvertisementReceivedEventArgs
>;
pub type StoppedHandler = TypedEventHandler<
    BluetoothLEAdvertisementWatcher,
    BluetoothLEAdvertisementWatcherStoppedEventArgs,
>;
