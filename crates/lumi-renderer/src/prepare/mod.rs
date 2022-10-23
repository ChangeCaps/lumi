mod lights;
mod meshes;

pub use lights::*;
pub use meshes::*;

use crate::{DefaultPhases, RenderPlugin, RendererBuilder};

#[derive(Clone, Copy, Debug, Default)]
pub struct PreparePlugin;

impl RenderPlugin for PreparePlugin {
    fn build(self, builder: &mut RendererBuilder) {
        builder.add_phase(DefaultPhases::PrepareMeshes, PrepareMeshes);
        builder.add_phase(DefaultPhases::PrepareLights, PrepareLights);
    }
}
