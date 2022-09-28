#![deny(unsafe_op_in_unsafe_fn)]
#![doc(html_favicon_url = "https://i.imgur.com/XTQGS8H.png")]
#![doc(html_logo_url = "https://i.imgur.com/XTQGS8H.png")]

pub mod bind;
pub mod binding;
pub mod bloom;
mod buffer;
pub mod camera;
mod device;
pub mod frame_buffer;
pub mod id;
pub mod image;
pub mod key_map;
pub mod light;
pub mod material;
pub mod mesh;
pub mod pbr;
mod queue;
pub mod renderable;
pub mod renderer;
pub mod resources;
mod sampler;
pub mod shader;
mod texture;
pub mod tone_mapping;
pub mod world;

pub use buffer::SharedBuffer;
pub use device::SharedDevice;
pub use queue::SharedQueue;
pub use sampler::SharedSampler;
pub use texture::{SharedTexture, SharedTextureView};

#[doc(hidden)]
pub use wgpu::*;

pub mod math {
    pub use glam::{swizzles::*, *};
}

pub mod prelude {
    pub use crate::math::*;

    pub use crate::bind::Bind;
    pub use crate::camera::{Camera, CameraTarget, Orthographic, Perspective, Projection};
    pub use crate::id::{CameraId, LightId, NodeId};
    pub use crate::image::{Image, ImageData, NormalMap};
    pub use crate::light::{DirectionalLight, PointLight};
    pub use crate::material::{Material, MeshNode};
    pub use crate::mesh::{shape, Mesh};
    pub use crate::pbr::PbrMaterial;
    pub use crate::renderer::RenderTarget;
    pub use crate::shader::{Shader, ShaderRef};
    pub use crate::world::{Node, World};
    pub use crate::{SharedDevice, SharedQueue};
}
