use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

pub struct MessageMgr {
    messages: Arc<Mutex<VecDeque<bytes::Bytes>>>,
}

#[allow(dead_code)]
impl MessageMgr {
    pub fn new() -> Self {
        MessageMgr {
            messages: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn write(&self, data: bytes::Bytes) {
        let mut queue = match self.messages.lock() {
            Ok(queue) => queue,
            Err(poisoned) => {
                tracing::warn!("found poisoned message mgr lock, returning inner");
                poisoned.into_inner()
            }
        };
        queue.push_back(data);
        let buffer_size = queue.len();
        tracing::trace!(buffer_size, "written to message buffer");
    }

    pub fn read_batch(&self, max_batch: usize) -> Vec<bytes::Bytes> {
        let mut queue = match self.messages.lock() {
            Ok(queue) => queue,
            Err(poisoned) => {
                tracing::warn!("found poisoned message mgr lock, returning inner");
                poisoned.into_inner()
            }
        };

        let will_read = std::cmp::min(max_batch, queue.len());
        let mut batch = Vec::with_capacity(will_read);

        // Actually remove messages from the queue
        for _ in 0..will_read {
            if let Some(msg) = queue.pop_front() {
                batch.push(msg);
            }
        }

        tracing::trace!(
            read_count = batch.len(),
            remaining = queue.len(),
            "read batch"
        );
        batch
    }
}
