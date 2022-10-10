mod storage;
mod uniform;

use wgpu::BufferUsages;

use crate::id::BufferId;

use std::{ops::Deref, sync::Arc};

pub use storage::*;
pub use uniform::*;

#[derive(Debug)]
struct SharedBufferInner {
    buffer: wgpu::Buffer,
    id: BufferId,

    usage: BufferUsages,
    size: u64,
}

#[derive(Clone, Debug)]
pub struct SharedBuffer {
    inner: Arc<SharedBufferInner>,
}

impl SharedBuffer {
    #[inline]
    pub fn new(buffer: wgpu::Buffer, size: u64, usage: BufferUsages) -> Self {
        Self {
            inner: Arc::new(SharedBufferInner {
                buffer,
                id: BufferId::new(),
                usage,
                size,
            }),
        }
    }

    #[inline]
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.inner.buffer
    }

    #[inline]
    pub fn id(&self) -> BufferId {
        self.inner.id
    }

    #[inline]
    pub fn size(&self) -> u64 {
        self.inner.size
    }

    #[inline]
    pub fn usage(&self) -> BufferUsages {
        self.inner.usage
    }
}

impl Deref for SharedBuffer {
    type Target = wgpu::Buffer;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.buffer()
    }
}

impl PartialEq for SharedBuffer {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl Eq for SharedBuffer {}
