mod data;

pub use data::*;
use lumi_material::Primitives;

use std::path::Path;

pub trait OpenGltfExt: Sized {
    fn open_gltf(path: impl AsRef<Path>) -> Result<Self, gltf::Error>;
}

impl OpenGltfExt for Primitives {
    fn open_gltf(path: impl AsRef<Path>) -> Result<Self, gltf::Error> {
        let data = GltfData::open(path)?;
        Ok(data.create_primitives())
    }
}
