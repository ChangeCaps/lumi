mod data;
mod loader;

pub use data::*;
pub use loader::*;

use lumi_renderer::{RenderPlugin, RendererBuilder};

#[derive(Clone, Copy, Debug, Default)]
pub struct GltfPlugin;

impl RenderPlugin for GltfPlugin {
    fn build(&self, builder: &mut RendererBuilder) {
        builder.add_asset_loader(GltfLoader);
    }
}
