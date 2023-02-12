//! Reimplementation of the [`mio_extras`](https://docs.rs/mio-extras/latest)
//! channels

use std::{
    io,
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc, Arc, Mutex,
    },
};

use mio::{event::Source, Waker};
use thiserror::Error;

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let (tx, rx) = mpsc::channel();

    let inner = Arc::new(Inner {
        pending: AtomicUsize::new(0),
        senders: AtomicUsize::new(0),
        waker: Arc::new(Mutex::new(None)),
    });

    let tx = Sender {
        tx,
        inner: inner.clone(),
    };

    let rx = Receiver {
        rx,
        inner: inner.clone(),
    };

    (tx, rx)
}

struct Inner {
    pending: AtomicUsize,
    senders: AtomicUsize,
    waker: Arc<Mutex<Option<Waker>>>,
}
impl Inner {
    fn increment(&self) -> io::Result<()> {
        let count = self.pending.fetch_add(1, Ordering::Acquire);

        if count == 0 {
            if let Some(waker) = self.waker.lock().unwrap().as_mut() {
                waker.wake()?;
            }
        }

        Ok(())
    }

    fn decrement(&self) -> io::Result<()> {
        let count = self.pending.fetch_sub(1, Ordering::Acquire);

        if count > 1 {
            if let Some(waker) = self.waker.lock().unwrap().as_mut() {
                waker.wake()?;
            }
        }

        Ok(())
    }
}

pub struct Sender<T> {
    tx: mpsc::Sender<T>,
    inner: Arc<Inner>,
}
impl<T> Sender<T> {
    pub fn send(&self, t: T) -> Result<(), SendError<T>> {
        self.tx.send(t).map_err(SendError::from).and_then(|_| {
            self.inner.increment()?;
            Ok(())
        })
    }
}
impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        self.inner.senders.fetch_add(1, Ordering::Relaxed);

        Self {
            tx: self.tx.clone(),
            inner: self.inner.clone(),
        }
    }
}

pub struct Receiver<T> {
    rx: mpsc::Receiver<T>,
    inner: Arc<Inner>,
}
impl<T> Receiver<T> {
    pub fn try_recv(&self) -> Result<T, mpsc::TryRecvError> {
        self.rx.try_recv().and_then(|res| {
            let _ = self.inner.decrement();
            Ok(res)
        })
    }
}
impl<T> Source for Receiver<T> {
    fn register(
        &mut self,
        registry: &mio::Registry,
        token: mio::Token,
        _: mio::Interest,
    ) -> io::Result<()> {
        if self.inner.waker.lock().unwrap().is_some() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "receiver already registered",
            ));
        }

        let waker = Waker::new(registry, token)?;

        if self.inner.pending.load(Ordering::Relaxed) > 0 {
            let _ = waker.wake();
        }

        *self.inner.waker.lock().unwrap() = Some(waker);

        Ok(())
    }

    fn reregister(
        &mut self,
        registry: &mio::Registry,
        token: mio::Token,
        _: mio::Interest,
    ) -> io::Result<()> {
        let mut inner_waker = self.inner.waker.lock().unwrap();
        if inner_waker.is_some() {
            let waker = Waker::new(registry, token)?;

            if self.inner.pending.load(Ordering::Relaxed) > 0 {
                let _ = waker.wake();
            }

            *inner_waker = Some(waker);

            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "receiver not registered",
            ))
        }
    }

    fn deregister(&mut self, _: &mio::Registry) -> io::Result<()> {
        let mut waker = self.inner.waker.lock().unwrap();
        if waker.is_some() {
            *waker = None;

            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "receiver not registered",
            ))
        }
    }
}

#[derive(Debug, Error)]
pub enum SendError<T> {
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error("Disconnected")]
    Disconnected(T),
}
impl<T> From<mpsc::SendError<T>> for SendError<T> {
    fn from(value: mpsc::SendError<T>) -> Self {
        Self::Disconnected(value.0)
    }
}
