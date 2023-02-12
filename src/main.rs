#![feature(file_create_new)]
#![feature(iter_collect_into)]
#![feature(if_let_guard)]
#![feature(let_chains)]

mod config;
mod device;
mod events;
mod input;
mod mio_channel;
mod thread;

use std::process::{Command, Stdio};

use device::{events::DeviceEventWatch, open_devices, path_in_devices, DeviceIdCombo};
use evdev::Device;
use events::{event_pipeline, Event};

use crate::{
    config::ActionType,
    device::{DeviceAccessor, DeviceId},
    input::State,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (config_path, mut config) = config::load()?;

    config
        .devices
        .iter_mut()
        .for_each(|device| device.accessor = device.accessor.canonicalized());

    println!("{:#?}", config);

    let (event_pipeline_sender, event_pipeline_receiver) = event_pipeline();

    let config_watch_handle = config::watch(event_pipeline_sender.clone(), config_path)?;
    let device_watch_handle = device::watch(event_pipeline_sender.clone())?;

    let device_event_watcher = DeviceEventWatch::new(event_pipeline_sender.clone())?;

    let mut accessors: Vec<DeviceAccessor> = config
        .devices
        .iter()
        .map(|dev| dev.accessor.clone())
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

                config = new_config;
                config
                    .devices
                    .iter_mut()
                    .for_each(|device| device.accessor = device.accessor.canonicalized());

                println!("Config file reload.\n{:#?}", config);

                let removed: Vec<DeviceId> = accessors
                    .iter()
                    .map(|accessor| accessor.to_string())
                    .collect();

                device_event_watcher.unwatch(removed);

                accessors = config
                    .devices
                    .iter()
                    .map(|dev| dev.accessor.clone())
                    .collect();

                device_event_watcher.watch(open_devices(&accessors));
            }
            Event::DeviceEvent(_) => {}
            Event::DeviceInput(input) => {
                if input.input_event().state() == State::Repeated {
                    continue;
                }
                println!(
                    "{} {}",
                    input.input_event().input(),
                    input.input_event().state(),
                );

                let Some(device_actions) = config.devices.iter().find(|dev| dev.accessor.to_string() == input.device()) else {
                    continue;
                };

                let actions = device_actions.actions.iter().filter(|action| {
                    if action.bind != input.input_event().input() {
                        return false;
                    }

                    match action.action {
                        ActionType::Hook { on, cmd: _ } => on == input.input_event().state(),
                        ActionType::Print { on, other: _ } => on == input.input_event().state(),
                        _ => true,
                    }
                });

                for action in actions {
                    match action.action {
                        ActionType::Hook { on: _, ref cmd } => {
                            let _ = Command::new("sh")
                                .arg("-c")
                                .arg(cmd)
                                .stdin(Stdio::null())
                                .stdout(Stdio::null())
                                .spawn();
                        }
                        _ => unimplemented!(),
                    }
                }
            }
        }
    }

    let _ = config_watch_handle.join();
    let _ = device_watch_handle.join();
    let _ = device_event_watcher.join();

    Ok(())
}
