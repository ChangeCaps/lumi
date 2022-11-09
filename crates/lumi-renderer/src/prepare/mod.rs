mod camera;
mod environment;
mod light;
mod mesh;
mod shadow;
mod transform;

pub use camera::*;
pub use environment::*;
pub use light::*;
pub use mesh::*;
pub use shadow::*;
pub use transform::*;

use lumi_mesh::Mesh;
use shiv::schedule::{IntoSystemDescriptor, SystemLabel};

use crate::{ExtractStage, Renderer, RendererPlugin};

#[derive(SystemLabel)]
pub enum ExtractSystem {
    Transform,
    Light,
    Mesh,
    Camera,
    Environment,
    Shadow,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct CoreExtractPlugin;

impl RendererPlugin for CoreExtractPlugin {
    fn build(&self, renderer: &mut Renderer) {
        renderer.world.init_resource::<PreparedLights>();
        renderer.world.init_resource::<ShadowTargets>();

        renderer
            .extract
            .add_system_to_stage(
                ExtractStage::PreExtract,
                insert_state_system.label(ExtractSystem::Shadow),
            )
            .add_system_to_stage(
                ExtractStage::PreExtract,
                clear_extracted_meshes_system.label(ExtractSystem::Mesh),
            )
            .add_system_to_stage(
                ExtractStage::Extract,
                extract_transform_system.label(ExtractSystem::Transform),
            )
            .add_system_to_stage(
                ExtractStage::Extract,
                extract_light_system.label(ExtractSystem::Light),
            )
            .add_system_to_stage(
                ExtractStage::Extract,
                extract_directional_shadow_system
                    .label(ExtractSystem::Shadow)
                    .after(ExtractSystem::Light),
            )
            .add_system_to_stage(
                ExtractStage::Extract,
                extract_camera_system.label(ExtractSystem::Camera),
            )
            .add_system_to_stage(
                ExtractStage::Extract,
                extract_environment_system.label(ExtractSystem::Environment),
            )
            .add_system_to_stage(
                ExtractStage::Extract,
                extract_mesh_system.label(ExtractSystem::Mesh),
            )
            .add_system_to_stage(
                ExtractStage::PostExtract,
                render_shadow_system.label(ExtractSystem::Shadow),
            );

        renderer.add_plugin(ExtractMeshPlugin::<Mesh>::default());
    }
}
