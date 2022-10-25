#![deny(unsafe_op_in_unsafe_fn)]
#![doc(html_favicon_url = "https://i.imgur.com/XTQGS8H.png")]
#![doc(html_logo_url = "https://i.imgur.com/XTQGS8H.png")]

pub use lumi_assets as assets;
pub use lumi_bake as bake;
pub use lumi_core as core;
#[cfg(feature = "gltf")]
pub use lumi_gltf as gltf;
pub use lumi_id as id;
#[cfg(feature = "material")]
pub use lumi_material as material;
pub use lumi_mesh as mesh;
pub use lumi_renderer as renderer;
pub use lumi_shader as shader;
pub use lumi_task as task;
pub use lumi_util as util;
pub use lumi_world as world;

pub use lumi_macro::ShaderType;

pub use lumi_util::math;

pub mod prelude {
    pub use crate::DefaultPlugin;
    pub use lumi_bind::Bind;
    pub use lumi_core::{Image, ImageData, StorageBuffer, UniformBuffer};
    #[cfg(feature = "gltf")]
    pub use lumi_gltf::OpenGltfExt;
    pub use lumi_macro::*;
    pub use lumi_material::{Material, MeshNode, Primitive, StandardMaterial};
    pub use lumi_mesh::{shape, Mesh, MeshId};
    pub use lumi_renderer::{
        PhaseContext, PhaseLabel, RenderPhase, RenderPlugin, RenderViewPhase, Renderer,
        RendererBuilder, ViewPhaseContext,
    };
    pub use lumi_shader::{DefaultShader, Shader, ShaderRef};
    pub use lumi_util::math::*;
    pub use lumi_world::{
        AmbientLight, Camera, CameraId, CameraTarget, DirectionalLight, Environment, EnvironmentId,
        EnvironmentSource, Light, LightId, Node, NodeId, Orthographic, Perspective, PointLight,
        Projection, World,
    };
}

use material::MaterialPlugin;
use renderer::{PostProcessPlugin, PreparePlugin, RenderPlugin, RendererBuilder, SkyPlugin};

#[derive(Clone, Copy, Debug, Default)]
pub struct DefaultPlugin;

impl RenderPlugin for DefaultPlugin {
    fn build(&self, builder: &mut RendererBuilder) {
        builder
            .add_plugin(PreparePlugin)
            .add_plugin(SkyPlugin)
            .add_plugin(PostProcessPlugin)
            .add_plugin(MaterialPlugin::default())
            .add_asset_loader(core::ImageLoader);

        #[cfg(feature = "gltf")]
        builder.add_asset_loader(gltf::GltfLoader);
    }
}
