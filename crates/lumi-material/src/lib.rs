mod material;
mod prepare;

pub use material::*;
pub use prepare::*;

use lumi_renderer::{RenderPlugin, RendererBuilder};

#[derive(Default)]
pub struct MaterialPlugin;

impl RenderPlugin for MaterialPlugin {
    fn build(self, builder: &mut RendererBuilder) {}
}
