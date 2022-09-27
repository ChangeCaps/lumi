mod storage;
mod uniform;

use wgpu::BufferUsages;

use crate::id::BufferId;

use std::{ops::Deref, sync::Arc};

pub use storage::*;
pub use uniform::*;

#[derive(Clone, Debug)]
pub struct SharedBuffer {
    buffer: Arc<wgpu::Buffer>,
    id: BufferId,

    size: u64,
    usage: BufferUsages,
}

impl SharedBuffer {
    pub fn new(buffer: wgpu::Buffer, size: u64, usage: BufferUsages) -> Self {
        Self {
            buffer: Arc::new(buffer),
            id: BufferId::new(),

            size,
            usage,
        }
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn id(&self) -> BufferId {
        self.id
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn usage(&self) -> BufferUsages {
        self.usage
    }
}

impl Deref for SharedBuffer {
    type Target = wgpu::Buffer;

    fn deref(&self) -> &Self::Target {
        self.buffer()
    }
}

impl PartialEq for SharedBuffer {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl Eq for SharedBuffer {}
