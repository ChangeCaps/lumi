use std::{
    any::Any,
    borrow::Cow,
    collections::LinkedList,
    num::{NonZeroU32, NonZeroU64},
};

use encase::{internal::WriteInto, ShaderType};
use once_cell::sync::OnceCell;
use wgpu::{
    BindingResource, BindingType, BufferBinding, BufferBindingType, BufferUsages,
    SamplerBindingType, ShaderStages, StorageTextureAccess, TextureFormat, TextureSampleType,
    TextureViewDimension,
};

pub use lumi_macro::Bind;

use crate::{
    bind_key::BindKey, Device, Queue, SharedBuffer, SharedDevice, SharedSampler, SharedTextureView,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SharedBufferBinding {
    pub buffer: SharedBuffer,
    pub offset: u64,
    pub size: Option<NonZeroU64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SharedBindingResource {
    Buffer(SharedBufferBinding),
    TextureView(SharedTextureView),
    Sampler(SharedSampler),
}

impl SharedBindingResource {
    pub fn as_binding_resource(&self) -> BindingResource {
        match self {
            Self::Buffer(buffer) => BindingResource::Buffer(BufferBinding {
                buffer: buffer.buffer.buffer(),
                offset: buffer.offset,
                size: buffer.size,
            }),
            Self::TextureView(view) => BindingResource::TextureView(view.view()),
            Self::Sampler(sampler) => BindingResource::Sampler(sampler.sampler()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct BindingLayoutEntry {
    pub name: Cow<'static, str>,
    pub state: fn() -> Box<dyn Any + Send + Sync>,
    pub visibility: ShaderStages,
    pub ty: BindingType,
    pub count: Option<NonZeroU32>,
}

pub trait Bind {
    fn entries() -> LinkedList<BindingLayoutEntry>
    where
        Self: Sized;

    fn get_uniform(
        &self,
        device: &Device,
        queue: &Queue,
        name: &str,
        state: &mut dyn Any,
    ) -> Option<SharedBindingResource>;

    fn get_storage(
        &self,
        device: &Device,
        queue: &Queue,
        name: &str,
        state: &mut dyn Any,
    ) -> Option<SharedBindingResource>;

    fn get_texture(
        &self,
        device: &Device,
        queue: &Queue,
        name: &str,
        state: &mut dyn Any,
    ) -> Option<SharedBindingResource>;

    fn get_storage_texture(
        &self,
        device: &Device,
        queue: &Queue,
        name: &str,
        state: &mut dyn Any,
    ) -> Option<SharedBindingResource>;

    fn get_sampler(
        &self,
        device: &Device,
        queue: &Queue,
        name: &str,
        state: &mut dyn Any,
    ) -> Option<SharedBindingResource>;
}

pub struct BindLayoutEntry {
    pub ty: BindingType,
    pub count: Option<NonZeroU32>,
}

impl BindLayoutEntry {
    pub fn into_layout_entry<T: Any + Default + Send + Sync>(
        self,
        name: impl Into<Cow<'static, str>>,
    ) -> BindingLayoutEntry {
        BindingLayoutEntry {
            name: name.into(),
            state: || Box::new(T::default()),
            visibility: ShaderStages::all(),
            ty: self.ty,
            count: self.count,
        }
    }
}

pub trait UniformBinding {
    type State: Any + Default;

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

    fn binding(
        &self,
        device: &Device,
        queue: &Queue,
        state: &mut Self::State,
    ) -> SharedBindingResource;
}

pub trait StorageBinding {
    type State: Any + Default;

    fn entry() -> BindLayoutEntry {
        BindLayoutEntry {
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }

    fn binding(
        &self,
        device: &Device,
        queue: &Queue,
        state: &mut Self::State,
    ) -> SharedBindingResource;
}

pub trait TextureBinding {
    type State: Any + Default;

    fn entry() -> BindLayoutEntry {
        BindLayoutEntry {
            ty: BindingType::Texture {
                sample_type: TextureSampleType::Float { filterable: true },
                view_dimension: TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        }
    }

    fn binding(
        &self,
        device: &Device,
        queue: &Queue,
        state: &mut Self::State,
    ) -> SharedBindingResource;
}

pub trait StorageTextureBinding {
    type State: Any + Default;

    fn entry() -> BindLayoutEntry {
        BindLayoutEntry {
            ty: BindingType::StorageTexture {
                access: StorageTextureAccess::ReadOnly,
                format: TextureFormat::Rgba8UnormSrgb,
                view_dimension: TextureViewDimension::D2,
            },
            count: None,
        }
    }

    fn binding(
        &self,
        device: &Device,
        queue: &Queue,
        state: &mut Self::State,
    ) -> SharedBindingResource;
}

pub trait SamplerBinding {
    type State: Any + Default;

    fn entry() -> BindLayoutEntry {
        BindLayoutEntry {
            ty: BindingType::Sampler(SamplerBindingType::Filtering),
            count: None,
        }
    }

    fn binding(
        &self,
        device: &Device,
        queue: &Queue,
        state: &mut Self::State,
    ) -> SharedBindingResource;
}

pub struct DefaultBindingState<T, U> {
    pub state: T,
    pub default_binding: OnceCell<U>,
}

impl<T: Default, U> Default for DefaultBindingState<T, U> {
    fn default() -> Self {
        Self {
            state: T::default(),
            default_binding: OnceCell::new(),
        }
    }
}

pub trait DefaultTexture {
    fn default_texture(device: &Device, queue: &Queue) -> SharedTextureView;
}

impl<T: TextureBinding + DefaultTexture> TextureBinding for Option<T> {
    type State = DefaultBindingState<T::State, SharedTextureView>;

    fn binding(
        &self,
        device: &Device,
        queue: &Queue,
        state: &mut Self::State,
    ) -> SharedBindingResource {
        match self {
            Some(texture) => texture.binding(device, queue, &mut state.state),
            None => {
                let view = state
                    .default_binding
                    .get_or_init(|| T::default_texture(device, queue));

                SharedBindingResource::TextureView(view.clone())
            }
        }
    }
}

pub trait DefaultSampler {
    fn default_sampler(device: &Device, queue: &Queue) -> SharedSampler;
}

impl<T: SamplerBinding + DefaultSampler> SamplerBinding for Option<T> {
    type State = DefaultBindingState<T::State, SharedSampler>;

    fn binding(
        &self,
        device: &Device,
        queue: &Queue,
        state: &mut Self::State,
    ) -> SharedBindingResource {
        match self {
            Some(texture) => texture.binding(device, queue, &mut state.state),
            None => {
                let sampler = state
                    .default_binding
                    .get_or_init(|| T::default_sampler(device, queue));

                SharedBindingResource::Sampler(sampler.clone())
            }
        }
    }
}

pub struct UniformBindingState {
    pub buffer: SharedBuffer,
    pub key: BindKey,
}

impl<T: ShaderType + WriteInto> UniformBinding for T {
    type State = Option<UniformBindingState>;

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

    fn binding(
        &self,
        device: &Device,
        queue: &Queue,
        state: &mut Self::State,
    ) -> SharedBindingResource {
        let mut data = encase::UniformBuffer::new(Vec::<u8>::new());
        data.write(self).unwrap();
        let data = data.into_inner();

        if let Some(state) = state {
            if state.buffer.size() < data.len() as u64 {
                let buffer = device.create_shared_buffer_init(&wgpu::util::BufferInitDescriptor {
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
            let buffer = device.create_shared_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: &data,
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            });

            let new_state = UniformBindingState {
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
