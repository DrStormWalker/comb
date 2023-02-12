#![feature(file_create_new)]
#![feature(iter_collect_into)]

mod action;
mod config;
mod device;
mod events;
mod input;
mod mio_channel;
mod thread;

use std::path::PathBuf;

use device::{events::DeviceEventWatch, open_devices, DeviceIdCombo};
use evdev::Device;
use events::{event_pipeline, Event};

use crate::device::DeviceAccessor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (config_path, mut config) = config::load()?;

    let (event_pipeline_sender, event_pipeline_receiver) = event_pipeline();

    let config_watch_handle = config::watch(event_pipeline_sender.clone(), config_path)?;
    let device_watch_handle = device::watch(event_pipeline_sender.clone())?;

    let device_event_watcher = DeviceEventWatch::new(event_pipeline_sender.clone())?;

    let xbox = DeviceAccessor::Name("Microsoft X-Box One S pad".to_string());
    let kb = DeviceAccessor::Path(PathBuf::from(
        "/dev/input/by-id/usb-BY_Tech_Usb_Gaming_Keyboard-event-kbd",
    ));

    device_event_watcher.watch(
        // evdev::enumerate()
        //     .map(|(path, dev)| DeviceIdCombo::from_accessor(DeviceAccessor::Path(path), dev))
        //     .collect(),
        open_devices(&[xbox, kb]),
    );

    while let Ok(event) = event_pipeline_receiver.recv() {
        match event {
            Event::DeviceWatchEvent { added, removed: _ } => {
                let added = added
                    .into_iter()
                    .map(|path| {
                        DeviceIdCombo::from_accessor(
                            DeviceAccessor::Path(path.clone()),
                            Device::open(path).unwrap(),
                        )
                    })
                    .collect();

                // The removed devices are automatically removed by the event stream map
                // when their event stream returns an error
                device_event_watcher.watch(added);
            }
            Event::ConfigWatchEvent(config_path) => {
                let Some(config) = config::reload(config_path)? else {
                    continue;
                };

                println!("Config file reload.\nNew config:\n{:#?}", config);
            }
            Event::DeviceEvent(_) => {}
            Event::DeviceInput(input) => {
                if input.value() == 2 {
                    continue;
                }
                println!(
                    "{} {}",
                    input.input(),
                    match input.value() {
                        0 => "released",
                        1 => "pressed",
                        2 => "repeated",
                        _ => "",
                    }
                );
            }
        }
    }

    let _ = config_watch_handle.join();
    let _ = device_watch_handle.join();
    let _ = device_event_watcher.join();

    Ok(())
}
