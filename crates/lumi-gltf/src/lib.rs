mod data;
mod loader;

use std::path::Path;

pub use data::*;
pub use loader::*;

use lumi_material::MeshNode;
use lumi_renderer::{RenderPlugin, RendererBuilder};

impl OpenGltfExt for MeshNode {
    fn open_gltf(path: impl AsRef<Path>) -> Result<Self, gltf::Error> {
        Ok(GltfData::open(path)?.create_mesh_node())
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GltfPlugin;

impl RenderPlugin for GltfPlugin {
    fn build(&self, builder: &mut RendererBuilder) {
        builder.add_asset_loader(GltfLoader);
    }
}
