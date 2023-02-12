pub mod events;
mod monitor;

use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
    time::SystemTime,
};

use evdev::{Device, InputEventKind};
pub use events::DeviceEvent;
pub use monitor::watch;
use serde::{Deserialize, Serialize};

use crate::input::InputEvent;

pub type DeviceId = String;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DeviceAccessor {
    Name(String),
    Path(PathBuf),
}
impl DeviceAccessor {
    pub fn canonicalized(&self) -> Self {
        match self {
            // if let guards are unstable
            // (issue #51114 https://github.com/rust-lang/rust/issues/51114)
            Self::Path(path) if let Ok(path) = path.canonicalize() => {
                Self::Path(path)
            }
            _ => self.clone()
        }
    }
}
impl ToString for DeviceAccessor {
    fn to_string(&self) -> String {
        match self {
            Self::Name(name) => name.clone(),
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
            id: accessor.to_string(),
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
#[allow(unused)]
pub struct DeviceInput {
    timestamp: SystemTime,
    input_event: InputEvent,
    device: DeviceId,
}
impl DeviceInput {
    pub fn input_event(&self) -> InputEvent {
        self.input_event
    }

    pub fn device(&self) -> &str {
        &self.device
    }
}
impl TryFrom<DeviceEvent> for DeviceInput {
    type Error = ();

    fn try_from(event: DeviceEvent) -> Result<Self, Self::Error> {
        let input_event = match event.kind() {
            InputEventKind::Key(key) => {
                InputEvent::try_from_raw_key(key, event.value()).ok_or(())?
            }
            _ => return Err(()),
        };

        Ok(Self {
            timestamp: event.timestamp(),
            input_event,
            device: event.device().to_string(),
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
            device_name_matches(&device, |name| names.contains(&name))
                .map(|name| name.to_string())
                .map(|name| DeviceIdCombo::new(name, device))
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

fn device_name_matches(device: &Device, mut predicate: impl FnMut(&str) -> bool) -> Option<&str> {
    // let chains are unstable
    // (issue #53667 https://github.com/rust-lang/rust/issues/53667)
    if let Some(name) = device.name() && predicate(name) {
        Some(name)
    } else if let Some(name) = device.name() && predicate(name) {
        Some(name)
    } else {
        None
    }
}

pub fn path_in_devices<'a>(
    path: impl AsRef<Path>,
    device: &Device,
    accessors: &'a [DeviceAccessor],
) -> Option<&'a DeviceAccessor> {
    for accessor in accessors {
        match accessor {
            DeviceAccessor::Path(p) if p == path.as_ref() => return Some(accessor),
            // if let guards are unstable
            // (issue #51114 https://github.com/rust-lang/rust/issues/51114)
            DeviceAccessor::Name(name) if let Some(name) = device_name_matches(&device, |n| n == &name[..]) => {
                return Some(accessor)
            }
            _ => {}
        }
    }

    None
}
