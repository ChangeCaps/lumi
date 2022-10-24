use lumi_core::Device;

use crate::{CorePhase, EmptyPhase, Renderer, RendererBuilder};

#[allow(unused_variables)]
pub trait RenderPlugin {
    fn build(&self, builder: &mut RendererBuilder) {}
    fn init(&self, renderer: &mut Renderer, device: &Device) {}
}

#[derive(Clone, Copy, Debug, Default)]
pub struct CorePlugin;

impl RenderPlugin for CorePlugin {
    fn build(&self, builder: &mut RendererBuilder) {
        builder.add_phase(CorePhase::Prepare, EmptyPhase);
        builder.add_view_phase(CorePhase::Clear, EmptyPhase);
        builder.add_view_phase(CorePhase::Render, EmptyPhase);
    }
}
