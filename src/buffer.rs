use std::sync::atomic::{AtomicUsize, Ordering};

use crate::{ctmp::Ctmp, Error};

const BUFFER_SIZE: usize = 0x400;

pub struct MessageMgr {
    messages: [Ctmp; BUFFER_SIZE],
    read_pos: AtomicUsize,
    write_pos: AtomicUsize,
}

unsafe impl Sync for MessageMgr {}

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

    /// needs to return a vector of owned msgs because we need to copy out messages 
    /// before it gets overwritten by producer
    /// 
    /// this is batched since we require clones and allocs, the other is cheap
    pub fn read_batch(&self, max_batch: usize) -> Vec<Ctmp> {
        let mut read_idx = self.read_pos.load(Ordering::Acquire);
        let write_idx = self.write_pos.load(Ordering::Acquire);

        let msg_diff = if write_idx >= read_idx {
            write_idx - read_idx
        } else {
            BUFFER_SIZE - read_idx + write_idx
        };

        let batch_size = msg_diff.min(max_batch);
        let mut res = Vec::with_capacity(batch_size);

        while read_idx != write_idx {
            let slot = unsafe { &*self.messages.as_ptr().add(read_idx) };
            res.push(slot.clone());

            read_idx = (read_idx + 1) % BUFFER_SIZE;
        }

        self.read_pos.store(write_idx, Ordering::Release);
        res
    }
}
