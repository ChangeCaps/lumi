#![doc(html_favicon_url = "https://i.imgur.com/XTQGS8H.png")]
#![doc(html_logo_url = "https://i.imgur.com/XTQGS8H.png")]
#![deny(unsafe_op_in_unsafe_fn)]

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
pub use lumi_util as util;

pub use lumi_macro::ShaderType;

pub use lumi_util::math;

pub mod prelude {
    pub use crate::DefaultPlugin;
    pub use lumi_bind::Bind;
    pub use lumi_core::{Image, ImageData, StorageBuffer, UniformBuffer};
    #[cfg(feature = "gltf")]
    pub use lumi_gltf::OpenGltfExt;
    pub use lumi_macro::*;
    pub use lumi_material::{
        Material, MaterialBundle, MaterialPlugin, Primitive, Primitives, StandardMaterial,
    };
    pub use lumi_mesh::{shape, Mesh, MeshId};
    pub use lumi_renderer::{
        Camera, DirectionalLight, DirectionalLightBundle, Entity, Environment, GlobalTransform,
        Mut, Orthographic, OrthographicCameraBundle, OwnedPtr, OwnedPtrMut, Perspective,
        PerspectiveCameraBundle, PointLight, PointLightBundle, Query, QueryState, Renderer,
        RendererPlugin, Transform, With, Without, World,
    };
    pub use lumi_shader::{DefaultShader, Shader, ShaderDefs, ShaderRef};
    pub use lumi_util::math::*;
}

use material::{MaterialPlugin, Primitive, Primitives, StandardMaterial};
use renderer::{CoreExtractPlugin, CorePlugin, ExtractMeshPlugin, Renderer, RendererPlugin};

#[derive(Clone, Copy, Debug, Default)]
pub struct DefaultPlugin;

impl RendererPlugin for DefaultPlugin {
    fn build(&self, renderer: &mut Renderer) {
        renderer
            .add_plugin(CorePlugin)
            .add_plugin(CoreExtractPlugin)
            .add_plugin(MaterialPlugin::<StandardMaterial>::default())
            .add_plugin(ExtractMeshPlugin::<Primitive>::default())
            .add_plugin(ExtractMeshPlugin::<Primitives>::default());
    }
}
