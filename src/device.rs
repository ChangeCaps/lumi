use std::{ops::Deref, sync::Arc};

use wgpu::BufferUsages;

use crate::{DeviceId, SharedBuffer, SharedQueue, VecBuffer};

#[derive(Clone, Debug)]
pub struct SharedDevice {
    device: Arc<wgpu::Device>,
    id: DeviceId,
}

impl SharedDevice {
    pub fn new(device: wgpu::Device) -> Self {
        Self {
            device: Arc::new(device),
            id: DeviceId::new(),
        }
    }

    pub fn create_shared_buffer(&self, desc: &wgpu::BufferDescriptor) -> SharedBuffer {
        let buffer = self.device.create_buffer(desc);
        SharedBuffer::new(buffer)
    }

    pub fn create_vec_buffer(
        &self,
        queue: &SharedQueue,
        size: usize,
        usage: BufferUsages,
    ) -> VecBuffer {
        VecBuffer::new(self, queue, size, usage)
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn id(&self) -> DeviceId {
        self.id
    }
}

impl Deref for SharedDevice {
    type Target = wgpu::Device;

    fn deref(&self) -> &Self::Target {
        self.device()
    }
}
