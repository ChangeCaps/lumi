use encase::{internal::WriteInto, ShaderType};
use wgpu::BufferUsages;

use crate::{
    id::BufferId, SharedBindingResource, SharedBufferBinding, SharedDevice, SharedQueue,
    UniformBinding,
};

use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
};

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

struct UniformBufferInner {
    buffer: SharedBuffer,
    write: bool,
}

pub struct UniformBuffer<T> {
    inner: Mutex<Option<UniformBufferInner>>,
    data: T,
}

impl<T> UniformBuffer<T> {
    pub fn new(data: T) -> Self {
        Self {
            inner: Mutex::new(None),
            data,
        }
    }

    pub fn into_data(self) -> T {
        self.data
    }
}

impl<T> UniformBuffer<T>
where
    T: ShaderType + WriteInto,
{
    pub fn bytes(&self) -> Vec<u8> {
        let mut data = encase::UniformBuffer::new(Vec::<u8>::new());
        data.write(&self.data).unwrap();
        data.into_inner()
    }

    pub fn buffer(&self, device: &SharedDevice, queue: &SharedQueue) -> SharedBuffer {
        let mut inner = self.inner.lock().unwrap();

        if let Some(inner) = inner.as_mut() {
            if inner.write {
                queue.write_buffer(&inner.buffer, 0, &self.bytes());

                inner.write = false;
            }

            inner.buffer.clone()
        } else {
            let buffer = device.create_shared_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: &self.bytes(),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            });

            *inner = Some(UniformBufferInner {
                buffer: buffer.clone(),
                write: false,
            });

            buffer
        }
    }
}

impl<T> Deref for UniformBuffer<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for UniformBuffer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        if let Some(inner) = self.inner.get_mut().unwrap() {
            inner.write = true;
        }

        &mut self.data
    }
}

impl<T> UniformBinding for UniformBuffer<T>
where
    T: ShaderType + WriteInto,
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

impl<T> UniformBinding for &UniformBuffer<T>
where
    T: ShaderType + WriteInto,
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
