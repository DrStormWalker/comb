use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Device {
    #[serde(flatten)]
    accessor: Accessor,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Accessor {
    #[serde(skip_deserializing)]
    Name(String),
    Path(PathBuf),
}
