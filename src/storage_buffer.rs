use std::{
    ops::{Deref, DerefMut},
    sync::Mutex,
};

use encase::{internal::WriteInto, ShaderSize, ShaderType};
use wgpu::{util::BufferInitDescriptor, BufferUsages};

use crate::{
    SharedBindingResource, SharedBuffer, SharedBufferBinding, SharedDevice, SharedQueue,
    StorageBinding,
};

struct StorageBufferInner {
    buffer: SharedBuffer,
    write: bool,
}

pub struct StorageBuffer<T> {
    inner: Mutex<Option<StorageBufferInner>>,
    data: Vec<T>,
}

impl<T> StorageBuffer<T> {
    pub fn new(data: Vec<T>) -> Self {
        Self {
            inner: Mutex::new(None),
            data,
        }
    }

    pub fn into_data(self) -> Vec<T> {
        self.data
    }
}

impl<T> Default for StorageBuffer<T> {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl<T> Deref for StorageBuffer<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for StorageBuffer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        if let Some(inner) = self.inner.get_mut().unwrap() {
            inner.write = true;
        }

        &mut self.data
    }
}

impl<T> StorageBuffer<T>
where
    T: ShaderType + ShaderSize + WriteInto,
{
    pub fn bytes(&self) -> Vec<u8> {
        let mut data = encase::StorageBuffer::new(Vec::<u8>::new());
        data.write(&self.data).unwrap();
        data.into_inner()
    }

    fn create_buffer(device: &SharedDevice, data: &[u8]) -> SharedBuffer {
        device.create_shared_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: data,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        })
    }

    pub fn buffer(&self, device: &SharedDevice, queue: &SharedQueue) -> SharedBuffer {
        let mut inner = self.inner.lock().unwrap();
        let data = self.bytes();

        if let Some(inner) = inner.as_mut() {
            if inner.buffer.size() < data.len() as u64 {
                inner.buffer = Self::create_buffer(device, &data);
            } else if inner.write {
                queue.write_buffer(&inner.buffer, 0, &data);
                inner.write = false;
            }

            inner.buffer.clone()
        } else {
            let buffer = Self::create_buffer(device, &data);

            *inner = Some(StorageBufferInner {
                buffer: buffer.clone(),
                write: false,
            });

            buffer
        }
    }
}

impl<T> StorageBinding for StorageBuffer<T>
where
    T: ShaderType + ShaderSize + WriteInto,
{
    type State = ();

    fn binding(
        &self,
        device: &SharedDevice,
        queue: &SharedQueue,
        _state: &mut Self::State,
    ) -> SharedBindingResource {
        SharedBindingResource::Buffer(SharedBufferBinding {
            buffer: self.buffer(device, queue),
            offset: 0,
            size: None,
        })
    }
}

impl<T> StorageBinding for &StorageBuffer<T>
where
    T: ShaderType + ShaderSize + WriteInto,
{
    type State = ();

    fn binding(
        &self,
        device: &SharedDevice,
        queue: &SharedQueue,
        _state: &mut Self::State,
    ) -> SharedBindingResource {
        (*self).binding(device, queue, _state)
    }
}
