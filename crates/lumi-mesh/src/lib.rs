mod attribute;
#[cfg(feature = "generate_normals")]
mod generate_normals;
#[cfg(feature = "generate_tangents")]
mod generate_tangents;
#[cfg(feature = "shape")]
pub mod shape;

pub use attribute::*;

#[cfg(feature = "bounds")]
use lumi_bounds::Aabb;
use lumi_id::Id;
use lumi_util::{
    bytemuck,
    math::{Mat4, Vec2, Vec3},
    HashMap, SharedState,
};

pub type MeshId = Id<Mesh>;

/// A mesh is a collection of vertices and indices.
///
/// **Note** data is cloned on write.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "shiv", derive(shiv::world::Component))]
pub struct Mesh {
    attributes: HashMap<String, SharedState<MeshAttribute>>,
    indices: Option<SharedState<Vec<u32>>>,
    id: MeshId,
}

impl Mesh {
    pub const POSITION: &'static str = "position";
    pub const NORMAL: &'static str = "normal";
    pub const TANGENT: &'static str = "tangent";
    pub const UV_0: &'static str = "uv_0";

    /// Creates a new mesh.
    pub fn new() -> Self {
        Self {
            attributes: HashMap::default(),
            indices: None,
            id: MeshId::new(),
        }
    }

    /// Returns the mesh id.
    pub fn id(&self) -> MeshId {
        self.id
    }

    /// Inserts a new attribute.
    pub fn insert_attribute(
        &mut self,
        name: impl Into<String>,
        attribute: impl Into<MeshAttribute>,
    ) {
        self.id = MeshId::new();

        self.attributes
            .insert(name.into(), SharedState::new(attribute.into()));
    }

    /// Removes an attribute.
    pub fn remove_attribute<T: From<MeshAttribute>>(&mut self, name: impl AsRef<str>) -> Option<T> {
        self.id = MeshId::new();

        self.attributes
            .remove(name.as_ref())
            .map(|attribute| T::from(attribute.into_inner()))
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
        self.get_attribute(name.as_ref())
            .and_then(T::as_mesh_attribute)
    }

    pub fn attribute_mut<T: AsMeshAttribute + ?Sized>(
        &mut self,
        name: impl AsRef<str>,
    ) -> Option<&mut T> {
        self.get_attribute_mut(name.as_ref())
            .and_then(T::as_mesh_attribute_mut)
    }

    pub fn get_attribute(&self, name: impl AsRef<str>) -> Option<&MeshAttribute> {
        self.attributes.get(name.as_ref()).map(SharedState::get)
    }

    pub fn get_attribute_mut(&mut self, name: impl AsRef<str>) -> Option<&mut MeshAttribute> {
        self.id = MeshId::new();

        self.attributes
            .get_mut(name.as_ref())
            .map(SharedState::get_mut)
    }

    pub fn attributes(&self) -> impl Iterator<Item = (&str, &MeshAttribute)> {
        self.attributes
            .iter()
            .map(|(name, attribute)| (name.as_ref(), attribute.as_ref()))
    }

    pub fn insert_indices(&mut self, indices: impl Into<Vec<u32>>) {
        self.id = MeshId::new();

        self.indices = Some(SharedState::new(indices.into()));
    }

    pub fn remove_indices(&mut self) -> Option<Vec<u32>> {
        self.id = MeshId::new();

        self.indices.take().map(|indices| indices.into_inner())
    }

    pub fn indices(&self) -> Option<&Vec<u32>> {
        self.indices.as_deref()
    }

    pub fn indices_mut(&mut self) -> Option<&mut Vec<u32>> {
        self.id = MeshId::new();

        self.indices.as_mut().map(SharedState::get_mut)
    }

    pub fn indices_as_bytes(&self) -> Option<&[u8]> {
        self.indices
            .as_deref()
            .map(|indices| bytemuck::cast_slice(indices.as_ref()))
    }

