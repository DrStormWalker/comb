use std::{io, path::PathBuf, sync::atomic::AtomicBool};

use evdev::{Device, MiscType};
use tokio::runtime;
use tokio_stream::{StreamExt, StreamMap};

use crate::{
    device::{DeviceEvent, InputEvent},
    events::{Event, EventPipelineSender},
    thread::{self, StoppableJoinHandle},
};

pub fn watch(
    event_pipeline: EventPipelineSender,
    devices: Vec<Device>,
) -> Result<StoppableJoinHandle<()>, io::Error> {
    let rt = runtime::Builder::new_current_thread().enable_io().build()?;

    let event_watch_handle =
        thread::spawn_named_stoppable("device event watcher", move |stopped| {
            rt.block_on(async move { watch_input_events(event_pipeline, devices, stopped).await });
        });

    Ok(event_watch_handle)
}

async fn watch_input_events(
    event_pipeline: EventPipelineSender,
    devices: Vec<Device>,
    stopped: &AtomicBool,
) {
    let mut stream_map = StreamMap::new();

    for device in devices {
        let path = device.physical_path().unwrap().to_string();
        let stream = device
            .into_event_stream()
            .unwrap()
            .map_while(|event| event.ok())
            .map(|event| InputEvent::from_raw(event));

        stream_map.insert(path, stream);
    }

    use std::sync::atomic::Ordering;

    let mut stream_map = stream_map.map(|(path, event)| DeviceEvent::new(path, event));

    while let Some(event) = stream_map.next().await {
        // Temporary implementation as feature let_chains are unstable
        // (issue #53667 https://github.com/rust-lang/rust/issues/53667)
        if stopped.load(Ordering::Relaxed) {
            break;
        }

        use evdev::InputEventKind::*;

        match event.kind() {
            Synchronization(_) | Misc(MiscType::MSC_SCAN) => continue,
            _ => {}
        }

        if event_pipeline.send(Event::DeviceEvent(event)).is_err() {
            break;
        }
    }
}
