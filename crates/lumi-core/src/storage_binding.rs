use lumi_util::smallvec::SmallVec;
use wgpu::{
    util::BufferInitDescriptor, BindingType, BufferBindingType, BufferUsages, Device, Queue,
};

use crate::{
    BindKey, BindLayoutEntry, ShaderType, SharedBindingResource, SharedBuffer, SharedBufferBinding,
    SharedDevice, StorageBinding,
};

pub struct StorageBindingState {
    pub buffer: SharedBuffer,
    pub key: BindKey,
}

impl<T: ShaderType> StorageBinding for T {
    type State = Option<StorageBindingState>;

    #[inline]
    fn entry() -> BindLayoutEntry {
        BindLayoutEntry {
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }

    #[inline]
    fn bind_key(&self) -> BindKey {
        let mut data = SmallVec::<[u8; 256]>::new();
        data.resize(self.size().get() as usize, 0u8);
        let mut uniform_buffer = encase::UniformBuffer::new(data.as_mut_slice());
        uniform_buffer.write(self).unwrap();

        BindKey::from_hash(&data)
    }

    #[inline]
    fn binding(
        &self,
        device: &Device,
        queue: &Queue,
        state: &mut Self::State,
    ) -> SharedBindingResource {
        let mut data = SmallVec::<[u8; 256]>::new();
        data.resize(self.size().get() as usize, 0u8);
        let mut uniform_buffer = encase::UniformBuffer::new(data.as_mut_slice());
        uniform_buffer.write(self).unwrap();

        if let Some(state) = state {
            if state.buffer.size() < data.len() as u64 {
                let buffer = device.create_shared_buffer_init(&BufferInitDescriptor {
                    label: None,
                    contents: &data,
                    usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                });

                state.buffer = buffer.clone();
                state.key = BindKey::from_hash(&data);

                SharedBindingResource::Buffer(SharedBufferBinding {
                    buffer,
                    offset: 0,
                    size: None,
                })
            } else {
                if state.key != BindKey::from_hash(&data) {
                    queue.write_buffer(&state.buffer, 0, &data);
                    state.key = BindKey::from_hash(&data);
                }

                SharedBindingResource::Buffer(SharedBufferBinding {
                    buffer: state.buffer.clone(),
                    offset: 0,
                    size: None,
                })
            }
        } else {
            let buffer = device.create_shared_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: &data,
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            });

            let new_state = StorageBindingState {
                buffer: buffer.clone(),
                key: BindKey::from_hash(&data),
            };

            *state = Some(new_state);

            SharedBindingResource::Buffer(SharedBufferBinding {
                buffer,
                offset: 0,
                size: None,
            })
        }
    }
}

impl StorageBinding for SharedBuffer {
    type State = ();

    #[inline]
    fn bind_key(&self) -> BindKey {
        BindKey::from_hash(self.size())
    }

    #[inline]
    fn binding(
        &self,
        _device: &Device,
        _queue: &Queue,
        _state: &mut Self::State,
    ) -> SharedBindingResource {
        SharedBindingResource::Buffer(SharedBufferBinding {
            buffer: self.clone(),
            offset: 0,
            size: None,
        })
    }
}

impl StorageBinding for &SharedBuffer {
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
        StorageBinding::binding(*self, device, queue, state)
    }
}
