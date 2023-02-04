pub mod events;
mod monitor;

use std::{
    ops::Deref,
    path::{Path, PathBuf},
    time::SystemTime,
};

use evdev::InputEventKind;
pub use monitor::watch;

#[derive(Debug)]
pub struct InputEvent {
    time: SystemTime,
    kind: InputEventKind,
    value: i32,
}
impl InputEvent {
    pub fn from_raw(event: evdev::InputEvent) -> Self {
        Self {
            time: event.timestamp(),
            kind: event.kind(),
            value: event.value(),
        }
    }

    pub fn kind(&self) -> InputEventKind {
        self.kind
    }
}

#[derive(Debug)]
pub struct DeviceEvent {
    device: String,
    event: InputEvent,
}
impl DeviceEvent {
    pub fn new(device: String, event: InputEvent) -> Self {
        Self { device, event }
    }

    pub fn device(&self) -> &str {
        &self.device
    }
}
impl Deref for DeviceEvent {
    type Target = InputEvent;

    fn deref(&self) -> &Self::Target {
        &self.event
    }
}
