use std::{
    iter::{self, Map, Once},
    slice::Iter,
};

use deref_derive::{Deref, DerefMut};
use lumi_mesh::Mesh;
use lumi_renderer::ExtractMeshes;
use shiv::{query::QueryItem, storage::DenseStorage, world::Component};

use crate::{ExtractMaterials, Material, StandardMaterial};

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

impl<T: Send + Sync + 'static> ExtractMeshes for Primitive<T> {
    type Iter<'a> = Once<&'a Mesh>;

    #[inline]
    fn extract_meshes(&self) -> Self::Iter<'_> {
        iter::once(&self.mesh)
    }
}

impl<T: Material> ExtractMaterials for Primitive<T> {
    type Material = T;
    type MeshQuery = &'static Self;
    type Iter<'w> = Once<&'w Self::Material>;
    type MeshIter<'w> = Once<(&'w Self::Material, &'w Mesh)>;

    #[inline]
    fn extract(&self) -> Self {
        self.clone()
    }

    #[inline]
    fn iter(&self) -> Self::Iter<'_> {
        iter::once(&self.material)
    }

    #[inline]
    fn mesh_iter<'w>(item: &'w QueryItem<Self::MeshQuery>) -> Self::MeshIter<'w> {
        iter::once((&item.material, &item.mesh))
    }
}

#[derive(Clone, Debug, Default, Deref, DerefMut)]
pub struct Primitives<T = StandardMaterial> {
    pub primitives: Vec<Primitive<T>>,
}

impl<T> Primitives<T> {
    #[inline]
    pub const fn new() -> Self {
        Self {
            primitives: Vec::new(),
        }
    }

    #[inline]
    pub fn add(&mut self, material: T, mesh: Mesh) {
        self.primitives.push(Primitive::new(material, mesh));
    }
}

impl<T: Send + Sync + 'static> Component for Primitives<T> {
    type Storage = DenseStorage;
}

impl<T: Send + Sync + 'static> ExtractMeshes for Primitives<T> {
    type Iter<'a> = Map<Iter<'a, Primitive<T>>, fn(&'a Primitive<T>) -> &'a Mesh>;

    #[inline]
    fn extract_meshes(&self) -> Self::Iter<'_> {
        self.primitives.iter().map(|p| &p.mesh)
    }
}

impl<T: Material> ExtractMaterials for Primitives<T> {
    type Material = T;
    type MeshQuery = &'static Self;
    type Iter<'w> = Map<Iter<'w, Primitive<T>>, fn(&'w Primitive<T>) -> &'w T>;
    type MeshIter<'w> = Map<Iter<'w, Primitive<T>>, fn(&'w Primitive<T>) -> (&'w T, &'w Mesh)>;

    #[inline]
    fn extract(&self) -> Self {
        self.clone()
    }

    #[inline]
    fn iter(&self) -> Self::Iter<'_> {
        self.primitives.iter().map(|p| &p.material)
    }

    #[inline]
    fn mesh_iter<'w>(item: &'w QueryItem<Self::MeshQuery>) -> Self::MeshIter<'w> {
        item.primitives.iter().map(|p| (&p.material, &p.mesh))
    }
}
