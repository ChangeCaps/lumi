use lumi_mesh::Mesh;
use shiv::{storage::DenseStorage, world::Component};

use crate::StandardMaterial;

#[derive(Clone, Debug, Default)]
pub struct Primitive<T = StandardMaterial> {
    pub material: T,
    pub mesh: Mesh,
}

impl<T> Primitive<T> {
    pub fn new(material: T, mesh: Mesh) -> Self {
        Self { material, mesh }
    }
}

impl<T: Send + Sync + 'static> Component for Primitive<T> {
    type Storage = DenseStorage;
}
