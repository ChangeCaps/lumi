mod draw;
mod material;
mod prepare;
mod primitive;
mod standard;
mod unlit;

pub use draw::*;
pub use material::*;
pub use prepare::*;
pub use primitive::*;
pub use standard::*;
pub use unlit::*;

use lumi_renderer::{ExtractStage, RenderStage, RenderSystem, Renderer, RendererPlugin};
use shiv::schedule::{IntoSystemDescriptor, SystemLabel};

use std::marker::PhantomData;

#[derive(SystemLabel)]
pub enum MaterialSystem {
    Extract,
    Prepare,
    Bindings,
    Draw,
}

pub struct MaterialPlugin<T: Material> {
    marker: PhantomData<T>,
}

impl<T: Material> Default for MaterialPlugin<T> {
    fn default() -> Self {
        Self {
            marker: PhantomData,
        }
    }
}

impl<T: Material> RendererPlugin for MaterialPlugin<T> {
    fn build(&self, renderer: &mut Renderer) {
        renderer.world.init_resource::<PreparedMaterialPipelines>();

        renderer.extract.add_system_to_stage(
            ExtractStage::Extract,
            extract_material_system::<T>.label(MaterialSystem::Extract),
        );

        renderer
            .render
            .add_system_to_stage(
                RenderStage::Prepare,
                prepare_material_system::<T>
                    .label(MaterialSystem::Prepare)
                    .after(RenderSystem::ScreenSpaceResize)
                    .after(RenderSystem::PrepareCamera),
            )
            .add_system_to_stage(
                RenderStage::Prepare,
                update_bindings_system
                    .label(MaterialSystem::Bindings)
                    .after(MaterialSystem::Prepare),
            )
            .add_system_to_stage(
                RenderStage::Draw,
                draw_material_system::<T>.label(MaterialSystem::Draw),
            );
    }
}
