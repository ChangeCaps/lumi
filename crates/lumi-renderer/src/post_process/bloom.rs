use lumi_core::{
    CommandEncoder, Device, LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor,
    Resources,
};
use lumi_util::math::Vec3;
use lumi_world::World;

use crate::{
    DownsampleBindings, MipChain, MipChainPipeline, RenderSettings, RenderViewPhase,
    UpsampleBindings, ViewPhaseContext,
};

pub struct BloomState {
    down: MipChain,
    up: MipChain,
}

impl BloomState {
    pub fn new(device: &Device, pipeline: &MipChainPipeline, width: u32, height: u32) -> Self {
        Self {
            down: MipChain::new(device, &pipeline.down_layout, width, height, None),
            up: MipChain::new(device, &pipeline.up_layout, width, height, None),
        }
    }

    pub fn resize(
        &mut self,
        device: &Device,
        pipeline: &MipChainPipeline,
        width: u32,
        height: u32,
    ) {
        if self.down.width() != width || self.down.height() != height {
            self.down = MipChain::new(device, &pipeline.down_layout, width, height, None);
            self.up = MipChain::new(device, &pipeline.up_layout, width, height, None);
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct BloomPhase;

impl RenderViewPhase for BloomPhase {
    fn prepare(&mut self, context: &ViewPhaseContext, _world: &World, resources: &mut Resources) {
        let pipeline = resources.remove::<MipChainPipeline>().unwrap();
        let settings = resources.get::<RenderSettings>().unwrap().clone();

        let state = if let Some(state) = resources.get_id_mut::<BloomState>(context.view.camera) {
            state
        } else {
            let state = BloomState::new(
                context.device,
                &pipeline,
                context.target.width(),
                context.target.height(),
            );

            resources.insert_id(context.view.camera.cast(), state);

            resources
                .get_id_mut::<BloomState>(context.view.camera)
                .unwrap()
        };

        state.resize(
            context.device,
            &pipeline,
            context.target.width(),
            context.target.height(),
        );

        let scale = state.down.filter_scale() * settings.bloom_scale;
        let curve = Vec3::new(
            settings.bloom_threshold - settings.bloom_knee,
            settings.bloom_knee * 2.0,
            0.25 / settings.bloom_knee,
        );

        state.down.bindings[0].bind(
            context.device,
            context.queue,
            &DownsampleBindings {
                source_texture: &context.target.hdr_view,
                scale,
                threshold: settings.bloom_threshold,
                curve,
            },
        );

        state.down.bindings[0].update_bind_groups(context.device);

        for mip in 1..state.down.mip_levels() as usize {
            state.down.bindings[mip].bind(
                context.device,
                context.queue,
                &DownsampleBindings {
                    source_texture: &state.down.views[mip - 1],
                    scale,
                    threshold: 0.0,
                    curve: Vec3::ZERO,
                },
            );

            state.down.bindings[mip].update_bind_groups(context.device);
        }

        for mip in 0..state.up.mip_levels() as usize {
            let source_mip_texture = if mip == state.up.mip_levels() as usize - 1 {
                &state.down.views[mip]
            } else {
                &state.up.views[mip + 1]
            };

            state.up.bindings[mip].bind(
                context.device,
                context.queue,
                &UpsampleBindings {
                    source_texture: &state.down.views[mip],
                    source_mip_texture,
                    scale,
                },
            );

            state.up.bindings[mip].update_bind_groups(context.device);
        }

        resources.insert(pipeline);
    }

    fn render(
        &self,
        context: &ViewPhaseContext,
        encoder: &mut CommandEncoder,
        _world: &World,
        resources: &Resources,
    ) {
        let pipeline = resources.get::<MipChainPipeline>().unwrap();
        let state = resources.get_id::<BloomState>(context.view.camera).unwrap();

        for mip in 0..state.down.mip_levels() {
            let mut pass = state.down.begin_render_pass(encoder, mip);

            pass.set_pipeline(&pipeline.downsample_pipeline);
            state.down.bindings[mip as usize].apply(&mut pass);
            pass.draw(0..3, 0..1);
        }

        for mip in (1..state.up.mip_levels()).rev() {
            let mut pass = state.up.begin_render_pass(encoder, mip);

            pass.set_pipeline(&pipeline.upsample_pipeline);
            state.up.bindings[mip as usize].apply(&mut pass);
            pass.draw(0..3, 0..1);
        }

        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Lumi Bloom upsample pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &context.target.hdr_view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        pass.set_pipeline(&pipeline.upsample_pipeline);
        state.up.bindings[0].apply(&mut pass);
        pass.draw(0..3, 0..1);
    }
}
