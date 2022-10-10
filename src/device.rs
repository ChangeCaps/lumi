use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroupDescriptor, BufferDescriptor, Device, Queue, RenderPipelineDescriptor,
    SamplerDescriptor, TextureDescriptor,
};

use crate::{SharedBindGroup, SharedBuffer, SharedRenderPipeline, SharedSampler, SharedTexture};

pub trait SharedDevice {
    fn create_shared_buffer(&self, desc: &BufferDescriptor) -> SharedBuffer;
    fn create_shared_buffer_init(&self, desc: &BufferInitDescriptor) -> SharedBuffer;
    fn create_shared_texture(&self, desc: &TextureDescriptor) -> SharedTexture;
    fn create_shared_texture_with_data(
        &self,
        queue: &Queue,
        desc: &TextureDescriptor,
        data: &[u8],
    ) -> SharedTexture;
    fn create_shared_sampler(&self, desc: &SamplerDescriptor) -> SharedSampler;
    fn create_shared_bind_group(&self, desc: &BindGroupDescriptor) -> SharedBindGroup;
    fn create_shared_render_pipeline(
        &self,
        desc: &RenderPipelineDescriptor,
    ) -> SharedRenderPipeline;
}

impl SharedDevice for Device {
    fn create_shared_buffer(&self, desc: &BufferDescriptor) -> SharedBuffer {
        let buffer = self.create_buffer(desc);
        SharedBuffer::new(buffer, desc.size, desc.usage)
    }

    fn create_shared_buffer_init(&self, desc: &BufferInitDescriptor) -> SharedBuffer {
        let buffer = self.create_buffer_init(desc);
        SharedBuffer::new(buffer, desc.contents.len() as u64, desc.usage)
    }

    fn create_shared_texture(&self, desc: &TextureDescriptor) -> SharedTexture {
        let texture = self.create_texture(desc);
        SharedTexture::new(texture, desc)
    }

    fn create_shared_texture_with_data(
        &self,
        queue: &Queue,
        desc: &TextureDescriptor,
        data: &[u8],
    ) -> SharedTexture {
        let texture = self.create_texture_with_data(queue, desc, data);
        SharedTexture::new(texture, desc)
    }

    fn create_shared_sampler(&self, desc: &SamplerDescriptor) -> SharedSampler {
        let sampler = self.create_sampler(desc);
        SharedSampler::new(sampler, desc)
    }

    fn create_shared_bind_group(&self, desc: &BindGroupDescriptor) -> SharedBindGroup {
        let bind_group = self.create_bind_group(desc);
        SharedBindGroup::new(bind_group)
    }

    fn create_shared_render_pipeline(
        &self,
        desc: &RenderPipelineDescriptor,
    ) -> SharedRenderPipeline {
        let render_pipeline = self.create_render_pipeline(desc);
        SharedRenderPipeline::new(render_pipeline)
    }
}
