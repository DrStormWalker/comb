#![feature(file_create_new)]

mod config;
mod device;
mod events;
mod thread;

use events::{event_pipeline, Event};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (config_path, mut config) = config::load()?;

    let (event_pipeline_sender, event_pipeline_receiver) = event_pipeline();

    let config_watch_handle = config::watch(event_pipeline_sender.clone(), config_path)?;
    let device_watch_handle = device::watch(event_pipeline_sender.clone())?;

    let mut event_watch_handle = device::events::watch(
        event_pipeline_sender.clone(),
        evdev::enumerate().map(|(_, dev)| dev).collect(),
    )?;

    while let Ok(event) = event_pipeline_receiver.recv() {
        match event {
            Event::DeviceWatchEvent { added, removed } => {
                println!("Added devices: {:?}. Removed devices: {:?}", added, removed);

                drop(event_watch_handle);

                event_watch_handle = device::events::watch(
                    event_pipeline_sender.clone(),
                    evdev::enumerate().map(|(_, dev)| dev).collect(),
                )?;
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

    Ok(())
}
