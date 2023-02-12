use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    sync::mpsc,
    thread::JoinHandle,
};

use notify::{EventKind, RecommendedWatcher, Watcher};

use crate::{
    events::{Event, EventPipelineSender},
    thread,
};

type EventResult = Result<notify::Event, notify::Error>;

fn new_watcher() -> Result<(RecommendedWatcher, mpsc::Receiver<EventResult>), notify::Error> {
    let (tx, rx) = mpsc::channel();

    let watcher = RecommendedWatcher::new(tx, notify::Config::default())?;

    Ok((watcher, rx))
}

fn populate_devices(devices: &mut HashSet<PathBuf>) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let available_devices: HashSet<PathBuf> = evdev::enumerate()
        .into_iter()
        .map(|(path, _)| path)
        .collect();

    let new_devices = available_devices
        .iter()
        .filter(|&dev| !devices.contains(dev))
        .map(|dev| dev.clone())
        .collect();

    let removed_devices = devices
        .iter()
        .filter(|&dev| !available_devices.contains(dev))
        .map(|dev| dev.clone())
        .collect();

    *devices = available_devices;

    (new_devices, removed_devices)
}

pub fn watch(event_pipeline: EventPipelineSender) -> Result<JoinHandle<()>, notify::Error> {
    let dev_path = Path::new("/dev/input");

    let (mut watcher, rx) = new_watcher()?;

    let device_watch_handle = thread::spawn_named("device watcher", move || {
        watcher
            .watch(dev_path, notify::RecursiveMode::NonRecursive)
            .unwrap();

        let mut devices = HashSet::new();

        let _ = populate_devices(&mut devices);

        loop {
            match rx.recv() {
                Ok(Ok(notify::Event {
                    kind: EventKind::Create(_) | EventKind::Remove(_),
                    paths: _,
                    attrs: _,
                })) => {}
                Ok(Ok(_)) => continue,
                Ok(Err(err)) => {
                    println!("Input device watcher error: {:?}", err);
                    continue;
                }
                Err(err) => {
                    println!("Input device watcher channel dropped: {:?}", err);
                    break;
                }
            };

            let (added, removed) = populate_devices(&mut devices);

            if event_pipeline
                .send(Event::DeviceWatchEvent { added, removed })
                .is_err()
            {
                break;
            }
        }
    });

    Ok(device_watch_handle)
}
