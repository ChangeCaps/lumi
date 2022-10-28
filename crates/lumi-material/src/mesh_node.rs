use std::any::TypeId;

use lumi_core::{Device, Queue, Resources};
use lumi_id::Id;
use lumi_mesh::Mesh;
use lumi_renderer::{PrepareMeshFunction, PrepareTransformFunction, ShadowRenderFunction};
use lumi_util::math::Mat4;
use lumi_world::{Extract, ExtractOne, Renderable};

use crate::{Material, MaterialRenderFunction, StandardMaterial};

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

#[derive(Clone, Debug)]
pub struct MeshNode<T = StandardMaterial> {
    pub primitives: Vec<Primitive<T>>,
    pub transform: Mat4,
}

impl<T> Default for MeshNode<T> {
    #[inline]
    fn default() -> Self {
        Self {
            primitives: Vec::new(),
            transform: Mat4::IDENTITY,
        }
    }
}

impl<T> MeshNode<T> {
    #[inline]
    pub fn new(material: T, mesh: Mesh, transform: Mat4) -> Self {
        Self {
            primitives: vec![Primitive::new(material, mesh)],
            transform,
        }
    }

    #[inline]
    pub fn add_primitive(&mut self, material: T, mesh: Mesh) {
        self.primitives.push(Primitive::new(material, mesh));
    }

    #[inline]
    pub fn with_primitive(mut self, material: T, mesh: Mesh) -> Self {
        self.add_primitive(material, mesh);
        self
    }
}

impl<T: Material> Extract<T> for MeshNode<T> {
    #[inline]
    fn extract(&self, extract: &mut dyn FnMut(&T)) {
        for primitive in &self.primitives {
            extract(&primitive.material);
        }
    }
}

impl<T: Material> Extract<Mesh> for MeshNode<T> {
    #[inline]
    fn extract(&self, extract: &mut dyn FnMut(&Mesh)) {
        for primitive in &self.primitives {
            extract(&primitive.mesh);
        }
    }
}

impl<T: Material> Extract<Primitive<T>> for MeshNode<T> {
    #[inline]
    fn extract(&self, extract: &mut dyn FnMut(&Primitive<T>)) {
        for primitive in &self.primitives {
            extract(primitive);
        }
    }
}

impl<T: Material> Extract<Mat4> for MeshNode<T> {
    #[inline]
    fn extract(&self, extract: &mut dyn FnMut(&Mat4)) {
        extract(&self.transform);
    }
}

impl<T: Material> ExtractOne<Mat4> for MeshNode<T> {
    #[inline]
    fn extract_one(&self) -> Option<&Mat4> {
        Some(&self.transform)
    }
}

impl<T: Material> Renderable for MeshNode<T> {
    fn register(_: &Device, _: &Queue, resources: &mut Resources) {
        resources.insert_id(
            Id::from_hash(TypeId::of::<T>()),
            PrepareMeshFunction::new::<Self>(),
        );
        resources.insert_id(
            Id::from_hash(TypeId::of::<T>()),
            PrepareTransformFunction::new::<Self>(),
        );
        resources.insert_id(
            Id::from_hash(TypeId::of::<T>()),
            ShadowRenderFunction::new::<Self>(),
        );
        resources.insert_id(
            Id::from_hash(TypeId::of::<T>()),
            MaterialRenderFunction::new::<MeshNode<T>, T>(),
        );
    }
}
