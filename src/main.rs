#![feature(file_create_new)]

mod config;
mod device;
mod events;
mod thread;

use events::{event_pipeline, Event};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (config_path, mut config) = match config::load() {
        Ok(result) => result,
        Err(err) => {
            println!("{}", err);
            panic!();
        }
    };

    let (event_pipeline_sender, event_pipeline_receiver) = event_pipeline();

    let config_watch_handle = config::watch(event_pipeline_sender.clone(), config_path)?;
    let device_watch_handle = device::watch(event_pipeline_sender.clone())?;

    let mut event_watch_handle =
        device::watch_input_events(evdev::enumerate().map(|(_, dev)| dev).collect())?;

    while let Ok(event) = event_pipeline_receiver.recv() {
        match event {
            Event::DeviceWatchEvent { added, removed } => {
                println!("Added devices: {:?}. Removed devices: {:?}", added, removed);

                drop(event_watch_handle);

                event_watch_handle =
                    device::watch_input_events(evdev::enumerate().map(|(_, dev)| dev).collect())?;
            }
            Event::ConfigWatchEvent(config_path) => {
                config = config::reload(config_path).unwrap();

                println!("Config file reload.\nNew config:\n{:#?}", config);
            }
        }
    }

    let _ = config_watch_handle.join();
    let _ = device_watch_handle.join();

    Ok(())
}
