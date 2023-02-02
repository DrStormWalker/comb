#![feature(file_create_new)]

mod config;

use std::{
    collections::HashSet,
    fs::File,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use config::{load_config_from_path, Config};
use notify::{EventKind, RecommendedWatcher, Watcher};
use notify_debouncer_mini::{DebounceEventResult, Debouncer as RawDebouncer};
use tokio::{
    join,
    runtime::{self, Handle},
    sync::{mpsc, RwLock},
    task::JoinHandle,
};

type Debouncer = RawDebouncer<RecommendedWatcher>;

fn new_debouncer(
    timeout: Duration,
    tick_rate: Option<Duration>,
    handle: Handle,
) -> Result<(Debouncer, mpsc::Receiver<DebounceEventResult>), notify::Error> {
    let (tx, rx) = mpsc::channel(1);

    let debouncer = notify_debouncer_mini::new_debouncer(timeout, tick_rate, move |res| {
        handle.block_on(async {
            tx.send(res).await.unwrap();
        });
    })?;

    Ok((debouncer, rx))
}

type EventResult = Result<notify::Event, notify::Error>;

fn new_watcher(
    handle: Handle,
) -> Result<(RecommendedWatcher, mpsc::Receiver<EventResult>), notify::Error> {
    let (tx, rx) = mpsc::channel(1);

    let watcher = RecommendedWatcher::new(
        move |res| {
            handle.block_on(async {
                tx.send(res).await.unwrap();
            });
        },
        notify::Config::default(),
    )?;

    Ok((watcher, rx))
}

async fn populate_devices(devices: &mut HashSet<PathBuf>) -> (Vec<PathBuf>, Vec<PathBuf>) {
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

async fn watch_config_file(
    config: Arc<RwLock<Config>>,
    config_path: impl AsRef<Path>,
) -> Result<JoinHandle<()>, notify::Error> {
    let handle = Handle::current();
    let (mut debouncer, mut rx) = new_debouncer(Duration::from_secs(1), None, handle)?;

    let config_path = config_path.as_ref().to_path_buf();

    debouncer
        .watcher()
        .watch(&config_path, notify::RecursiveMode::NonRecursive)?;

    let config_watch_handle = tokio::spawn(async move {
        let _debouncer = debouncer;

        while let Some(events) = rx.recv().await {
            handle_config_events(&config, events, &config_path).await;
        }
    });

    Ok(config_watch_handle)
}

async fn handle_config_events(
    config: &Arc<RwLock<Config>>,
    events: DebounceEventResult,
    config_path: impl AsRef<Path>,
) {
    match events {
        Ok(events) => {
            if events.iter().any(|e| e.path == config_path.as_ref()) {
                let new_config = load_config_from_path(&config_path).await.unwrap();

                let mut config = config.write().await;

                *config = new_config;

                println!("new Config: {:#?}", *config);
            }
        }
        Err(e) => println!("{:?}", e),
    }
}

async fn watch_input_devices() -> Result<JoinHandle<()>, notify::Error> {
    let handle = Handle::current();
    let (mut watcher, mut rx) = new_watcher(handle)?;

    let dev_path = Path::new("/dev/input");

    watcher.watch(dev_path, notify::RecursiveMode::NonRecursive)?;

    let input_watch_handle = tokio::spawn(async move {
        let _watcher = watcher;

        let mut devices = HashSet::new();

        while let Some(event) = rx.recv().await {
            handle_input_device_event(event, &mut devices).await;
        }
    });

    Ok(input_watch_handle)
}

async fn handle_input_device_event(event: EventResult, devices: &mut HashSet<PathBuf>) {
    match event {
        Ok(notify::Event {
            kind: EventKind::Create(_) | EventKind::Remove(_),
            paths: _,
            attrs: _,
        }) => {
            let (added, removed) = populate_devices(devices).await;
            println!("Added devices: {:?}. Removed devices: {:?}", added, removed);
        }
        Ok(_) => {}
        Err(e) => println!("{:?}", e),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rt = runtime::Runtime::new()?;

    rt.block_on(async {
        let xdg_dirs = xdg::BaseDirectories::with_prefix("comb")?;
        let config_path = xdg_dirs.place_config_file("config.toml")?;

        let _ = File::create_new(&config_path);

        let config = Arc::new(RwLock::new(load_config_from_path(&config_path).await?));

        let config_watch_handle = watch_config_file(config, config_path).await?;
        let input_watch_handle = watch_input_devices().await?;

        let _ = join!(config_watch_handle, input_watch_handle,);

        Result::<(), Box<dyn std::error::Error>>::Ok(())
    })?;

    Ok(())
}
