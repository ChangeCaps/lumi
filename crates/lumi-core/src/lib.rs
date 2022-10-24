#![deny(unsafe_op_in_unsafe_fn)]

mod bind_group;
mod bind_key;
mod binding;
mod buffer;
mod default_binding;
mod draw_command;
mod image;
mod render_pipeline;
mod render_target;
mod resources;
mod sampler;
mod sampler_binding;
mod shared_device;
mod storage_binding;
mod storage_texture_binding;
mod texture;
mod texture_binding;
mod uniform_binding;

#[cfg(feature = "assets")]
mod handle_bindings;

pub use self::image::*;
pub use bind_group::*;
pub use bind_key::*;
pub use binding::*;
pub use buffer::*;
pub use default_binding::*;
pub use draw_command::*;
pub use render_pipeline::*;
pub use render_target::*;
pub use resources::*;
pub use sampler::*;
pub use sampler_binding::*;
pub use shared_device::*;
pub use storage_binding::*;
pub use storage_texture_binding::*;
pub use texture::*;
pub use texture_binding::*;
pub use uniform_binding::*;

pub use wgpu::{util::BufferInitDescriptor, *};

#[doc(hidden)]
pub use encase;

pub trait ShaderType: encase::ShaderType + encase::internal::WriteInto {}

impl<T> ShaderType for T where Self: encase::ShaderType + encase::internal::WriteInto {}
