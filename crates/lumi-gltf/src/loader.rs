use std::path::Path;

use lumi_assets::{AssetLoader, LoadContext};
use lumi_material::MeshNode;
use lumi_util::async_trait;

use crate::GltfData;

pub trait OpenGltfExt: Sized {
    fn open_gltf(path: impl AsRef<Path>) -> Result<Self, gltf::Error>;
}

pub struct GltfLoader;

#[async_trait]
impl AssetLoader for GltfLoader {
    async fn load(&self, ctx: &LoadContext<'_>) -> Result<(), ()> {
        let (document, buffers, images) = gltf::import_slice(ctx.bytes).unwrap();
        let gltf_data = GltfData::new(document, &buffers, &images);

        if ctx.handle.is::<MeshNode>() {
            let mesh_node = gltf_data.create_mesh_node();
            let _ = ctx.handle.set(mesh_node);
        } else if ctx.handle.is::<GltfData>() {
            let _ = ctx.handle.set(gltf_data);
        }

        Ok(())
    }

    fn extensions(&self) -> &[&str] {
        &["gltf", "glb"]
    }
}
