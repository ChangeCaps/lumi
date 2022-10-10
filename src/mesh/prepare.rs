use wgpu::Device;

use crate::{
    prelude::Node,
    renderer::{PhaseContext, RenderPhase},
    resources::Resources,
    world::World,
};

use super::{Mesh, MeshBuffers};

#[derive(Clone, Copy)]
pub struct PrepareMeshFn {
    prepare: fn(&Device, &World, &mut Resources),
}

impl PrepareMeshFn {
    pub fn new(prepare: fn(&Device, &World, &mut Resources)) -> Self {
        Self { prepare }
    }

    pub fn new_as_ref<T: Node + AsRef<Mesh>>() -> Self {
        Self {
            prepare: |device, world, resources| {
                for node in world.nodes::<T>() {
                    if resources.contains_key::<MeshBuffers>(&node.as_ref().id()) {
                        continue;
                    }

                    let buffers = MeshBuffers::new(device, node.as_ref());
                    resources.insert_key(node.as_ref().id(), buffers);
                }
            },
        }
    }

    pub fn prepare(&self, device: &Device, world: &World, resources: &mut Resources) {
        (self.prepare)(device, world, resources)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct PrepareMeshPhase;

impl RenderPhase for PrepareMeshPhase {
    fn prepare(&mut self, context: &PhaseContext, world: &World, resources: &mut Resources) {
        let prepare_mesh_fns = resources.remove_keyed::<PrepareMeshFn>();

        for prepare_mesh_fn in prepare_mesh_fns.values() {
            prepare_mesh_fn.prepare(&context.device, world, resources);
        }

        resources.insert(prepare_mesh_fns);
    }
}
