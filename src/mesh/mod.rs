mod attribute;
pub mod shape;

use std::{collections::HashMap, sync::Arc};

pub use attribute::*;
use glam::Vec3;

use crate::MeshId;

#[derive(Clone, Debug, Default)]
pub struct Mesh {
    attributes: HashMap<String, Arc<MeshAttribute>>,
    indices: Option<Arc<Vec<u32>>>,
    id: MeshId,
}

impl Mesh {
    pub const POSITION: &'static str = "position";
    pub const NORMAL: &'static str = "normal";
    pub const TANGENT: &'static str = "tangent";
    pub const UV_0: &'static str = "uv_0";

    pub fn new() -> Self {
        Self {
            attributes: HashMap::new(),
            indices: None,
            id: MeshId::new(),
        }
    }

    pub fn id(&self) -> MeshId {
        self.id
    }

    pub fn insert_attribute(
        &mut self,
        name: impl Into<String>,
        attribute: impl Into<MeshAttribute>,
    ) {
        self.id = MeshId::new();

        self.attributes
            .insert(name.into(), Arc::new(attribute.into()));
    }

    pub fn remove_attribute<T: From<MeshAttribute>>(&mut self, name: impl AsRef<str>) -> Option<T> {
        self.id = MeshId::new();

        self.attributes.remove(name.as_ref()).map(|attribute| {
            T::from(Arc::try_unwrap(attribute).unwrap_or_else(|attr| attr.as_ref().clone()))
        })
    }

    pub fn has_attribute(&self, name: impl AsRef<str>) -> bool {
        self.attributes.contains_key(name.as_ref())
    }

    pub fn attribute_len(&self, name: impl AsRef<str>) -> usize {
        self.attributes
            .get(name.as_ref())
            .map(|attribute| attribute.len())
            .unwrap_or(0)
    }

    pub fn attribute<T: AsMeshAttribute + ?Sized>(&self, name: impl AsRef<str>) -> Option<&T> {
        self.attributes
            .get(name.as_ref())
            .and_then(|attribute| T::as_mesh_attribute(attribute))
    }

    pub fn attribute_mut<T: AsMeshAttribute + ?Sized>(
        &mut self,
        name: impl AsRef<str>,
    ) -> Option<&mut T> {
        self.id = MeshId::new();

        self.attributes
            .get_mut(name.as_ref())
            .and_then(|attribute| T::as_mesh_attribute_mut(Arc::make_mut(attribute)))
    }

    pub fn attributes(&self) -> impl Iterator<Item = (&str, &MeshAttribute)> {
        self.attributes
            .iter()
            .map(|(name, attribute)| (name.as_ref(), attribute.as_ref()))
    }

    pub fn insert_indices(&mut self, indices: impl Into<Vec<u32>>) {
        self.id = MeshId::new();

        self.indices = Some(Arc::new(indices.into()));
    }

    pub fn remove_indices(&mut self) -> Option<Vec<u32>> {
        self.id = MeshId::new();

        self.indices.take().map(|indices| {
            Arc::try_unwrap(indices).unwrap_or_else(|indices| indices.as_ref().clone())
        })
    }

    pub fn indices(&self) -> Option<&Vec<u32>> {
        self.indices.as_deref()
    }

    pub fn indices_mut(&mut self) -> Option<&mut Vec<u32>> {
        self.id = MeshId::new();

        self.indices.as_mut().map(Arc::make_mut)
    }

    pub fn generate_normals(&mut self) {
        if let Some(positions) = self.attribute::<[Vec3]>(Self::POSITION) {
            let mut normals: Vec<Vec3> = Vec::with_capacity(positions.len());

            if let Some(indices) = self.indices() {
                for i in (0..indices.len()).step_by(3) {
                    let a = positions[indices[i + 0] as usize];
                    let b = positions[indices[i + 1] as usize];
                    let c = positions[indices[i + 2] as usize];

                    let normal = (b - a).cross(c - a).normalize();

                    normals.push(normal);
                    normals.push(normal);
                    normals.push(normal);
                }

                for normal in normals.iter_mut() {
                    *normal = normal.normalize();
                }
            } else {
                for i in (0..positions.len()).step_by(3) {
                    let a = positions[i + 0];
                    let b = positions[i + 1];
                    let c = positions[i + 2];

                    let normal = (b - a).cross(c - a).normalize();

                    normals.push(normal);
                    normals.push(normal);
                    normals.push(normal);
                }
            }

            self.insert_attribute(Self::NORMAL, normals);
        }
    }

    pub fn generate_tangents(&mut self) -> bool {
        if !self.has_attribute(Self::POSITION)
            || !self.has_attribute(Self::NORMAL)
            || !self.has_attribute(Self::UV_0)
        {
            return false;
        }

        let tangents = vec![[0.0; 4]; self.attribute_len(Self::POSITION)];
        self.insert_attribute(Self::TANGENT, tangents);

        mikktspace::generate_tangents(self)
    }
}

impl mikktspace::Geometry for Mesh {
    fn num_faces(&self) -> usize {
        match self.indices() {
            Some(indices) => indices.len() / 3,
            None => self.attribute_len(Self::POSITION) / 3,
        }
    }

    fn num_vertices_of_face(&self, _face: usize) -> usize {
        3
    }

    fn position(&self, face: usize, vert: usize) -> [f32; 3] {
        match self.indices() {
            Some(indices) => {
                let index = indices[face * 3 + vert] as usize;

                self.attribute::<[[f32; 3]]>(Self::POSITION).unwrap()[index]
            }
            None => self.attribute::<[[f32; 3]]>(Self::POSITION).unwrap()[face * 3 + vert],
        }
    }

    fn normal(&self, face: usize, vert: usize) -> [f32; 3] {
        match self.indices() {
            Some(indices) => {
                let index = indices[face * 3 + vert] as usize;

                self.attribute::<[[f32; 3]]>(Self::NORMAL).unwrap()[index]
            }
            None => self.attribute::<[[f32; 3]]>(Self::NORMAL).unwrap()[face * 3 + vert],
        }
    }

    fn tex_coord(&self, face: usize, vert: usize) -> [f32; 2] {
        match self.indices() {
            Some(indices) => {
                let index = indices[face * 3 + vert] as usize;

                self.attribute::<[[f32; 2]]>(Self::UV_0).unwrap()[index]
            }
            None => self.attribute::<[[f32; 2]]>(Self::UV_0).unwrap()[face * 3 + vert],
        }
    }

    fn set_tangent_encoded(&mut self, tangent: [f32; 4], face: usize, vert: usize) {
        match self.indices() {
            Some(indices) => {
                let index = indices[face * 3 + vert] as usize;

                self.attribute_mut::<[[f32; 4]]>(Self::TANGENT).unwrap()[index] = tangent;
            }
            None => {
                self.attribute_mut::<[[f32; 4]]>(Self::TANGENT).unwrap()[face * 3 + vert] = tangent
            }
        }
    }
}
