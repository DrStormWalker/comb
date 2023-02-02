use std::{path::Path, time::Duration};

use notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_mini::{DebounceEventResult, Debouncer as RawDebouncer};
use tokio::{runtime::Handle, sync::mpsc, task::JoinHandle};

type Debouncer = RawDebouncer<RecommendedWatcher>;

fn new_debouncer(
    timeout: Duration,
    tick_rate: Option<Duration>,
    handle: Handle,
) -> Result<(Debouncer, mpsc::Receiver<DebounceEventResult>), notify::Error> {
    let (tx, rx) = mpsc::channel(1);

    let debouncer = notify_debouncer_mini::new_debouncer(timeout, tick_rate, move |events| {
        handle.block_on(async {
            let _ = tx.send(events).await;
        });
    })?;

    Ok((debouncer, rx))
}

pub fn watch(config_path: impl AsRef<Path>) -> Result<JoinHandle<()>, notify::Error> {
    let config_path = config_path.as_ref().to_path_buf();

    let handle = Handle::current();

    let (mut debouncer, mut rx) = new_debouncer(Duration::from_secs(1), None, handle)?;

    let config_watch_handle = tokio::spawn(async move {
        debouncer
            .watcher()
            .watch(&config_path, RecursiveMode::NonRecursive)
            .unwrap();

        loop {
            let events = match rx.recv().await {
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
                // TODO: Reload config file
                println!("Reload config file");
            }
        }
    });

    Ok(config_watch_handle)
}
