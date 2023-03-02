use std::path::PathBuf;

use crate::device::{DeviceEvent, DeviceInput};

pub use self::pipeline::*;

#[cfg(not(feature = "tokio"))]
mod pipeline {
    use super::Event;
    use std::sync::mpsc;

    pub type EventPipelineSender = mpsc::Sender<Event>;
    pub type EventPipelineReceiver = mpsc::Receiver<Event>;

    pub fn event_pipeline() -> (EventPipelineSender, EventPipelineReceiver) {
        mpsc::channel()
    }
}

#[cfg(feature = "tokio")]
mod pipeline {
    use super::Event;
    use tokio::sync::mpsc;

    pub type EventPipelineSender = mpsc::UnboundedSender<Event>;
    pub type EventPipelineReceiver = mpsc::UnboundedReceiver<Event>;

    pub fn event_pipeline() -> (EventPipelineSender, EventPipelineReceiver) {
        mpsc::unbounded_channel()
    }
}

pub enum Event {
    ConfigWatchEvent(PathBuf),
    DeviceWatchEvent {
        added: Vec<PathBuf>,
        removed: Vec<PathBuf>,
    },
    DeviceEvent(DeviceEvent),
    DeviceInput(DeviceInput),
}
