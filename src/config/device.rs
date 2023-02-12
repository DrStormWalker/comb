use serde::{Deserialize, Serialize};

use crate::{
    device::DeviceAccessor,
    input::{Input, State},
};

use super::util::display_from_str;

#[derive(Debug, Serialize, Deserialize)]
pub struct Device {
    #[serde(flatten)]
    pub accessor: DeviceAccessor,
    #[serde(default)]
    pub actions: Vec<Action>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Action {
    #[serde(with = "display_from_str")]
    pub bind: Input,
    #[serde(flatten)]
    pub action: ActionType,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged, rename_all = "snake_case")]
pub enum ActionType {
    Hook {
        #[serde(default, with = "display_from_str")]
        on: State,
        cmd: String,
    },
    Bind {
        #[serde(with = "display_from_str")]
        to: Input,
    },
    Print {
        #[serde(default, with = "display_from_str")]
        on: State,
        other: String,
    },
}
