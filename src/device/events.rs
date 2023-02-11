use std::{io, os::fd::AsRawFd, thread::JoinHandle, time::Duration};

use evdev::{Device, MiscType};
use mio::{unix::SourceFd, Events, Interest, Poll, Token};

use crate::{
    device::{DeviceEvent, InputEvent},
    events::{Event, EventPipelineSender},
    mio_channel, thread,
};

enum DeviceUpdate {
    Add(Device),
    Remove(Device),
}

pub struct DeviceEventWatch {
    thread_handle: JoinHandle<()>,
    device_update_channel: mio_channel::Sender<DeviceUpdate>,
}
impl DeviceEventWatch {
    pub fn new(event_pipeline: EventPipelineSender) -> io::Result<Self> {
        let (tx, rx) = mio_channel::channel();

        let thread_handle = thread::spawn_named("device event watcher", move || {
            DeviceEventWatcher::new(event_pipeline, rx).watch();
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
            let e = self.device_update_channel.send(DeviceUpdate::Add(device));
        }
    }

    pub fn unwatch(&self, devices: Vec<Device>) {
        for device in devices {
            let _ = self
                .device_update_channel
                .send(DeviceUpdate::Remove(device));
        }
    }
}

const UPDATE_CHANNEL: Token = Token(0);

struct DeviceEventWatcher {
    poll: Poll,
    devices: Vec<Device>,
    event_pipeline: EventPipelineSender,
    device_update_channel: mio_channel::Receiver<DeviceUpdate>,
}
impl DeviceEventWatcher {
    pub fn new(
        event_pipeline: EventPipelineSender,
        mut device_update_channel: mio_channel::Receiver<DeviceUpdate>,
    ) -> Self {
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

    pub fn watch(mut self) {
        let mut events = Events::with_capacity(64);

        loop {
            self.poll.poll(&mut events, None).unwrap();

            for event in events.iter() {
                match event.token() {
                    UPDATE_CHANNEL => self.update_devices(),
                    idx => self.handle_event(idx.0 - UPDATE_CHANNEL.0 - 1),
                }
            }
        }
    }

    fn update_devices(&mut self) {
        match self.device_update_channel.try_recv().unwrap() {
            DeviceUpdate::Add(device) => self.add_device(device),
            DeviceUpdate::Remove(device) => {
                let Some(idx) = self.devices.iter().position(|dev| {
                    dev.physical_path()
                        .map(|a| device.physical_path().map(|b| a == b).unwrap_or(false))
                        .unwrap_or(false)
                }) else {
                    return;
                };

                self.remove_device(idx);
            }
        }
    }

    fn add_device(&mut self, device: Device) {
        use nix::fcntl::{fcntl, FcntlArg, OFlag};

        let raw_fd = device.as_raw_fd();
        fcntl(raw_fd, FcntlArg::F_SETFL(OFlag::O_NONBLOCK)).unwrap();

        let _ = self.poll.registry().register(
            &mut SourceFd(&raw_fd),
            Token(self.devices.len() + UPDATE_CHANNEL.0 + 1),
            Interest::READABLE,
        );

        self.devices.reserve_exact(1);
        self.devices.push(device);
    }

    fn remove_device(&mut self, idx: usize) {
        let device = self.devices.swap_remove(idx);
        let raw_fd = device.as_raw_fd();

        let _ = self.poll.registry().deregister(&mut SourceFd(&raw_fd));
    }

    fn send_events(
        event_pipeline: &EventPipelineSender,
        events: evdev::FetchEventsSynced,
        id: String,
    ) {
        for event in events {
            let event = DeviceEvent::new(id.clone(), InputEvent::from_raw(event));

            use evdev::InputEventKind::*;

            match event.kind() {
                Synchronization(_) | Misc(MiscType::MSC_SCAN) => return,
                _ => {}
            }

            event_pipeline.send(Event::DeviceEvent(event)).unwrap();
        }
    }

    fn handle_event(&mut self, idx: usize) {
        let Some(device) = self.devices.get_mut(idx) else {
            return;
        };

        let id = device.physical_path().unwrap().to_string();

        {
            let events = device.fetch_events();
            // println!("{:?}", events.is_err());

            let events = match events {
                Err(e) => return println!("{:?}", e),
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => return,
                events => events,
            };

            if let Ok(events) = events {
                Self::send_events(&self.event_pipeline, events, id);
                return;
            }
        }

        self.remove_device(idx);
    }
}
