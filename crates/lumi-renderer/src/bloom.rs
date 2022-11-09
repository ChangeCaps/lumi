use std::ops::Deref;

use lumi_core::{
    CommandEncoder, Device, LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor,
};
use lumi_util::{math::Vec3, HashMap};
use shiv::{
    system::{Commands, Local, Res, ResInit, ResMut},
    world::Entity,
};

use crate::{
    DownsampleBindings, Extract, MipChain, MipChainPipeline, RenderDevice, RenderQueue,
    UpsampleBindings, View,
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

#[derive(Clone, Debug)]
pub struct BloomSettings {
    pub enabled: bool,
    pub threshold: f32,
    pub knee: f32,
    pub scale: f32,
}

impl Default for BloomSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            threshold: 3.5,
            knee: 1.0,
            scale: 1.0,
        }
    }
}

pub fn extract_bloom_settings_system(
    mut commands: Commands,
    settings: Extract<Option<Res<BloomSettings>>>,
) {
    if let Some(settings) = settings.deref() {
        if settings.is_changed() {
            commands.insert_resource(settings.as_ref().clone());
        }
    }
}

pub fn render_bloom_system(
    mut state: Local<HashMap<Entity, BloomState>>,
    mut encoder: ResMut<CommandEncoder>,
    view: Res<View>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    pipeline: ResInit<MipChainPipeline>,
    settings: Option<Res<BloomSettings>>,
) {
    let state = state.entry(view.camera).or_insert_with(|| {
        BloomState::new(
            &device,
            &pipeline,
            view.frame_buffer.width(),
            view.frame_buffer.height(),
        )
    });

    state.resize(
        &device,
        &pipeline,
        view.frame_buffer.width(),
        view.frame_buffer.height(),
    );

    let settings = settings.as_deref().cloned().unwrap_or_default();

    let scale = state.down.filter_scale() * settings.scale;
    let curve = Vec3::new(
        settings.threshold - settings.knee,
        settings.knee * 2.0,
        0.25 / settings.knee,
    );

    state.down.bindings[0].bind(
        &device,
        &queue,
        &DownsampleBindings {
            source_texture: &view.frame_buffer.hdr_view,
            scale,
            threshold: settings.threshold,
            curve,
        },
    );

    state.down.bindings[0].update_bind_groups(&device);

    for mip in 1..state.down.mip_levels() as usize {
        state.down.bindings[mip].bind(
            &device,
            &queue,
            &DownsampleBindings {
                source_texture: &state.down.views[mip - 1],
                scale,
                threshold: 0.0,
                curve: Vec3::ZERO,
            },
        );

        state.down.bindings[mip].update_bind_groups(&device);
    }

    for mip in 0..state.up.mip_levels() as usize {
        let source_mip_texture = if mip == state.up.mip_levels() as usize - 1 {
            &state.down.views[mip]
        } else {
            &state.up.views[mip + 1]
        };

        state.up.bindings[mip].bind(
            &device,
            &queue,
            &UpsampleBindings {
                source_texture: &state.down.views[mip],
                source_mip_texture,
                scale,
            },
        );

        state.up.bindings[mip].update_bind_groups(&device);
    }

    for mip in 0..state.down.mip_levels() {
        let mut pass = state.down.begin_render_pass(&mut encoder, mip);

        pass.set_pipeline(&pipeline.downsample_pipeline);
        state.down.bindings[mip as usize].apply(&mut pass);
        pass.draw(0..3, 0..1);
    }

    for mip in (1..state.up.mip_levels()).rev() {
        let mut pass = state.up.begin_render_pass(&mut encoder, mip);

        pass.set_pipeline(&pipeline.upsample_pipeline);
        state.up.bindings[mip as usize].apply(&mut pass);
        pass.draw(0..3, 0..1);
    }

    let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
        label: Some("Lumi Bloom upsample pass"),
        color_attachments: &[Some(RenderPassColorAttachment {
            view: &view.frame_buffer.hdr_view,
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
