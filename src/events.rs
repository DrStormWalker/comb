use std::{path::PathBuf, sync::mpsc};

use crate::device::DeviceEvent;

pub type EventPipelineSender = mpsc::Sender<Event>;
pub type EventPipelineReceiver = mpsc::Receiver<Event>;

pub fn event_pipeline() -> (EventPipelineSender, EventPipelineReceiver) {
    mpsc::channel()
}

pub enum Event {
    ConfigWatchEvent(PathBuf),
    DeviceWatchEvent {
        added: Vec<PathBuf>,
        removed: Vec<PathBuf>,
    },
    DeviceEvent(DeviceEvent),
}
