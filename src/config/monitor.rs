use std::{path::Path, sync::mpsc, thread::JoinHandle, time::Duration};

use notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_mini::{DebounceEventResult, Debouncer as RawDebouncer};

use crate::{
    events::{Event, EventPipelineSender},
    thread,
};

type Debouncer = RawDebouncer<RecommendedWatcher>;

fn new_debouncer(
    timeout: Duration,
    tick_rate: Option<Duration>,
) -> Result<(Debouncer, mpsc::Receiver<DebounceEventResult>), notify::Error> {
    let (tx, rx) = mpsc::channel();

    let debouncer = notify_debouncer_mini::new_debouncer(timeout, tick_rate, tx)?;

    Ok((debouncer, rx))
}

pub fn watch(
    event_pipeline: EventPipelineSender,
    config_path: impl AsRef<Path>,
) -> Result<JoinHandle<()>, notify::Error> {
    let config_path = config_path.as_ref().to_path_buf();

    let (mut debouncer, mut rx) = new_debouncer(Duration::from_secs(1), None)?;

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
