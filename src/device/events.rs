use std::{io, sync::Arc, thread::JoinHandle};

use evdev::{Device, MiscType};
use tokio::{
    runtime::{self, Runtime},
    sync::Mutex,
};
use tokio_stream::{Stream, StreamExt, StreamMap};

use crate::{
    device::{DeviceEvent, InputEvent},
    events::{Event, EventPipelineSender},
    thread,
};

struct EventStream {
    raw_event_stream: evdev::EventStream,
}
impl EventStream {
    pub fn from_raw(raw_event_stream: evdev::EventStream) -> Self {
        Self { raw_event_stream }
    }
}
impl Stream for EventStream {
    type Item = InputEvent;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.raw_event_stream
            .poll_event(cx)
            .map(|event| event.ok().map(|event| InputEvent::from_raw(event)))
    }
}

type DeviceEventStreamMap = StreamMap<String, EventStream>;

pub struct DeviceEventWatcher {
    device_event_streams: Arc<Mutex<DeviceEventStreamMap>>,
    thread_handle: JoinHandle<()>,
    runtime: Arc<Runtime>,
}
impl DeviceEventWatcher {
    pub fn new(event_pipeline: EventPipelineSender) -> io::Result<Self> {
        let runtime = Arc::new(runtime::Builder::new_current_thread().enable_io().build()?);

        let device_event_streams = Arc::new(Mutex::new(StreamMap::new()));

        let thread_handle = {
            let device_event_streams = device_event_streams.clone();
            let runtime = runtime.clone();

            thread::spawn_named("device event watcher", move || {
                runtime.block_on(async move {
                    Self::watch_input_events(event_pipeline, device_event_streams).await
                });
            })
        };

        Ok(Self {
            device_event_streams,
            thread_handle,
            runtime,
        })
    }

    pub fn join(self) -> std::thread::Result<()> {
        self.thread_handle.join()
    }

    pub fn watch(&self, devices: Vec<Device>) {
        self.runtime.block_on(async {
            let mut device_event_streams = self.device_event_streams.lock().await;

            for device in devices {
                let path = device.physical_path().unwrap().to_string();
                // `device.into_event_stream` must be called from a tokio runtime
                let event_stream = EventStream::from_raw(device.into_event_stream().unwrap());

                device_event_streams.insert(path, event_stream);
            }
        })
    }

    pub fn unwatch(&self, devices: Vec<Device>) {
        let mut device_event_streams = self.device_event_streams.blocking_lock();

        for device in devices {
            let path = device.physical_path().unwrap();

            device_event_streams.remove(path);
        }
    }

    async fn watch_input_events(
        event_pipeline: EventPipelineSender,
        device_event_streams: Arc<Mutex<DeviceEventStreamMap>>,
    ) {
        loop {
            let Some((id, event)) = device_event_streams.lock().await.next().await else {
                continue;
            };

            let event = DeviceEvent::new(id, event);

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
}
