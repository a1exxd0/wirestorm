use std::sync::atomic::{AtomicUsize, Ordering};

use crate::{Ctmp, Error};

const BUFFER_SIZE: usize = 0x400;

/// Each message is 64KB. 1000 messages is 64MB, trivial for
/// even most embedded systems.
///
/// Meant for a single producer, single consumer mechanism
pub struct MessageMgr {
    messages: [Ctmp; BUFFER_SIZE],
    read_pos: AtomicUsize,
    write_pos: AtomicUsize,
}

#[allow(dead_code)]
impl MessageMgr {
    pub fn new() -> Self {
        MessageMgr {
            messages: ::core::array::from_fn(|_| Ctmp::default()),
            read_pos: AtomicUsize::new(0),
            write_pos: AtomicUsize::new(0),
        }
    }

    pub fn write(&self, data: Ctmp) -> Result<(), Error> {
        let write_idx = self.write_pos.load(Ordering::Relaxed);
        let read_idx = self.read_pos.load(Ordering::Acquire);

        let next_write = (write_idx + 1) % BUFFER_SIZE;
        if next_write == read_idx {
            return Err(Error::BufferWriteOverflow);
        }

        let slot = unsafe { &mut *self.messages.as_ptr().add(write_idx).cast_mut() };
        *slot = data;

        self.write_pos.store(next_write, Ordering::Release);
        Ok(())
    }
}
