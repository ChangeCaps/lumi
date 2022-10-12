#![deny(unsafe_op_in_unsafe_fn)]
#![doc(html_favicon_url = "https://i.imgur.com/XTQGS8H.png")]
#![doc(html_logo_url = "https://i.imgur.com/XTQGS8H.png")]

pub mod aabb;
pub mod bind;
pub mod bind_group;
pub mod bind_key;
pub mod binding;
pub mod bloom;
pub mod buffer;
pub mod camera;
pub mod device;
pub mod environment;
pub mod frame_buffer;
pub mod fxaa;
#[cfg(feature = "gltf")]
pub mod gltf;
pub mod id;
pub mod image;
pub mod key_map;
pub mod light;
pub mod material;
pub mod mesh;
pub mod render_pipeline;
pub mod renderable;
pub mod renderer;
pub mod resources;
pub mod sampler;
pub mod shader;
pub mod shadow;
pub mod texture;
pub mod tone_mapping;
mod util;
pub mod world;

pub use bind_group::SharedBindGroup;
pub use buffer::SharedBuffer;
pub use device::SharedDevice;
pub use render_pipeline::SharedRenderPipeline;
pub use sampler::SharedSampler;
pub use texture::{SharedTexture, SharedTextureView};

pub use wgpu::{BindingType, Device, Queue, SamplerBindingType, TextureViewDimension};

pub use smallvec::SmallVec;

#[doc(hidden)]
pub use wgpu;

pub mod math {
    pub use glam::{swizzles::*, *};
}

pub mod prelude {
    pub use crate::math::*;

    pub use crate::bind::Bind;
    pub use crate::buffer::{SharedBuffer, StorageBuffer, UniformBuffer};
    pub use crate::camera::{Camera, CameraTarget, Orthographic, Perspective, Projection};
    pub use crate::device::SharedDevice;
    pub use crate::environment::{Environment, EnvironmentKind};
    pub use crate::id::{CameraId, LightId, NodeId};
    pub use crate::image::{Image, ImageData, NormalMap};
    pub use crate::light::{DirectionalLight, PointLight};
    pub use crate::material::{Material, MeshNode, StandardMaterial, UnlitMaterial};
    pub use crate::mesh::{shape, Mesh};
    pub use crate::renderable::Renderable;
    pub use crate::renderer::{
        PhaseContext, PhaseLabel, RenderPhase, RenderTarget, RenderViewPhase, Renderer,
        ViewPhaseContext,
    };
    pub use crate::resources::Resources;
    pub use crate::sampler::SharedSampler;
    pub use crate::shader::{Shader, ShaderRef};
    pub use crate::texture::{SharedTexture, SharedTextureView};
    pub use crate::world::{Node, World};
    pub use wgpu::{
        util::BufferInitDescriptor, AddressMode, BufferDescriptor, BufferUsages, Device, Extent3d,
        FilterMode, Queue, SamplerDescriptor, TextureAspect, TextureDescriptor, TextureDimension,
        TextureFormat, TextureUsages, TextureViewDescriptor, TextureViewDimension,
    };
}
