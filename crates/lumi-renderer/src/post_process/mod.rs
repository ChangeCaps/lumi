mod bloom;

pub use bloom::*;
use lumi_core::Device;

use crate::{CorePhase, MipChainPipeline, PhaseLabel, RenderPlugin, Renderer, RendererBuilder};

#[derive(Clone, Copy, Debug, PhaseLabel)]
pub enum PostProcessPhase {
    Bloom,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PostProcessPlugin;

impl RenderPlugin for PostProcessPlugin {
    fn build(&self, builder: &mut RendererBuilder) {
        builder.add_view_phase_after(CorePhase::PostProcess, PostProcessPhase::Bloom, BloomPhase);
    }

    fn init(&self, renderer: &mut Renderer, device: &Device) {
        MipChainPipeline::init(renderer, device);
    }
}
