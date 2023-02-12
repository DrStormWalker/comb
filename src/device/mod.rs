pub mod events;
mod monitor;

use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
    path::PathBuf,
    time::SystemTime,
};

use evdev::{Device, InputEventKind};
pub use events::DeviceEvent;
pub use monitor::watch;

use crate::input::Input;

pub type DeviceId = String;

#[derive(Clone)]
pub enum DeviceAccessor {
    Name(String),
    Path(PathBuf),
}
impl Into<DeviceId> for DeviceAccessor {
    fn into(self) -> DeviceId {
        match self {
            Self::Name(name) => name,
            Self::Path(path) => path.to_string_lossy().to_string(),
        }
    }
}

pub struct DeviceIdCombo {
    device: Device,
    id: DeviceId,
}
impl DeviceIdCombo {
    pub fn new(id: DeviceId, device: Device) -> Self {
        Self { device, id }
    }

    pub fn from_accessor(accessor: DeviceAccessor, device: Device) -> Self {
        Self {
            device,
            id: accessor.into(),
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}
impl Deref for DeviceIdCombo {
    type Target = Device;

    fn deref(&self) -> &Self::Target {
        &self.device
    }
}
impl DerefMut for DeviceIdCombo {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.device
    }
}
impl Debug for DeviceIdCombo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DeviceIdCombo({})", self.id)
    }
}

#[derive(Debug, Clone)]
pub struct DeviceInput {
    time: SystemTime,
    input: Input,
    value: i32,
    device: DeviceId,
}
impl DeviceInput {
    pub fn input(&self) -> Input {
        self.input
    }

    pub fn value(&self) -> i32 {
        self.value
    }
}
impl TryFrom<DeviceEvent> for DeviceInput {
    type Error = ();

    fn try_from(value: DeviceEvent) -> Result<Self, Self::Error> {
        let input = match value.kind() {
            InputEventKind::Key(key) => key.into(),
            _ => return Err(()),
        };

        Ok(Self {
            time: value.time(),
            input,
            value: value.value(),
            device: value.device().to_string(),
        })
    }
}

pub fn open_devices(devices: &[DeviceAccessor]) -> Vec<DeviceIdCombo> {
    let mut opened_devices = vec![];

    let names = devices
        .iter()
        .filter_map(|dev| match dev {
            DeviceAccessor::Name(ref name) => Some(name.trim()),
            _ => None,
        })
        .collect::<Vec<&str>>();

    evdev::enumerate()
        .filter_map(|(_, device)| {
            device
                .unique_name()
                .or_else(|| device.name())
                .map(|name| name.to_string())
                .and_then(|name| {
                    if names.contains(&name.trim()) {
                        Some(DeviceIdCombo::new(name, device))
                    } else {
                        None
                    }
                })
        })
        // Iterator::collect_into is unstable
        // (issue #94780 <https://github.com/rust-lang/rust/issues/94780>)
        .collect_into(&mut opened_devices);

    devices
        .iter()
        .filter_map(|dev| match dev {
            DeviceAccessor::Path(path) => Some(DeviceIdCombo::from_accessor(
                dev.clone(),
                Device::open(path).ok()?,
            )),
            _ => None,
        })
        // Iterator::collect_into is unstable
        // (issue #94780 <https://github.com/rust-lang/rust/issues/94780>)
        .collect_into(&mut opened_devices);

    opened_devices
}
