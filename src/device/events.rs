use std::{io, os::fd::AsRawFd, sync::Arc, thread::JoinHandle};

use evdev::{Device, MiscType};
use mio::{unix::SourceFd, Events, Interest, Poll, Token};
use tokio::{
    runtime::{self, Runtime},
    sync::Mutex,
};
use tokio_stream::{Stream, StreamExt, StreamMap};

use crate::{
    device::{DeviceEvent, InputEvent},
    events::{Event, EventPipelineSender},
    mio_channel, thread,
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

enum DeviceUpdate {
    Add(Device),
    Remove(Device),
}

pub struct DeviceEventWatcher {
    thread_handle: JoinHandle<()>,
    device_update_channel: mio_channel::Sender<DeviceUpdate>,
}
impl DeviceEventWatcher {
    pub fn new(event_pipeline: EventPipelineSender) -> io::Result<Self> {
        let (tx, rx) = mio_channel::channel();

        let thread_handle = thread::spawn_named("device event watcher", move || {
            Self::watch_input_events(rx, event_pipeline)
        });

        Ok(Self {
            thread_handle,
            device_update_channel: tx,
        })
    }

    pub fn join(self) -> std::thread::Result<()> {
        self.thread_handle.join()
    }

    pub fn watch(&self, devices: Vec<Device>) {
        for device in devices {
            let _ = self.device_update_channel.send(DeviceUpdate::Add(device));
        }
    }

    pub fn unwatch(&self, devices: Vec<Device>) {
        for device in devices {
            let _ = self
                .device_update_channel
                .send(DeviceUpdate::Remove(device));
        }
    }

    fn watch_input_events(
        mut device_update_channel: mio_channel::Receiver<DeviceUpdate>,
        event_pipeline: EventPipelineSender,
    ) {
        let poll = Poll::new().unwrap();

        let events = Events::with_capacity(128);

        const UPDATE_CHANNEL: Token = Token(0);

        poll.registry()
            .register(
                &mut device_update_channel,
                UPDATE_CHANNEL,
                Interest::WRITABLE,
            )
            .unwrap();

        let mut devices: Vec<Device> = vec![];

        for event in events.iter() {
            match event.token() {
                UPDATE_CHANNEL => match device_update_channel.try_recv().unwrap() {
                    DeviceUpdate::Add(device) => {
                        use nix::fcntl::{fcntl, FcntlArg, OFlag};

                        let raw_fd = device.as_raw_fd();
                        fcntl(raw_fd, FcntlArg::F_SETFL(OFlag::O_NONBLOCK)).unwrap();

                        let _ = poll.registry().register(
                            &mut SourceFd(&raw_fd),
                            Token(devices.len() + UPDATE_CHANNEL.0 + 1),
                            Interest::READABLE,
                        );

                        devices.push(device);
                    }
                    DeviceUpdate::Remove(device) => {
                        let Some(idx) = devices.iter().position(|dev| {
                            dev.physical_path()
                                .map(|a| device.physical_path().map(|b| a == b).unwrap_or(false))
                                .unwrap_or(false)
                        }) else {
                            continue;
                        };

                        let device = devices.swap_remove(idx);
                        let raw_fd = device.as_raw_fd();

                        let _ = poll.registry().deregister(&mut SourceFd(&raw_fd));
                    }
                },
                idx => {
                    let idx = idx.0 - UPDATE_CHANNEL.0;

                    let Some(device) = devices.get_mut(idx) else {
                        continue;
                    };

                    let id = device.physical_path().unwrap().to_string();

                    match device.fetch_events() {
                        Ok(iter) => {
                            for event in iter {
                                let event =
                                    DeviceEvent::new(id.clone(), InputEvent::from_raw(event));

                                use evdev::InputEventKind::*;

                                match event.kind() {
                                    Synchronization(_) | Misc(MiscType::MSC_SCAN) => continue,
                                    _ => {}
                                }

                                event_pipeline.send(Event::DeviceEvent(event)).unwrap();
                            }
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            continue;
                        }
                        Err(e) => {
                            todo!("Remove device from poll")
                        }
                    };
                }
            }
        }
    }
}
