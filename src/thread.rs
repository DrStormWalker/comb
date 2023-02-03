use std::{
    ops::Deref,
    sync::{atomic::AtomicBool, Arc, Weak},
    thread::{Builder, JoinHandle},
};

pub fn spawn_named<F, T, S>(name: S, f: F) -> JoinHandle<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
    S: Into<String>,
{
    Builder::new().name(name.into()).spawn(f).unwrap()
}

pub struct StoppableJoinHandle<T> {
    join_handle: JoinHandle<T>,
    stopped: Weak<AtomicBool>,
}
impl<T> Deref for StoppableJoinHandle<T> {
    type Target = JoinHandle<T>;

    fn deref(&self) -> &Self::Target {
        &self.join_handle
    }
}
impl<T> Drop for StoppableJoinHandle<T> {
    fn drop(&mut self) {
        use std::sync::atomic::Ordering;

        if let Some(stopped) = self.stopped.upgrade() {
            stopped.store(true, Ordering::Relaxed);
        }
    }
}

pub fn spawn_named_stoppable<F, T, S>(name: S, f: F) -> StoppableJoinHandle<T>
where
    F: FnOnce(&AtomicBool) -> T + Send + 'static,
    T: Send + 'static,
    S: Into<String>,
{
    let stopped = Arc::new(AtomicBool::new(false));
    let weak_stopped = Arc::downgrade(&stopped);

    StoppableJoinHandle {
        join_handle: spawn_named(name, move || f(&*stopped)),
        stopped: weak_stopped,
    }
}
