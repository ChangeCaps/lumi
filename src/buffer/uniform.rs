use std::{
    ops::{Deref, DerefMut},
    sync::Mutex,
};

use encase::{internal::WriteInto, ShaderType};
use wgpu::BufferUsages;

use crate::{
    bind::{SharedBindingResource, SharedBufferBinding, UniformBinding},
    bind_key::BindKey,
    Device, Queue, SharedBuffer, SharedDevice,
};

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

    pub fn buffer(&self, device: &Device, queue: &Queue) -> SharedBuffer {
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

    fn bind_key(&self) -> BindKey {
        BindKey::from_hash(&self.bytes())
    }

    fn binding(
        &self,
        device: &Device,
        queue: &Queue,
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

    fn bind_key(&self) -> BindKey {
        BindKey::from_hash(&self.bytes())
    }

    fn binding(
        &self,
        device: &Device,
        queue: &Queue,
        _state: &mut Self::State,
    ) -> SharedBindingResource {
        (*self).binding(device, queue, _state)
    }
}
