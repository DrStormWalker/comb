#![feature(file_create_new)]
#![feature(iter_collect_into)]
#![feature(if_let_guard)]
#![feature(let_chains)]

mod action;
mod config;
mod device;
mod events;
mod input;
mod mio_channel;
mod thread;

use device::{events::DeviceEventWatch, open_devices, path_in_devices, DeviceIdCombo};
use evdev::Device;
use events::{event_pipeline, Event};

use crate::device::{DeviceAccessor, DeviceId};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (config_path, mut config) = config::load()?;

    let (event_pipeline_sender, event_pipeline_receiver) = event_pipeline();

    let config_watch_handle = config::watch(event_pipeline_sender.clone(), config_path)?;
    let device_watch_handle = device::watch(event_pipeline_sender.clone())?;

    let device_event_watcher = DeviceEventWatch::new(event_pipeline_sender.clone())?;

    // let xbox = DeviceAccessor::Name("Microsoft X-Box One S pad".to_string());
    // let kb = DeviceAccessor::Path(PathBuf::from(
    //     "/dev/input/by-id/usb-BY_Tech_Usb_Gaming_Keyboard-event-kbd",
    // ));

    let mut accessors: Vec<DeviceAccessor> = config
        .devices
        .iter()
        .map(|dev| dev.accessor.canonicalized())
        .collect();

    device_event_watcher.watch(open_devices(&accessors));

    while let Ok(event) = event_pipeline_receiver.recv() {
        match event {
            Event::DeviceWatchEvent { added, removed: _ } => {
                let added = added
                    .into_iter()
                    .filter_map(|path| {
                        let device = Device::open(&path).ok()?;

                        path_in_devices(path, &device, &accessors)
                            .map(|accessor| DeviceIdCombo::from_accessor(accessor.clone(), device))
                    })
                    .collect();

                // The removed devices are automatically removed by the event stream map
                // when their event stream returns an error
                device_event_watcher.watch(added);
            }
            Event::ConfigWatchEvent(config_path) => {
                let Some(new_config) = config::reload(config_path)? else {
                    continue;
                };

                println!("Config file reload.\nNew config:\n{:#?}", new_config);

                config = new_config;
                let new_accessors: Vec<DeviceAccessor> = config
                    .devices
                    .iter()
                    .map(|dev| dev.accessor.canonicalized())
                    .collect();

                let removed: Vec<DeviceId> = accessors
                    .iter()
                    .filter(|accessor| !new_accessors.contains(&accessor))
                    .map(|accessor| accessor.to_string())
                    .collect();

                accessors = new_accessors;

                device_event_watcher.unwatch(removed);
                device_event_watcher.watch(open_devices(&accessors));
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
