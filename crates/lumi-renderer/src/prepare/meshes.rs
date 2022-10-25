use lumi_core::{BufferInitDescriptor, BufferUsages, Resources, SharedBuffer, SharedDevice};
use lumi_id::IdMap;
use lumi_mesh::Mesh;
use lumi_util::HashMap;
use lumi_world::{Extract, Node, World};

use crate::{PhaseContext, RenderPhase};

#[derive(Default)]
pub struct PreparedMesh {
    pub attributes: HashMap<String, SharedBuffer>,
    pub indices: Option<SharedBuffer>,
}

pub struct PrepareMeshFunction {
    prepare: fn(&PhaseContext, &World, &mut Resources),
}

impl PrepareMeshFunction {
    #[inline]
    pub fn new<T: Node + Extract<Mesh>>() -> Self {
        Self {
            prepare: Self::prepare::<T>,
        }
    }

    fn prepare<T>(context: &PhaseContext, world: &World, resources: &mut Resources)
    where
        T: Node + Extract<Mesh>,
    {
        for node in world.nodes::<T>() {
            node.extract(&mut |mesh| {
                let id = mesh.id();

                if resources.contains_id::<PreparedMesh>(id) {
                    return;
                }

                let mesh = mesh.clone().with_normals().with_tangents();
                let mut prepared_mesh = PreparedMesh::default();

                for (name, attribute) in mesh.attributes() {
                    let buffer = context
                        .device
                        .create_shared_buffer_init(&BufferInitDescriptor {
                            label: Some(&format!("mesh-{}-attribute-{}", id, name)),
                            contents: attribute.as_bytes(),
                            usage: BufferUsages::VERTEX,
                        });

                    prepared_mesh.attributes.insert(name.to_string(), buffer);
                }

                if let Some(indices) = mesh.indices_as_bytes() {
                    let buffer = context
                        .device
                        .create_shared_buffer_init(&BufferInitDescriptor {
                            label: Some(&format!("mesh-{}-indices", id)),
                            contents: indices,
                            usage: BufferUsages::INDEX,
                        });

                    prepared_mesh.indices = Some(buffer);
                }

                if let Some(aabb) = mesh.aabb() {
                    resources.insert_id(id.cast(), aabb);
                }

                resources.insert_id(id.cast(), prepared_mesh);
            });
        }
    }

    #[inline]
    pub fn prepare_meshes(&self, context: &PhaseContext, world: &World, resources: &mut Resources) {
        (self.prepare)(context, world, resources);
    }
}

#[derive(Clone, Copy, Default)]
pub struct PrepareMeshes;

impl RenderPhase for PrepareMeshes {
    #[inline]
    fn prepare(&mut self, context: &PhaseContext, world: &World, resources: &mut Resources) {
        resources.register_id::<PrepareMeshFunction>();

        resources.scope(
            |resources: &mut Resources, prepare_meshes: &mut IdMap<PrepareMeshFunction>| {
                for prepare_mesh in prepare_meshes.values() {
                    prepare_mesh.prepare_meshes(context, world, resources);
                }
            },
        );
    }
}
