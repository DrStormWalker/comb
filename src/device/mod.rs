pub mod events;
mod monitor;

use std::{
    ops::Deref,
    path::{Path, PathBuf},
    time::SystemTime,
};

use evdev::{EventType, InputEventKind};
pub use monitor::watch;

#[derive(Debug)]
pub struct InputEvent {
    time: SystemTime,
    kind: InputEventKind,
    value: i32,
}
impl InputEvent {
    pub fn new(kind: InputEventKind, value: i32) -> Self {
        Self {
            time: SystemTime::now(),
            kind,
            value,
        }
    }

    pub fn from_raw(event: evdev::InputEvent) -> Self {
        Self {
            time: event.timestamp(),
            kind: event.kind(),
            value: event.value(),
        }
    }

    pub fn as_raw(&self) -> evdev::InputEvent {
        let (type_, code) = match self.kind {
            InputEventKind::Synchronization(sync) => (EventType::SYNCHRONIZATION, sync.0),
            InputEventKind::Key(key) => (EventType::KEY, key.0),
            InputEventKind::RelAxis(axis) => (EventType::RELATIVE, axis.0),
            InputEventKind::AbsAxis(axis) => (EventType::ABSOLUTE, axis.0),
            InputEventKind::Misc(misc) => (EventType::MISC, misc.0),
            InputEventKind::Switch(switch) => (EventType::SWITCH, switch.0),
            InputEventKind::Led(led) => (EventType::LED, led.0),
            InputEventKind::Sound(sound) => (EventType::SOUND, sound.0),
            InputEventKind::ForceFeedback(ff) => (EventType::FORCEFEEDBACK, ff),
            InputEventKind::ForceFeedbackStatus(ffs) => (EventType::FORCEFEEDBACKSTATUS, ffs),
            InputEventKind::UInput(uinput) => (EventType::UINPUT, uinput),
            InputEventKind::Other => todo!(),
        };

        evdev::InputEvent::new(type_, code, self.value)
    }

    pub fn kind(&self) -> InputEventKind {
        self.kind
    }

    pub fn value(&self) -> i32 {
        self.value
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
