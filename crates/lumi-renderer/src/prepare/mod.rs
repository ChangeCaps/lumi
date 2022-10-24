mod environment;
mod lights;
mod meshes;
mod shadow;
mod transform;

pub use environment::*;
pub use lights::*;
pub use meshes::*;
pub use shadow::*;
pub use transform::*;

use lumi_macro::PhaseLabel;

use crate::{CorePhase, RenderPlugin, RendererBuilder};

#[derive(Clone, Copy, Debug, PhaseLabel)]
pub enum PreparePhase {
    Environment,
    Lights,
    Meshes,
    Transforms,
    Shadow,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PreparePlugin;

impl RenderPlugin for PreparePlugin {
    fn build(&self, builder: &mut RendererBuilder) {
        builder.add_phase_after(
            CorePhase::Prepare,
            PreparePhase::Environment,
            PrepareEnvironment,
        );
        builder.add_phase_after(
            PreparePhase::Environment,
            PreparePhase::Lights,
            PrepareLights,
        );
        builder.add_phase_after(PreparePhase::Lights, PreparePhase::Meshes, PrepareMeshes);
        builder.add_phase_after(
            PreparePhase::Meshes,
            PreparePhase::Transforms,
            PrepareTransforms,
        );
        builder.add_phase_after(
            PreparePhase::Transforms,
            PreparePhase::Shadow,
            PrepareShadows,
        );
    }
}
