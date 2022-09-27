use std::collections::HashMap;

use wgpu::{util::BufferInitDescriptor, BufferUsages};

use crate::{id::MeshId, SharedBuffer, SharedDevice};

use super::Mesh;

#[derive(Clone, Debug)]
pub struct MeshBuffers {
    pub attributes: HashMap<String, SharedBuffer>,
    pub index_buffer: Option<SharedBuffer>,
}

impl MeshBuffers {
    pub fn new(device: &SharedDevice, mesh: &Mesh) -> Self {
        let mut attributes = HashMap::new();
        for (name, attribute) in &mesh.attributes {
            let buffer = device.create_shared_buffer_init(&BufferInitDescriptor {
                label: Some(&format!("{} attribute buffer", name)),
                contents: bytemuck::cast_slice(attribute.data()),
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            });
            attributes.insert(name.clone(), buffer);
        }

        let index_buffer = mesh.indices.as_ref().map(|indices| {
            device.create_shared_buffer_init(&BufferInitDescriptor {
                label: Some("index buffer"),
                contents: bytemuck::cast_slice(indices),
                usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            })
        });

        Self {
            attributes,
            index_buffer,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct MeshBufferCache {
    buffers: HashMap<MeshId, MeshBuffers>,
}

impl MeshBufferCache {
    pub fn contains(&self, mesh: &Mesh) -> bool {
        self.buffers.contains_key(&mesh.id())
    }

    pub fn prepare(&mut self, device: &SharedDevice, mesh: &Mesh) {
        if !self.buffers.contains_key(&mesh.id()) {
            self.buffers
                .insert(mesh.id(), MeshBuffers::new(device, mesh));
        }
    }

    pub fn insert(&mut self, mesh: &Mesh, buffers: MeshBuffers) {
        self.buffers.insert(mesh.id(), buffers);
    }

    pub fn get(&self, mesh: &Mesh) -> Option<&MeshBuffers> {
        self.buffers.get(&mesh.id())
    }
}
