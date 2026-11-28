//! Passing a console's output from the thread that reads it to whoever shows
//! it, and telling them that it is there.

use std::io::Read;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::SyncSender;
use std::sync::Arc;

use imperifex_domain::ports::Wake;

const CHUNK: usize = 16 * 1024;

/// A [`Wake`] that is not called again until the last call was answered, so
/// that a burst of output does not flood whoever listens.
#[derive(Clone)]
pub struct Doorbell {
    rung: Arc<AtomicBool>,
    wake: Wake,
}

impl Doorbell {
    pub fn new(wake: Wake) -> Self {
        Self {
            rung: Arc::new(AtomicBool::new(false)),
            wake,
        }
    }

    /// After leaving something to be read.
    fn ring(&self) {
        if !self.rung.swap(true, Ordering::SeqCst) {
            (self.wake)();
        }
    }

    /// Before looking for what there is to read: whatever arrives after this
    /// rings again.
    pub fn answer(&self) {
        self.rung.store(false, Ordering::SeqCst);
    }
}

/// Passes the output on until the console closes its pipe. With nobody left to
/// take it, it is still read and dropped: closing may wait for that. The end
/// is always told, once the channel is closed.
pub fn forward(mut from: impl Read, to: SyncSender<Vec<u8>>, doorbell: &Doorbell) {
    let mut chunk = vec![0u8; CHUNK];
    let mut listener = Some(to);
    // An error here is the pipe closing: the end of the output.
    while let Ok(read @ 1..) = from.read(&mut chunk) {
        if let Some(to) = &listener {
            if to.send(chunk[..read].to_vec()).is_ok() {
                doorbell.ring();
            } else {
                listener = None;
            }
        }
    }
    drop(listener);
    (doorbell.wake)();
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicUsize;
    use std::sync::mpsc::{sync_channel, TryRecvError};

    use super::*;

    fn counting() -> (Doorbell, Arc<AtomicUsize>) {
        let calls = Arc::new(AtomicUsize::new(0));
        let counted = Arc::clone(&calls);
        let doorbell = Doorbell::new(Arc::new(move || {
            counted.fetch_add(1, Ordering::SeqCst);
        }));
        (doorbell, calls)
    }

    #[test]
    fn a_burst_rings_once_until_it_is_answered() {
        let (doorbell, calls) = counting();
        doorbell.ring();
        doorbell.ring();
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        doorbell.answer();
        doorbell.ring();
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn all_the_output_is_passed_on_with_one_call_for_it_and_one_for_the_end() {
        let (doorbell, calls) = counting();
        let (send, receive) = sync_channel(8);
        let output = vec![7u8; 3 * CHUNK + 5];
        forward(output.as_slice(), send, &doorbell);
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        let mut found = Vec::new();
        let end = loop {
            match receive.try_recv() {
                Ok(chunk) => found.extend(chunk),
                Err(end) => break end,
            }
        };
        assert_eq!(end, TryRecvError::Disconnected);
        assert_eq!(found, output);
    }

    #[test]
    fn with_nobody_listening_it_is_still_read_to_the_end() {
        let (doorbell, calls) = counting();
        let (send, receive) = sync_channel(1);
        drop(receive);
        let output = vec![7u8; 3 * CHUNK];
        let mut left = output.as_slice();
        forward(&mut left, send, &doorbell);
        assert!(left.is_empty());
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}
