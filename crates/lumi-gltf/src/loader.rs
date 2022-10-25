use lumi_assets::{AssetLoader, LoadContext};
use lumi_material::MeshNode;
use lumi_util::{async_trait, math::Mat4};

use crate::GltfData;

pub struct GltfLoader;

#[async_trait]
impl AssetLoader for GltfLoader {
    async fn load(&self, ctx: &LoadContext<'_>) -> Result<(), ()> {
        let (document, buffers, images) = gltf::import_slice(ctx.bytes).unwrap();
        let gltf_data = GltfData::new(&document, &buffers, &images);

        if ctx.handle.is::<MeshNode>() {
            let mut mesh_node = MeshNode::default();

            if let Some(scene) = document.default_scene() {
                for node in scene.nodes() {
                    load_mesh_node(&mut mesh_node, &gltf_data, node, Mat4::IDENTITY);
                }
            }

            ctx.handle.set(mesh_node).unwrap();
        }

        Ok(())
    }

    fn extensions(&self) -> &[&str] {
        &["gltf", "glb"]
    }
}

fn load_mesh_node(
    mesh_node: &mut MeshNode,
    gltf_data: &GltfData,
    node: gltf::Node,
    global_transform: Mat4,
) {
    let transform = global_transform * Mat4::from_cols_array_2d(&node.transform().matrix());

    if let Some(mesh) = node.mesh() {
        let mesh = &gltf_data.meshes[mesh.index()];
        let transform = transform * mesh.transform;

        for primitive in mesh.primitives.iter() {
            let mut mesh = primitive.mesh.clone();
            mesh.transform(transform);

            mesh_node.add_primitive(primitive.material.clone(), mesh);
        }
    }

    for child in node.children() {
        load_mesh_node(mesh_node, gltf_data, child, transform);
    }
}
