use std::{
    borrow::Cow,
    collections::LinkedList,
    num::{NonZeroU32, NonZeroU64},
};

use wgpu::{BindingResource, BindingType, BufferBinding, BufferBindingType, ShaderStages};

use crate::{SharedBuffer, SharedDevice};

pub use lumi_macro::Bind;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SharedBufferBinding {
    pub buffer: SharedBuffer,
    pub offset: u64,
    pub size: Option<NonZeroU64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SharedBindingResource {
    Buffer(SharedBufferBinding),
}

impl SharedBindingResource {
    pub fn as_binding_resource(&self) -> BindingResource {
        match self {
            Self::Buffer(buffer) => BindingResource::Buffer(BufferBinding {
                buffer: buffer.buffer.buffer(),
                offset: buffer.offset,
                size: buffer.size,
            }),
        }
    }
}

#[derive(Clone, Debug)]
pub struct BindingLayoutEntry {
    pub name: Cow<'static, str>,
    pub group: u32,
    pub visibility: ShaderStages,
    pub ty: BindingType,
    pub count: Option<NonZeroU32>,
}

#[derive(Clone, Debug)]
pub struct SharedBinding {
    pub name: Cow<'static, str>,
    pub resource: SharedBindingResource,
    pub group: u32,
}

pub trait AsBinding {
    fn entry(name: Cow<'static, str>, group: u32) -> BindingLayoutEntry;
    fn as_binding(
        &self,
        device: &SharedDevice,
        name: Cow<'static, str>,
        group: u32,
    ) -> SharedBinding;
}

pub trait Bind {
    fn entries() -> LinkedList<BindingLayoutEntry>
    where
        Self: Sized;

    fn bindings(&self, device: &SharedDevice) -> Vec<SharedBinding>;
}

impl AsBinding for SharedBufferBinding {
    fn entry(name: Cow<'static, str>, group: u32) -> BindingLayoutEntry {
        BindingLayoutEntry {
            name,
            group,
            visibility: ShaderStages::all(),
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }

    fn as_binding(
        &self,
        _device: &SharedDevice,
        name: Cow<'static, str>,
        group: u32,
    ) -> SharedBinding {
        SharedBinding {
            name,
            resource: SharedBindingResource::Buffer(self.clone()),
            group,
        }
    }
}
