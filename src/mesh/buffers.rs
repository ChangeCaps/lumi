use wgpu::{util::BufferInitDescriptor, BufferUsages, Device};

use crate::{util::HashMap, SharedBuffer, SharedDevice};

use super::Mesh;

#[derive(Clone, Debug)]
pub struct MeshBuffers {
    pub attributes: HashMap<String, SharedBuffer>,
    pub index_buffer: Option<SharedBuffer>,
}

impl MeshBuffers {
    pub fn new(device: &Device, mesh: &Mesh) -> Self {
        let mut attributes = HashMap::default();
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
