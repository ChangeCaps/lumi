use lumi_mesh::Mesh;
use lumi_util::math::Mat4;

#[derive(Clone, Debug)]
pub struct MeshTransform {
    pub mesh: Mesh,
    pub transform: Mat4,
}

impl MeshTransform {
    #[inline]
    pub fn new(mesh: &Mesh, transform: Mat4) -> Self {
        Self {
            mesh: mesh.clone(),
            transform,
        }
    }
}
