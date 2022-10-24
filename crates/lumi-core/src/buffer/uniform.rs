use std::ops::{Deref, DerefMut};

use lumi_util::{once_cell::sync::OnceCell, smallvec::SmallVec, AtomicMarker};
use wgpu::{BufferUsages, Device, Queue};

use crate::{
    BindKey, ShaderType, SharedBindingResource, SharedBuffer, SharedDevice, UniformBinding,
};

#[derive(Debug, Default)]
pub struct UniformBuffer<T> {
    buffer: OnceCell<SharedBuffer>,
    write: AtomicMarker,
    data: T,
}

impl<T> UniformBuffer<T> {
    #[inline]
    pub fn new(data: T) -> Self {
        Self {
            buffer: Default::default(),
            write: Default::default(),
            data,
        }
    }

    #[inline]
    pub fn set(&mut self, data: T) {
        self.data = data;
        self.write.mark();
    }

    #[inline]
    pub fn into_data(self) -> T {
        self.data
    }
}

impl<T> UniformBuffer<T>
where
    T: ShaderType,
{
    #[inline]
    pub fn bytes(&self) -> SmallVec<[u8; 64]> {
        let size = self.data.size().get() as usize;
        let mut data = SmallVec::<[u8; 64]>::with_capacity(size);
        data.resize(size, 0);

        let mut buffer = encase::StorageBuffer::new(data.as_mut_slice());
        buffer.write(&self.data).unwrap();

        data
    }

    #[inline]
    fn create_buffer(&self, device: &Device) -> SharedBuffer {
        device.create_shared_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: self.bytes().as_slice(),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        })
    }

    #[inline]
    pub fn buffer(&self, device: &Device, queue: &Queue) -> SharedBuffer {
        let should_write = self.write.take();

        if let Some(buffer) = self.buffer.get() {
            if should_write {
                queue.write_buffer(&buffer, 0, self.bytes().as_slice());
            }

            return buffer.clone();
        } else {
            let buffer = self.create_buffer(device);
            let _ = self.buffer.set(buffer.clone());
            buffer
        }
    }
}

impl<T> Deref for UniformBuffer<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for UniformBuffer<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.write.mark();

        &mut self.data
    }
}

impl<T: ShaderType> UniformBinding for UniformBuffer<T> {
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

impl<T: ShaderType> UniformBinding for &UniformBuffer<T> {
    type State = ();

    #[inline]
    fn bind_key(&self) -> BindKey {
        UniformBinding::bind_key(*self)
    }

    #[inline]
    fn binding(
        &self,
        device: &Device,
        queue: &Queue,
        state: &mut Self::State,
    ) -> SharedBindingResource {
        UniformBuffer::binding(*self, device, queue, state)
    }
}
