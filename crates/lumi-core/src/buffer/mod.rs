mod storage;
mod uniform;

use lumi_id::Id;
use wgpu::BufferUsages;

use std::{ops::Deref, sync::Arc};

pub use storage::*;
pub use uniform::*;

use crate::{SharedBindingResource, SharedBufferBinding};

#[derive(Debug)]
struct SharedBufferInner {
    buffer: wgpu::Buffer,
    id: Id<wgpu::Buffer>,
}

#[derive(Clone, Debug)]
pub struct SharedBuffer {
    inner: Arc<SharedBufferInner>,
}

impl SharedBuffer {
    #[inline]
    pub fn new(buffer: wgpu::Buffer) -> Self {
        Self {
            inner: Arc::new(SharedBufferInner {
                buffer,
                id: Id::new(),
            }),
        }
    }

    #[inline]
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.inner.buffer
    }

    #[inline]
    pub fn id(&self) -> Id<wgpu::Buffer> {
        self.inner.id
    }

    #[inline]
    pub fn as_entire_binding(self) -> SharedBindingResource {
        SharedBindingResource::Buffer(SharedBufferBinding {
            buffer: self,
            offset: 0,
            size: None,
        })
    }

    #[inline]
    pub fn size(&self) -> u64 {
        self.inner.buffer.size()
    }

    #[inline]
    pub fn usage(&self) -> BufferUsages {
        self.inner.buffer.usage()
    }
}

impl AsRef<Id<wgpu::Buffer>> for SharedBuffer {
    #[inline]
    fn as_ref(&self) -> &Id<wgpu::Buffer> {
        &self.inner.id
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
