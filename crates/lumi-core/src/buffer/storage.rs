use std::ops::{Deref, DerefMut};

use lumi_util::{once_cell::sync::OnceCell, smallvec::SmallVec, AtomicMarker};
use wgpu::{util::BufferInitDescriptor, BufferUsages, Device, Queue};

use crate::{
    BindKey, ShaderType, SharedBindingResource, SharedBuffer, SharedDevice, StorageBinding,
};

#[derive(Debug, Default)]
pub struct StorageBuffer<T> {
    buffer: OnceCell<SharedBuffer>,
    write: AtomicMarker,
    data: T,
}

impl<T> StorageBuffer<T> {
    #[inline]
    pub fn new(data: T) -> Self {
        Self {
            buffer: Default::default(),
            write: Default::default(),
            data,
        }
    }

    #[inline]
    pub fn into_data(self) -> T {
        self.data
    }
}

impl<T> StorageBuffer<T>
where
    T: ShaderType,
{
    #[inline]
    pub fn bytes(&self) -> SmallVec<[u8; 64]> {
        let size = self.data.size().get() as usize;
        let mut data = SmallVec::<[u8; 64]>::with_capacity(size);
        data.resize(size, 0);

        let mut buffer = encase::UniformBuffer::new(data.as_mut_slice());
        buffer.write(&self.data).unwrap();

        data
    }

    #[inline]
    fn create_buffer(device: &Device, data: &[u8]) -> SharedBuffer {
        device.create_shared_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: data,
            usage: BufferUsages::STORAGE,
        })
    }

    #[inline]
    pub fn buffer(&self, device: &Device, queue: &Queue) -> SharedBuffer {
        let should_write = self.write.take();

        if let Some(buffer) = self.buffer.get() {
            if should_write {
                queue.write_buffer(&buffer, 0, self.bytes().as_slice());
            }

            buffer.clone()
        } else {
            let buffer = Self::create_buffer(device, self.bytes().as_slice());
            let _ = self.buffer.set(buffer.clone());
            buffer
        }
    }
}

impl<T> Deref for StorageBuffer<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for StorageBuffer<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.write.mark();

        &mut self.data
    }
}

impl<T: ShaderType> StorageBinding for StorageBuffer<T> {
    type State = ();

    #[inline]
    fn bind_key(&self) -> BindKey {
        BindKey::from_hash(self.bytes())
    }

    #[inline]
    fn binding(
        &self,
        device: &Device,
        queue: &Queue,
        _state: &mut Self::State,
    ) -> SharedBindingResource {
        self.buffer(device, queue).as_entire_binding()
    }
}

impl<T: ShaderType> StorageBinding for &StorageBuffer<T> {
    type State = ();

    #[inline]
    fn bind_key(&self) -> BindKey {
        StorageBinding::bind_key(*self)
    }

    #[inline]
    fn binding(
        &self,
        device: &Device,
        queue: &Queue,
        state: &mut Self::State,
    ) -> SharedBindingResource {
        StorageBuffer::binding(*self, device, queue, state)
    }
}
