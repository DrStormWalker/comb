use std::{io, time::SystemTime};

use crate::events::{Event, EventPipelineSender};
#[cfg(feature = "tokio")]
use evdev::EventStream;
use evdev::{InputEvent, InputEventKind, MiscType};

#[cfg(not(feature = "tokio"))]
use mio::{unix::SourceFd, Events, Interest, Poll, Token};
#[cfg(feature = "tokio")]
use tokio::{
    sync::mpsc::{channel, Receiver, Sender},
    task::JoinHandle,
};

#[cfg(not(feature = "tokio"))]
use crate::mio_channel::{channel, Receiver, Sender};
#[cfg(not(feature = "tokio"))]
use crate::thread;
#[cfg(not(feature = "tokio"))]
use std::os::fd::AsRawFd;
#[cfg(not(feature = "tokio"))]
use std::thread::JoinHandle;

#[cfg(feature = "tokio")]
use tokio_stream::StreamMap;

use super::{DeviceId, DeviceIdCombo};

#[derive(Debug, Clone)]
pub struct DeviceEvent {
    device: DeviceId,
    timestamp: SystemTime,
    kind: InputEventKind,
    value: i32,
}
impl DeviceEvent {
    fn new(device: String, event: InputEvent) -> Self {
        Self {
            device,
            timestamp: event.timestamp(),
            kind: event.kind(),
            value: event.value(),
        }
    }

    pub fn device(&self) -> &str {
        &self.device
    }

    pub fn timestamp(&self) -> SystemTime {
        self.timestamp
    }

    pub fn value(&self) -> i32 {
        self.value
    }

    pub fn kind(&self) -> InputEventKind {
        self.kind
    }
}

#[derive(Debug)]
enum DeviceUpdate {
    Add(DeviceIdCombo),
    Remove(DeviceId),
}

pub struct DeviceEventWatch {
    thread_handle: JoinHandle<()>,
    device_update_channel: Sender<DeviceUpdate>,
}
impl DeviceEventWatch {
    pub fn new(event_pipeline: EventPipelineSender) -> io::Result<Self> {
        #[cfg(feature = "tokio")]
        let (tx, rx) = channel(1);

        #[cfg(not(feature = "tokio"))]
        let (tx, rx) = channel();

        #[cfg(not(feature = "tokio"))]
        let thread_handle = thread::spawn_named("device event watcher", move || {
            DeviceEventWatcher::new(event_pipeline, rx).watch();
        });

        #[cfg(feature = "tokio")]
        let thread_handle =
            tokio::spawn(async move { DeviceEventWatcher::new(event_pipeline, rx).watch().await });

        Ok(Self {
            thread_handle,
            device_update_channel: tx,
        })
    }

    pub fn handle(self) -> JoinHandle<()> {
        self.thread_handle
    }

    #[cfg(feature = "tokio")]
    pub async fn watch(&self, devices: Vec<DeviceIdCombo>) {
        for device in devices {
            let _ = self
                .device_update_channel
                .send(DeviceUpdate::Add(device))
                .await;
        }
    }

    #[cfg(not(feature = "tokio"))]
    pub fn watch(&self, devices: Vec<DeviceIdCombo>) {
        for device in devices {
            let _ = self.device_update_channel.send(DeviceUpdate::Add(device));
        }
    }

    #[allow(unused)]
    pub fn unwatch(&self, devices: Vec<DeviceId>) {
        for device in devices {
            let _ = self
                .device_update_channel
                .send(DeviceUpdate::Remove(device));
        }
    }
}

#[cfg(not(feature = "tokio"))]
const UPDATE_CHANNEL: Token = Token(0);

struct DeviceEventWatcher {
    #[cfg(feature = "tokio")]
    event_stream_map: StreamMap<DeviceId, EventStream>,
    #[cfg(not(feature = "tokio"))]
    poll: Poll,
    #[cfg(not(feature = "tokio"))]
    devices: Vec<DeviceIdCombo>,
    event_pipeline: EventPipelineSender,
    device_update_channel: Receiver<DeviceUpdate>,
}
impl DeviceEventWatcher {
    #[allow(unused_mut)]
    pub fn new(
        event_pipeline: EventPipelineSender,
        mut device_update_channel: Receiver<DeviceUpdate>,
    ) -> Self {
        #[cfg(not(feature = "tokio"))]
        {
            let poll = Poll::new().unwrap();

            poll.registry()
                .register(
                    &mut device_update_channel,
                    UPDATE_CHANNEL,
                    Interest::WRITABLE,
                )
                .unwrap();

            Self {
                poll,
                devices: vec![],
                event_pipeline,
                device_update_channel,
            }
        }

        #[cfg(feature = "tokio")]
        {
            let event_stream_map = StreamMap::new();

            Self {
                event_stream_map,
                event_pipeline,
                device_update_channel,
            }
        }
    }

