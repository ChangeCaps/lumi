use std::{ops::Deref, sync::Arc};

use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BufferDescriptor, SamplerDescriptor, TextureDescriptor,
};

use crate::{DeviceId, SharedBuffer, SharedQueue, SharedSampler, SharedTexture};

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

    pub fn create_shared_buffer(&self, desc: &BufferDescriptor) -> SharedBuffer {
        let buffer = self.device.create_buffer(desc);
        SharedBuffer::new(buffer, desc.size, desc.usage)
    }

    pub fn create_shared_buffer_init(&self, desc: &BufferInitDescriptor) -> SharedBuffer {
        let buffer = self.device.create_buffer_init(desc);
        SharedBuffer::new(buffer, desc.contents.len() as u64, desc.usage)
    }

    pub fn create_shared_texture(&self, desc: &TextureDescriptor) -> SharedTexture {
        let texture = self.device.create_texture(desc);
        SharedTexture::new(texture, desc)
    }

    pub fn create_shared_texture_with_data(
        &self,
        queue: &SharedQueue,
        desc: &TextureDescriptor,
        data: &[u8],
    ) -> SharedTexture {
        let texture = self.device.create_texture_with_data(queue, desc, data);
        SharedTexture::new(texture, desc)
    }

    pub fn create_shared_sampler(&self, desc: &SamplerDescriptor) -> SharedSampler {
        let sampler = self.device.create_sampler(desc);
        SharedSampler::new(sampler, desc)
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
