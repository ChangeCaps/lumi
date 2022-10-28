mod data;

pub use data::*;

use std::path::Path;

use lumi_material::MeshNode;

pub trait OpenGltfExt: Sized {
    fn open_gltf(path: impl AsRef<Path>) -> Result<Self, gltf::Error>;
}

impl OpenGltfExt for MeshNode {
    fn open_gltf(path: impl AsRef<Path>) -> Result<Self, gltf::Error> {
        Ok(GltfData::open(path)?.create_mesh_node())
    }
}
