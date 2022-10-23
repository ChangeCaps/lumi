use std::{
    any::Any,
    borrow::Cow,
    num::{NonZeroU32, NonZeroU64},
};

use wgpu::{
    BindingResource, BindingType, BufferBinding, BufferBindingType, Device, Queue,
    SamplerBindingType, ShaderStages, StorageTextureAccess, TextureFormat, TextureSampleType,
    TextureViewDimension,
};

use crate::{BindKey, SharedBuffer, SharedSampler, SharedTextureView};

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
    pub visibility: ShaderStages,
    pub ty: BindingType,
    pub count: Option<NonZeroU32>,
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

    fn bind_key(&self) -> BindKey;

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

    fn bind_key(&self) -> BindKey;

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

    fn bind_key(&self) -> BindKey;

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
                access: StorageTextureAccess::WriteOnly,
                format: TextureFormat::Rgba8UnormSrgb,
                view_dimension: TextureViewDimension::D2,
            },
            count: None,
        }
    }

    fn bind_key(&self) -> BindKey;

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

    fn bind_key(&self) -> BindKey;

    fn binding(
        &self,
        device: &Device,
        queue: &Queue,
        state: &mut Self::State,
    ) -> SharedBindingResource;
}
