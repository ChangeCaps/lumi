use std::{
    iter,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use deref_derive::{Deref, DerefMut};
use lumi_bounds::Aabb;
use lumi_core::{
    BufferInitDescriptor, BufferUsages, Device, DrawCommand, SharedBuffer, SharedDevice,
};
use lumi_id::IdMap;
use lumi_mesh::{Mesh, MeshId};

use lumi_util::HashMap;
use shiv::{
    query::{Changed, Query, Without},
    schedule::IntoSystemDescriptor,
    system::{Commands, Res, ResMut},
    world::{Component, Entity},
};

use crate::{Extract, ExtractStage, ExtractSystem, RenderDevice, Renderer, RendererPlugin};

#[derive(Component, Clone, Debug, Default, Deref, DerefMut)]
pub struct ExtractedMeshes {
    pub(crate) meshes: Vec<MeshId>,
}

pub struct PreparedMesh {
    pub attributes: HashMap<String, SharedBuffer>,
    pub indices: Option<SharedBuffer>,
    pub aabb: Option<Aabb>,
    pub draw: DrawCommand,
}

#[derive(Default)]
pub struct PreparedMeshes {
    pub meshes: IdMap<Mesh, PreparedMesh>,
}

impl Deref for PreparedMeshes {
    type Target = IdMap<Mesh, PreparedMesh>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.meshes
    }
}

impl DerefMut for PreparedMeshes {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.meshes
    }
}

impl PreparedMeshes {
    #[inline]
    pub fn prepare(&mut self, device: &Device, mesh: &Mesh) {
        let mesh_id = mesh.id();

        let mesh = mesh.clone().with_normals().with_tangents();
        let mut prepared_mesh = PreparedMesh {
            attributes: HashMap::default(),
            indices: None,
            aabb: None,
            draw: mesh.draw_command(),
        };

        for (name, attribute) in mesh.attributes() {
            let buffer = device.create_shared_buffer_init(&BufferInitDescriptor {
                label: Some(&format!("mesh-{}-attribute{}", mesh_id, name)),
                contents: attribute.as_bytes(),
                usage: BufferUsages::VERTEX,
            });

            prepared_mesh.attributes.insert(name.to_string(), buffer);
        }

        if let Some(indices) = mesh.indices_as_bytes() {
            let buffer = device.create_shared_buffer_init(&BufferInitDescriptor {
                label: Some(&format!("mesh-{}-indices", mesh_id)),
                contents: indices,
                usage: BufferUsages::INDEX,
            });

            prepared_mesh.indices = Some(buffer);
        }

        prepared_mesh.aabb = mesh.aabb();

        self.meshes.insert(mesh_id, prepared_mesh);
    }
}

pub trait ExtractMeshes: Component {
    type Iter<'a>: Iterator<Item = &'a Mesh>;

    fn extract_meshes(&self) -> Self::Iter<'_>;
}

impl ExtractMeshes for Mesh {
    type Iter<'a> = iter::Once<&'a Mesh>;

    #[inline]
    fn extract_meshes(&self) -> Self::Iter<'_> {
        iter::once(self)
    }
}

pub fn extract_mesh_system(
    mut commands: Commands,
    query: Extract<Query<(Entity, &Mesh), Changed<Mesh>>>,
) {
    for (entity, mesh) in query.iter() {
        commands.entity(entity).insert(mesh.clone());
    }
}

pub fn clear_extracted_meshes_system(
    mut commands: Commands,
    mut query: Query<&mut ExtractedMeshes>,
    spawn_query: Query<Entity, Without<ExtractedMeshes>>,
) {
    for mut extracted_meshes in query.iter_mut() {
        extracted_meshes.clear();
    }

    for entity in spawn_query.iter() {
        commands.entity(entity).insert(ExtractedMeshes::default());
    }
}

pub fn extract_meshes_system<T: ExtractMeshes>(
    device: Res<RenderDevice>,
    mut prepared_meshes: ResMut<PreparedMeshes>,
    mesh_query: Extract<Query<(Entity, &T)>>,
    mut extracted_query: Query<&mut ExtractedMeshes>,
) {
    for (entity, extract_meshes) in mesh_query.iter() {
        let mut extracted = extracted_query.get_mut(entity).unwrap();

        for mesh in extract_meshes.extract_meshes() {
            extracted.push(mesh.id());

            if !prepared_meshes.contains_id(mesh.id()) {
                prepared_meshes.prepare(&device, mesh);
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ExtractMeshPlugin<T> {
    _marker: PhantomData<T>,
}

impl<T: ExtractMeshes> RendererPlugin for ExtractMeshPlugin<T> {
    #[inline]
    fn build(&self, renderer: &mut Renderer) {
        renderer.extract.add_system_to_stage(
            ExtractStage::Extract,
            extract_meshes_system::<T>.label(ExtractSystem::Mesh),
        );

        renderer.world.init_resource::<PreparedMeshes>();
    }
}