    #[cfg(feature = "tokio")]
    pub async fn watch(mut self) {
        use tokio_stream::StreamExt;

        loop {
            tokio::select! {
                Some((id, event)) = self.event_stream_map.next() => self.handle_event(id, event),
                Some(update) = self.device_update_channel.recv() => self.update_devices(update),
            }
        }
    }

    #[cfg(not(feature = "tokio"))]
    pub fn watch(mut self) {
        let mut events = Events::with_capacity(64);

        loop {
            self.poll.poll(&mut events, None).unwrap();

            for event in events.iter() {
                match event.token() {
                    UPDATE_CHANNEL => {
                        let update = self.device_update_channel.try_recv().unwrap();

                        self.update_devices(update);
                    }
                    idx => {
                        self.handle_events(idx.0 - UPDATE_CHANNEL.0 - 1);
                    }
                }
            }
        }
    }

    fn update_devices(&mut self, update: DeviceUpdate) {
        match update {
            DeviceUpdate::Add(device) => self.add_device(device),
            DeviceUpdate::Remove(device) => self.remove_device(device),
        }
    }

    fn add_device(&mut self, device: DeviceIdCombo) {
        let name = device
            .name()
            .or(device.unique_name())
            .map(|name| name.to_owned());
        let id = device.id().to_owned();

        #[cfg(not(feature = "tokio"))]
        {
            use nix::fcntl::{fcntl, FcntlArg, OFlag};

            let raw_fd = device.as_raw_fd();
            fcntl(raw_fd, FcntlArg::F_SETFL(OFlag::O_NONBLOCK)).unwrap();

            if self
                .poll
                .registry()
                .register(
                    &mut SourceFd(&raw_fd),
                    Token(self.devices.len() + UPDATE_CHANNEL.0 + 1),
                    Interest::READABLE,
                )
                .is_ok()
            {
                self.devices.reserve_exact(1);
                self.devices.push(device);
            }
        }

        #[cfg(feature = "tokio")]
        self.event_stream_map.insert(
            device.id().to_string(),
            device.device.into_event_stream().unwrap(),
        );

        if let Some(name) = name {
            println!("Added {} ({})", name, id);
        } else {
            println!("Added {}", id);
        }
    }

    fn remove_device(&mut self, id: DeviceId) {
        #[cfg(not(feature = "tokio"))]
        let device = {
            let Some(idx) = self.devices.iter().position(|dev| {
                dev.id() == id
            }) else {
                return;
            };

            let device = self.devices.swap_remove(idx);

            let raw_fd = device.as_raw_fd();

            let _ = self.poll.registry().deregister(&mut SourceFd(&raw_fd));

            device
        };

        #[cfg(feature = "tokio")]
        let Some(device) = self.event_stream_map.remove(&id) else {
            return;
        };

        #[cfg(feature = "tokio")]
        let device = device.device();

        if let Some(name) = device.name().or(device.unique_name()) {
            println!("Removed {} ({})", name, id);
        } else {
            println!("Removed {}", id);
        }
    }

    fn send_event(event_pipeline: &EventPipelineSender, event: InputEvent, id: &str) {
        let event = DeviceEvent::new(id.to_string(), event);

        use evdev::InputEventKind::*;

        match event.kind() {
            Synchronization(_) | Misc(MiscType::MSC_SCAN) => return,
            _ => {}
        }

        let _ = event_pipeline.send(Event::DeviceEvent(event.clone()));

        if let Ok(input) = event.try_into() {
            let _ = event_pipeline.send(Event::DeviceInput(input));
        }
    }

    #[cfg(feature = "tokio")]
    fn handle_event(&mut self, id: DeviceId, event: Result<InputEvent, io::Error>) {
        if let Ok(event) = event {
            Self::send_event(&self.event_pipeline, event, &id);
        } else {
            self.remove_device(id);
        }
    }

    #[cfg(not(feature = "tokio"))]
    fn handle_events(&mut self, idx: usize) {
        let Some(device) = self.devices.get_mut(idx) else {
                return;
            };

        let id = device.id().to_string();

        {
            let events = device.fetch_events();

            let events = match events {
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => return,
                events => events,
            };

            if let Ok(events) = events {
                for event in events {
                    Self::send_event(&self.event_pipeline, event, &id);
                }
                return;
            }
        }

        self.remove_device(id);
    }
}
