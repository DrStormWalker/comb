use std::{path::Path, time::Duration};

use notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_mini::{DebounceEventResult, Debouncer as RawDebouncer};

use crate::events::{Event, EventPipelineSender};

#[cfg(feature = "tokio")]
use tokio::{sync::mpsc, task::JoinHandle};

#[cfg(not(feature = "tokio"))]
use std::{sync::mpsc, thread::JoinHandle};

#[cfg(not(feature = "tokio"))]
use crate::thread;

type Debouncer = RawDebouncer<RecommendedWatcher>;

fn new_debouncer(
    timeout: Duration,
    tick_rate: Option<Duration>,
) -> Result<(Debouncer, mpsc::Receiver<DebounceEventResult>), notify::Error> {
    #[cfg(feature = "tokio")]
    let (tx, rx) = mpsc::channel(1);

    #[cfg(not(feature = "tokio"))]
    let (tx, rx) = mpsc::channel();

    let debouncer = notify_debouncer_mini::new_debouncer(timeout, tick_rate, move |res| {
        #[cfg(feature = "tokio")]
        let _ = tx.blocking_send(res);

        #[cfg(not(feature = "tokio"))]
        let _ = tx.send(res);
    })?;

    Ok((debouncer, rx))
}

#[cfg(feature = "tokio")]
pub async fn watch(
    event_pipeline: EventPipelineSender,
    config_path: impl AsRef<Path>,
) -> Result<JoinHandle<()>, notify::Error> {
    let config_path = config_path.as_ref().to_path_buf();

    let (mut debouncer, mut rx) = new_debouncer(Duration::from_secs(1), None)?;

    let config_watch_handle = tokio::spawn(async move {
        debouncer
            .watcher()
            .watch(&config_path, RecursiveMode::NonRecursive)
            .unwrap();

        loop {
            #[cfg(feature = "tokio")]
            let events = rx.recv().await;

            #[cfg(not(feature = "tokio"))]
            let events = rx.recv();

            let events = match events {
                Some(Ok(events)) => events,
                Some(Err(err)) => {
                    println!("Config watcher error: {:?}", err);
                    continue;
                }
                None => {
                    println!("Config watcher channel dropped");
                    break;
                }
            };

            if events.iter().any(|e| e.path == config_path) {
                if event_pipeline
                    .send(Event::ConfigWatchEvent(config_path.clone()))
                    .is_err()
                {
                    break;
                }
            }
        }
    });

    Ok(config_watch_handle)
}

#[cfg(not(feature = "tokio"))]
pub fn watch(
    event_pipeline: EventPipelineSender,
    config_path: impl AsRef<Path>,
) -> Result<JoinHandle<()>, notify::Error> {
    let config_path = config_path.as_ref().to_path_buf();

    let (mut debouncer, rx) = new_debouncer(Duration::from_secs(1), None)?;

    let config_watch_handle = thread::spawn_named("config watcher", move || {
        debouncer
            .watcher()
            .watch(&config_path, RecursiveMode::NonRecursive)
            .unwrap();

        loop {
            let events = match rx.recv() {
                Ok(Ok(events)) => events,
                Ok(Err(err)) => {
                    println!("Config watcher error: {:?}", err);
                    continue;
                }
                Err(err) => {
                    println!("Config watcher channel dropped: {:?}", err);
                    break;
                }
            };

            if events.iter().any(|e| e.path == config_path) {
                if event_pipeline
                    .send(Event::ConfigWatchEvent(config_path.clone()))
                    .is_err()
                {
                    break;
                }
            }
        }
    });

    Ok(config_watch_handle)
}
