use serde::{Deserialize, Serialize};

use crate::device::DeviceAccessor;

#[derive(Debug, Serialize, Deserialize)]
pub struct Device {
    #[serde(flatten)]
    accessor: DeviceAccessor,
}
