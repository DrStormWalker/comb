#![feature(file_create_new)]
#![feature(iter_intersperse)]

mod action;
mod config;
mod device;
mod events;
mod mio_channel;
mod thread;

use action::{Action, ActionExecutor};
use device::{events::DeviceEventWatch, InputEvent};
use evdev::Device;
use events::{event_pipeline, Event};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (config_path, mut config) = config::load()?;

    let (event_pipeline_sender, event_pipeline_receiver) = event_pipeline();

    let config_watch_handle = config::watch(event_pipeline_sender.clone(), config_path)?;
    let device_watch_handle = device::watch(event_pipeline_sender.clone())?;

    let device_event_watcher = DeviceEventWatch::new(event_pipeline_sender)?;

    device_event_watcher.watch(evdev::enumerate().map(|(_, dev)| dev).collect());

    // let mut action_executor = ActionExecutor::new()?;

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
                let Some(config) = config::reload(config_path)? else {
                    continue;
                };

                println!("Config file reload.\nNew config:\n{:#?}", config);
            }
            Event::DeviceEvent(event) => {
                use evdev::{InputEventKind, Key};

                match event.kind() {
                    InputEventKind::Key(Key::BTN_SOUTH) => {
                        Action::InputEvents(vec![InputEvent::new(
                            InputEventKind::Key(Key::KEY_PLAYPAUSE),
                            event.value(),
                        )]);
                        // .execute(&mut action_executor)
                        // .unwrap();
                    } // _ => println!("Device event: {:?}", event),
                    _ => {}
                }
            }
        }
    }

    let _ = config_watch_handle.join();
    let _ = device_watch_handle.join();
    let _ = device_event_watcher.join();

    Ok(())
}
