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

use lumi_renderer::{ExtractStage, Renderer, RendererPlugin, ViewStage, ViewSystem};
use shiv::schedule::{IntoSystemDescriptor, SystemLabel};

use std::marker::PhantomData;

#[derive(SystemLabel)]
pub enum MaterialSystem {
    Extract,
    Prepare,
    Bindings,
    Draw,
}

pub struct ExtractMaterialPlugin<T: ExtractMaterials> {
    _marker: PhantomData<T>,
}

impl<T: ExtractMaterials> Default for ExtractMaterialPlugin<T> {
    #[inline]
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T: ExtractMaterials> RendererPlugin for ExtractMaterialPlugin<T> {
    fn build(&self, renderer: &mut Renderer) {
        renderer.extract.add_system_to_stage(
            ExtractStage::Extract,
            extract_material_system::<T>.label(MaterialSystem::Extract),
        );

        renderer
            .view
            .add_system_to_stage(
                ViewStage::Prepare,
                prepare_material_system::<T>
                    .label(MaterialSystem::Prepare)
                    .after(ViewSystem::ScreenSpaceResize)
                    .after(ViewSystem::PrepareCamera),
            )
            .add_system_to_stage(
                ViewStage::Draw,
                draw_material_system::<T>.label(MaterialSystem::Draw),
            );
    }
}

pub struct MaterialPlugin<T: Material> {
    marker: PhantomData<T>,
}

impl<T: Material> Default for MaterialPlugin<T> {
    #[inline]
    fn default() -> Self {
        Self {
            marker: PhantomData,
        }
    }
}

impl<T: Material> RendererPlugin for MaterialPlugin<T> {
    fn build(&self, renderer: &mut Renderer) {
        renderer.world.init_resource::<PreparedMaterialPipelines>();

        renderer.view.add_system_to_stage(
            ViewStage::Prepare,
            update_bindings_system
                .label(MaterialSystem::Bindings)
                .after(MaterialSystem::Prepare),
        );

        renderer.add_plugin(ExtractMaterialPlugin::<T>::default());
        renderer.add_plugin(ExtractMaterialPlugin::<Primitive<T>>::default());
        renderer.add_plugin(ExtractMaterialPlugin::<Primitives<T>>::default());
    }
}
