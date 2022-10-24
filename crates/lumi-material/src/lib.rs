mod draw;
mod material;
mod mesh_node;
mod prepare;
mod render;
mod standard;

pub use draw::*;
use lumi_core::Device;
pub use material::*;
pub use mesh_node::*;
pub use prepare::*;
pub use render::*;
pub use standard::*;

use lumi_renderer::{
    CorePhase, MipChainPipeline, PhaseLabel, RenderPlugin, Renderer, RendererBuilder,
};

#[derive(PhaseLabel)]
pub enum MaterialPhase {
    Render,
}

#[derive(Default)]
pub struct MaterialPlugin;

impl RenderPlugin for MaterialPlugin {
    fn build(&self, builder: &mut RendererBuilder) {
        builder.add_view_phase_after(
            CorePhase::Render,
            MaterialPhase::Render,
            RenderMaterials::default(),
        );
    }

    fn init(&self, renderer: &mut Renderer, device: &Device) {
        MipChainPipeline::init(renderer, device);
    }
}
