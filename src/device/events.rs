use std::{
    borrow::Borrow,
    hash::Hash,
    io,
    pin::Pin,
    task::{Context, Poll},
};

use evdev::Device;
use tokio::runtime;
use tokio_stream::{Stream, StreamExt, StreamMap};

use crate::thread::{self, StoppableJoinHandle};

pub struct EventStream(evdev::EventStream);
impl EventStream {
    pub fn new(event_stream: evdev::EventStream) -> Self {
        Self(event_stream)
    }
}
impl Stream for EventStream {
    type Item = evdev::InputEvent;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.0.poll_event(cx).map(|res| res.ok())
    }
}

// pub struct StreamMap<K, V> {
//     entries: Vec<(K, V)>,
// }
// impl<K, V> StreamMap<K, V> {
//     pub fn new() -> Self {
//         Self { entries: vec![] }
//     }

//     pub fn remove<Q: ?Sized>(&mut self, k: &Q) -> Option<V>
//     where
//         K: Borrow<Q>,
//         Q: Hash + Eq,
//     {
//         for i in 0..self.entries.len() {
//             if self.entries[i].0.borrow() == k {
//                 return Some(self.entries.swap_remove(i).1);
//             }
//         }

//         None
//     }

//     pub fn insert(&mut self, k: K, stream: V) -> Option<V>
//     where
//         K: Hash + Eq,
//     {
//         let ret = self.remove(&k);
//         self.entries.push((k, stream));

//         ret
//     }
// }
// impl<K, V> StreamMap<K, V>
// where
//     K: Unpin,
//     V: Stream + Unpin,
// {
//     fn poll_next_entry(&mut self, cx: &mut Context<'_>) -> Poll<Option<(usize, V::Item)>> {
//         use Poll::*;

//         let start = rand::thread_rng().gen_range(0..self.entries.len() as u32) as usize;
//         let mut index = start;

//         for _ in 0..self.entries.len() {
//             let (_, stream) = &mut self.entries[index];

//             match Pin::new(stream).poll_next(cx) {
//                 Ready(Some(val)) => return Ready(Some((index, val))),
//                 Ready(None) => {
//                     self.entries.swap_remove(index);

//                     if index == self.entries.len() {
//                         index = 0;
//                     } else if index < start && start <= self.entries.len() {
//                         index = index.wrapping_add(1) % self.entries.len();
//                     }
//                 }
//                 Pending => {
//                     index = index.wrapping_add(1) % self.entries.len();
//                 }
//             }
//         }

//         if self.entries.is_empty() {
//             Ready(None)
//         } else {
//             Pending
//         }
//     }
// }
// impl<K, V> Stream for StreamMap<K, V>
// where
//     K: Clone + Unpin,
//     V: Stream + Unpin,
// {
//     type Item = (K, V::Item);

//     fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
//         if let Some((index, val)) = ready!(self.poll_next_entry(cx)) {
//             let key = self.entries[index].0.clone();
//             Poll::Ready(Some((key, val)))
//         } else {
//             Poll::Ready(None)
//         }
//     }
// }

pub fn watch_input_events(devices: Vec<Device>) -> Result<StoppableJoinHandle<()>, io::Error> {
    let rt = runtime::Builder::new_current_thread().enable_io().build()?;

    let event_watch_handle =
        thread::spawn_named_stoppable("device event watcher", move |stopped| {
            rt.block_on(async {
                // smol::block_on(async {
                let mut stream_map = StreamMap::new();

                for device in devices {
                    let path = device.physical_path().unwrap().to_string();
                    let stream = EventStream::new(device.into_event_stream().unwrap());

                    stream_map.insert(path, stream);
                }

                use std::sync::atomic::Ordering;

                while let Some(event) = stream_map.next().await {
                    // Temporary implementation as feature let_chains are unstable
                    // (issue #53667 https://github.com/rust-lang/rust/issues/53667)
                    if stopped.load(Ordering::Relaxed) {
                        break;
                    }

                    println!("Event: {:?}", event);
                }
            })
        });

    Ok(event_watch_handle)
}