    #[cfg(feature = "bounds")]
    pub fn aabb(&self) -> Option<Aabb> {
        let positions = self.positions()?;

        if positions.is_empty() {
            return None;
        }

        let mut aabb = Aabb::ZERO;

        for &position in positions {
            aabb.add_point(position);
        }

        Some(aabb)
    }

    #[cfg(feature = "lumi-core")]
    pub fn draw_command(&self) -> lumi_core::DrawCommand {
        if let Some(indices) = self.indices() {
            lumi_core::DrawCommand::Indexed {
                indices: 0..indices.len() as u32,
                base_vertex: 0,
                instances: 0..1,
            }
        } else {
            let len = self.positions().map_or(0, |positions| positions.len());

            lumi_core::DrawCommand::Vertex {
                vertices: 0..len as u32,
                instances: 0..1,
            }
        }
    }

    #[cfg(feature = "lumi-core")]
    pub fn draw_command_instanced(
        &self,
        instances: std::ops::Range<u32>,
    ) -> lumi_core::DrawCommand {
        if let Some(indices) = self.indices() {
            lumi_core::DrawCommand::Indexed {
                indices: 0..indices.len() as u32,
                base_vertex: 0,
                instances,
            }
        } else {
            let len = self.positions().map_or(0, |positions| positions.len());

            lumi_core::DrawCommand::Vertex {
                vertices: 0..len as u32,
                instances,
            }
        }
    }

    pub fn transform(&mut self, transform: Mat4) {
        if let Some(positions) = self.positions_mut() {
            for position in positions {
                *position = transform.transform_point3(*position);
            }
        }

        if let Some(normals) = self.normals_mut() {
            for normal in normals {
                *normal = transform.transform_vector3(*normal).normalize();
            }
        }
    }
}

impl Mesh {
    pub fn insert_positions(&mut self, positions: impl Into<Vec<Vec3>>) {
        self.insert_attribute(Self::POSITION, positions.into());
    }

    pub fn remove_positions(&mut self) -> Option<Vec<Vec3>> {
        self.remove_attribute(Self::POSITION)
    }

    pub fn positions(&self) -> Option<&[Vec3]> {
        self.attribute(Self::POSITION)
    }

    pub fn positions_mut(&mut self) -> Option<&mut [Vec3]> {
        self.attribute_mut(Self::POSITION)
    }
}

impl Mesh {
    pub fn insert_normals(&mut self, normals: impl Into<Vec<Vec3>>) {
        self.insert_attribute(Self::NORMAL, normals.into());
    }

    pub fn remove_normals(&mut self) -> Option<Vec<Vec3>> {
        self.remove_attribute(Self::NORMAL)
    }

    pub fn normals(&self) -> Option<&[Vec3]> {
        self.attribute(Self::NORMAL)
    }

    pub fn normals_mut(&mut self) -> Option<&mut [Vec3]> {
        self.attribute_mut(Self::NORMAL)
    }
}

impl Mesh {
    pub fn insert_tangents(&mut self, tangents: impl Into<Vec<[f32; 4]>>) {
        self.insert_attribute(Self::TANGENT, tangents.into());
    }

    pub fn remove_tangents(&mut self) -> Option<Vec<[f32; 4]>> {
        self.remove_attribute(Self::TANGENT)
    }

    pub fn tangents(&self) -> Option<&[[f32; 4]]> {
        self.attribute(Self::TANGENT)
    }

    pub fn tangents_mut(&mut self) -> Option<&mut [[f32; 4]]> {
        self.attribute_mut(Self::TANGENT)
    }
}

impl Mesh {
    pub fn insert_uv0(&mut self, uvs: impl Into<Vec<Vec2>>) {
        self.insert_attribute(Self::UV_0, uvs.into());
    }

    pub fn remove_uv0(&mut self) -> Option<Vec<Vec2>> {
        self.remove_attribute(Self::UV_0)
    }

    pub fn uv_0(&self) -> Option<&[Vec2]> {
        self.attribute(Self::UV_0)
    }

    pub fn uv_0_mut(&mut self) -> Option<&mut [Vec2]> {
        self.attribute_mut(Self::UV_0)
    }
}
