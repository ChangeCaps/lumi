#![deny(unsafe_op_in_unsafe_fn)]

mod bind;
mod binding;
mod buffer;
mod camera;
mod device;
mod frame_buffer;
mod id;
mod image;
mod key_map;
mod light;
mod material;
mod mesh;
mod pbr;
mod queue;
mod renderer;
mod sampler;
mod shader;
mod shader_io;
mod shader_processor;
mod storage_buffer;
mod texture;
mod tone_mapping;
mod world;

pub use self::image::*;
pub use bind::*;
pub use binding::*;
pub use buffer::*;
pub use camera::*;
pub use device::*;
pub use frame_buffer::*;
pub use id::*;
pub use key_map::*;
pub use light::*;
pub use material::*;
pub use mesh::*;
pub use pbr::*;
pub use queue::*;
pub use renderer::*;
pub use sampler::*;
pub use shader::*;
pub use shader_io::*;
pub use shader_processor::*;
pub use storage_buffer::*;
pub use texture::*;
pub use tone_mapping::*;
pub use world::*;

pub use wgpu::*;

pub mod math {
    pub use glam::{swizzles::*, *};
}

pub mod prelude {
    pub use crate::math::*;
    pub use crate::*;
}
