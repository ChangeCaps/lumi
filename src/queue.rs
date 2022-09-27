use std::{ops::Deref, sync::Arc};

use crate::id::QueueId;

#[derive(Clone, Debug)]
pub struct SharedQueue {
    queue: Arc<wgpu::Queue>,
    id: QueueId,
}

impl SharedQueue {
    pub fn new(queue: wgpu::Queue) -> Self {
        Self {
            queue: Arc::new(queue),
            id: QueueId::new(),
        }
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn id(&self) -> QueueId {
        self.id
    }
}

impl Deref for SharedQueue {
    type Target = wgpu::Queue;

    fn deref(&self) -> &Self::Target {
        self.queue()
    }
}
