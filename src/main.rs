#![feature(file_create_new)]
#![feature(type_alias_impl_trait)]

mod config;
mod device;
mod events;
mod thread;

use device::events::DeviceEventWatcher;
use evdev::Device;
use events::{event_pipeline, Event};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (config_path, mut config) = config::load()?;

    let (event_pipeline_sender, event_pipeline_receiver) = event_pipeline();

    let config_watch_handle = config::watch(event_pipeline_sender.clone(), config_path)?;
    let device_watch_handle = device::watch(event_pipeline_sender.clone())?;

    let device_event_watcher = DeviceEventWatcher::new(event_pipeline_sender)?;

    device_event_watcher.watch(evdev::enumerate().map(|(_, dev)| dev).collect());

    while let Ok(event) = event_pipeline_receiver.recv() {
        match event {
            Event::DeviceWatchEvent { added, removed } => {
                println!("Added devices: {:?}. Removed devices: {:?}", added, removed);

                let added = added
                    .into_iter()
                    .map(|path| Device::open(path).unwrap())
                    .collect();

                // The removed devices are automatically removed by the event stream map
                // when their event stream returns an error
                device_event_watcher.watch(added);
            }
            Event::ConfigWatchEvent(config_path) => {
                config = if let Some(config) = config::reload(config_path)? {
                    config
                } else {
                    continue;
                };

                println!("Config file reload.\nNew config:\n{:#?}", config);
            }
            Event::DeviceEvent(event) => {
                println!("Device event: {:?}", event);
            }
        }
    }

    let _ = config_watch_handle.join();
    let _ = device_watch_handle.join();
    let _ = device_event_watcher.join();

    Ok(())
}
