use lumi_bind::Bind;
use lumi_core::{CommandEncoder, Device, Extent3d, SharedTextureView};
use shiv::{
    query::Query,
    system::{Commands, Res, ResInit, ResMut},
    world::Component,
};

use crate::{MipChain, MipChainPipeline, RenderDevice, RenderQueue, TransparentDraws, View};

#[derive(Clone, Bind)]
pub struct ScreenSpaceBindings {
    #[texture]
    #[sampler(name = "ssr_sampler")]
    pub ssr_texture: SharedTextureView,
}

#[derive(Component)]
pub struct ScreenSpaceTarget {
    pub mip_chain: MipChain,
}

impl ScreenSpaceTarget {
    pub fn new(device: &Device, pipeline: &MipChainPipeline, size: Extent3d) -> Self {
        let mip_chain = MipChain::new(device, &pipeline.down_layout, size.width, size.height, None);

        Self { mip_chain }
    }

    pub fn bindings(&self) -> ScreenSpaceBindings {
        ScreenSpaceBindings {
            ssr_texture: self.mip_chain.view.clone(),
        }
    }
}

pub fn screen_space_resize_system(
    mut commands: Commands,
    device: Res<RenderDevice>,
    view: Res<View>,
    pipeline: ResInit<MipChainPipeline>,
    mut query: Query<&mut ScreenSpaceTarget>,
) {
    if let Some(mut target) = query.get_mut(view.camera) {
        let size = view.frame_buffer.size();

        if target.mip_chain.size() != size {
            (target.mip_chain).resize(&device, size.width, size.height, None);
        }
    } else {
        let target = ScreenSpaceTarget::new(&device, &pipeline, view.frame_buffer.size());
        commands.get_or_spawn(view.camera).insert(target);
    }
}

pub fn screen_space_render_system(
    mut encoder: ResMut<CommandEncoder>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    pipeline: ResInit<MipChainPipeline>,
    view: Res<View>,
    transparent_draws: Res<TransparentDraws>,
    mut query: Query<&mut ScreenSpaceTarget>,
) {
    if transparent_draws.is_empty() {
        return;
    }

    // ScreenSpaceTarget was inserted in screen_space_resize_system
    let mut target = query.get_mut(view.camera).unwrap();

    target
        .mip_chain
        .prepare_downsample_bindings(&device, &queue, &view.frame_buffer.hdr_view, 4.0);
    target.mip_chain.downsample(&pipeline, &mut encoder);
}
