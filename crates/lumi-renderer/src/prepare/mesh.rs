use std::{iter, marker::PhantomData};

use lumi_bounds::Aabb;
use lumi_core::{
    BufferInitDescriptor, BufferUsages, Device, DrawCommand, SharedBuffer, SharedDevice,
};
use lumi_id::IdMap;
use lumi_mesh::{Mesh, MeshId};
use lumi_util::HashMap;

use deref_derive::{Deref, DerefMut};
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
    pub references: usize,
}

#[derive(Default, Deref, DerefMut)]
pub struct PreparedMeshes {
    #[deref]
    pub meshes: IdMap<Mesh, PreparedMesh>,
    pub has_changed: bool,
}

impl PreparedMeshes {
    #[inline]
    pub fn prepare(&mut self, device: &Device, mesh: &Mesh) {
        let mesh_id = mesh.id();

        if let Some(prepared_mesh) = self.meshes.get_mut(mesh_id) {
            prepared_mesh.references += 1;
            return;
        }

        let mesh = mesh.clone().with_normals().with_tangents();
        let mut prepared_mesh = PreparedMesh {
            attributes: HashMap::default(),
            indices: None,
            aabb: None,
            draw: mesh.draw_command(),
            references: 1,
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
        self.has_changed = true;
    }

    #[inline]
    pub fn remove_mesh(&mut self, mesh_id: MeshId) {
        if let Some(prepared_mesh) = self.meshes.get_mut(mesh_id) {
            prepared_mesh.references -= 1;

            self.has_changed = true;
        }
    }

    #[inline]
    pub fn remove_unused(&mut self) {
        if self.has_changed {
            (self.meshes).retain(|_, prepared_mesh| prepared_mesh.references > 0);
            self.has_changed = false;
        }
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

pub fn maintain_extracted_meshes_system<T: ExtractMeshes>(
    mut commands: Commands,
    mut prepared_meshes: ResMut<PreparedMeshes>,
    without_query: Extract<Query<Entity, Without<T>>>,
    changed_query: Extract<Query<(), Changed<T>>>,
    mut mesh_query: Query<(Entity, &mut ExtractedMeshes)>,
) {
    for entity in without_query.iter() {
        if !mesh_query.contains(entity) {
            commands.entity(entity).insert(ExtractedMeshes::default());
        }
    }

    for (entity, mut meshes) in mesh_query.iter_mut() {
        if !changed_query.contains(entity) {
            continue;
        }

        for mesh in meshes.drain(..) {
            prepared_meshes.remove_mesh(mesh);
        }
    }
}

pub fn extract_meshes_system<T: ExtractMeshes>(
    device: Res<RenderDevice>,
    mut prepared_meshes: ResMut<PreparedMeshes>,
    mesh_query: Extract<Query<(Entity, &T), Changed<T>>>,
    mut extracted_query: Query<&mut ExtractedMeshes>,
) {
    for (entity, extract_meshes) in mesh_query.iter() {
        let mut extracted = extracted_query.get_mut(entity).unwrap();

        for mesh in extract_meshes.extract_meshes() {
            extracted.push(mesh.id());
            prepared_meshes.prepare(&device, mesh);
        }
    }

    prepared_meshes.remove_unused();
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ExtractMeshPlugin<T> {
    _marker: PhantomData<T>,
}

impl<T: ExtractMeshes> RendererPlugin for ExtractMeshPlugin<T> {
    #[inline]
    fn build(&self, renderer: &mut Renderer) {
        renderer.extract.add_system_to_stage(
            ExtractStage::PreExtract,
            maintain_extracted_meshes_system::<T>.label(ExtractSystem::Mesh),
        );
        renderer.extract.add_system_to_stage(
            ExtractStage::Extract,
            extract_meshes_system::<T>.label(ExtractSystem::Mesh),
        );

        renderer.world.init_resource::<PreparedMeshes>();
    }
}
