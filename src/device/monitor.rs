use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use notify::{EventKind, RecommendedWatcher, Watcher};
use tokio::{runtime::Handle, sync::mpsc, task::JoinHandle};

use crate::thread;

type EventResult = Result<notify::Event, notify::Error>;

fn new_watcher(
    handle: Handle,
) -> Result<(RecommendedWatcher, mpsc::Receiver<EventResult>), notify::Error> {
    let (tx, rx) = mpsc::channel(1);

    let watcher = RecommendedWatcher::new(
        move |event| {
            handle.block_on(async {
                let _ = tx.send(event).await;
            })
        },
        notify::Config::default(),
    )?;

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

pub fn watch() -> Result<JoinHandle<()>, notify::Error> {
    let dev_path = Path::new("/dev/input");

    let handle = Handle::current();

    let (mut watcher, mut rx) = new_watcher(handle)?;

    let device_watch_handle = tokio::spawn(async move {
        watcher
            .watch(dev_path, notify::RecursiveMode::NonRecursive)
            .unwrap();

        let mut devices = HashSet::new();

        let _ = populate_devices(&mut devices);

        loop {
            match rx.recv().await {
                Some(Ok(notify::Event {
                    kind: EventKind::Create(_) | EventKind::Remove(_),
                    paths: _,
                    attrs: _,
                })) => {}
                Some(Ok(_)) => continue,
                Some(Err(err)) => {
                    println!("Input device watcher error: {:?}", err);
                    continue;
                }
                None => {
                    println!("Input device watcher channel dropped");
                    break;
                }
            };

            let (added, removed) = populate_devices(&mut devices);

            println!("Added devices: {:?}. Removed devices: {:?}", added, removed);
        }
    });

    Ok(device_watch_handle)
}
