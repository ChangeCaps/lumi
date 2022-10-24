use std::num::NonZeroU32;

use glam::Vec3;
use wgpu::{
    BlendComponent, BlendFactor, BlendOperation, BlendState, Color, ColorTargetState, ColorWrites,
    CommandEncoder, Extent3d, FragmentState, LoadOp, Operations, RenderPass,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    TextureFormat, TextureUsages, VertexState,
};

use crate::{
    bind::Bind,
    binding::{Bindings, BindingsLayout},
    shader::{ShaderProcessor, ShaderRef},
    Device, Queue, SharedDevice, SharedTexture, SharedTextureView,
};

pub struct Bloom {
    down: MipChain,
    up: MipChain,
}

impl Bloom {
    pub fn new(device: &Device, pipeline: &BloomPipeline, width: u32, height: u32) -> Self {
        let down = MipChain::new(device, &pipeline.down_layout, width, height, None);
        let up = MipChain::new(device, &pipeline.up_layout, width, height, None);

        Self { down, up }
    }

    pub fn resize(&mut self, device: &Device, pipeline: &BloomPipeline, width: u32, height: u32) {
        if self.down.width() != width || self.down.height() != height {
            self.down = MipChain::new(device, &pipeline.down_layout, width, height, None);
            self.up = MipChain::new(device, &pipeline.up_layout, width, height, None);
        }
    }

    pub fn render(
        &mut self,
        device: &Device,
        queue: &Queue,
        pipeline: &BloomPipeline,
        encoder: &mut CommandEncoder,
        source: &SharedTextureView,
        threshold: f32,
        knee: f32,
        scale: f32,
    ) {
        let scale = self.down.filter_scale() * scale;

        let curve = Vec3::new(threshold - knee, knee * 2.0, 0.25 / knee);

        self.down.bindings[0].bind(
            device,
            queue,
            &DownsampleBindings {
                source_texture: source,
                scale,
                threshold,
                curve,
            },
        );

        self.down.bindings[0].update_bind_groups(device);

        for mip in 1..self.down.mip_levels() as usize {
            self.down.bindings[mip].bind(
                device,
                queue,
                &DownsampleBindings {
                    source_texture: &self.down.views[mip - 1],
                    scale,
                    threshold: 0.0,
                    curve: Vec3::ZERO,
                },
            );

            self.down.bindings[mip].update_bind_groups(device);
        }

        for mip in 0..self.up.mip_levels() as usize {
            let source_mip_texture = if mip == self.up.mip_levels() as usize - 1 {
                &self.down.views[mip]
            } else {
                &self.up.views[mip + 1]
            };

            self.up.bindings[mip].bind(
                device,
                queue,
                &UpsampleBindings {
                    source_texture: &self.down.views[mip],
                    source_mip_texture,
                    scale,
                },
            );

            self.up.bindings[mip].update_bind_groups(device);
        }

        for mip in 0..self.down.mip_levels() {
            let mut pass = self.down.begin_render_pass(encoder, mip);

            pass.set_pipeline(&pipeline.downsample_pipeline);
            self.down.bindings[mip as usize].apply(&mut pass);
            pass.draw(0..3, 0..1);
        }

        for mip in (1..self.up.mip_levels()).rev() {
            let mut pass = self.up.begin_render_pass(encoder, mip);

            pass.set_pipeline(&pipeline.upsample_pipeline);
            self.up.bindings[mip as usize].apply(&mut pass);
            pass.draw(0..3, 0..1);
        }

        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Lumi Bloom upsample pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: source,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        pass.set_pipeline(&pipeline.upsample_pipeline);
        self.up.bindings[0].apply(&mut pass);
        pass.draw(0..3, 0..1);
    }
}
