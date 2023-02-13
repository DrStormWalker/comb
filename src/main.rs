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

use std::path::PathBuf;

use action::ActionExecutor;
use config::Config;
use device::{events::DeviceEventWatch, open_devices, path_in_devices, DeviceIdCombo};
use evdev::Device;
use events::{event_pipeline, Event};

use crate::device::{DeviceAccessor, DeviceId};

struct State {
    accessors: Vec<DeviceAccessor>,
    device_event_watch: DeviceEventWatch,
}
impl State {
    pub fn new(config: &Config, device_event_watch: DeviceEventWatch) -> Self {
        let accessors: Vec<DeviceAccessor> = config
            .devices
            .iter()
            .map(|dev| dev.accessor.clone())
            .collect();

        Self {
            accessors,
            device_event_watch,
        }
    }

    pub fn watch_devices(&self) {
        self.device_event_watch.watch(open_devices(&self.accessors));
    }

    pub fn add_devices_to_watch(&self, added: Vec<PathBuf>) {
        let added = added
            .into_iter()
            .filter_map(|path| {
                let device = Device::open(&path).ok()?;

                path_in_devices(path, &device, &self.accessors)
                    .map(|accessor| DeviceIdCombo::from_accessor(accessor.clone(), device))
            })
            .collect();

        // The removed devices are automatically removed by the event stream map
        // when their event stream returns an error
        self.device_event_watch.watch(added);
    }

    pub fn update_config(&mut self, new_config: &Config) {
        let removed: Vec<DeviceId> = self
            .accessors
            .iter()
            .map(|accessor| accessor.to_string())
            .collect();

        self.device_event_watch.unwatch(removed);

        self.accessors = new_config
            .devices
            .iter()
            .map(|dev| dev.accessor.clone())
            .collect();

        self.device_event_watch.watch(open_devices(&self.accessors));
    }

    pub fn into_device_event_watch(self) -> DeviceEventWatch {
        self.device_event_watch
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (config_path, config) = config::load()?;

    let (event_pipeline_sender, event_pipeline_receiver) = event_pipeline();

    let config_watch_handle = config::watch(event_pipeline_sender.clone(), config_path)?;
    let device_watch_handle = device::watch(event_pipeline_sender.clone())?;

    let device_event_watcher = DeviceEventWatch::new(event_pipeline_sender.clone())?;

    let mut state = State::new(&config, device_event_watcher);
    let mut action_executor = ActionExecutor::from_config(config);

    state.watch_devices();

    while let Ok(event) = event_pipeline_receiver.recv() {
        match event {
            Event::DeviceWatchEvent { added, removed: _ } => state.add_devices_to_watch(added),
            Event::ConfigWatchEvent(config_path) => {
                let Some(config) = config::reload(config_path)? else {
                    continue;
                };

                state.update_config(&config);

                action_executor.update_config(config);
            }
            Event::DeviceEvent(_) => {}
            Event::DeviceInput(input) => {
                action_executor.handle_input(input);
            }
        }
    }

    let _ = config_watch_handle.join();
    let _ = device_watch_handle.join();
    let _ = state.into_device_event_watch().join();

    Ok(())
}
