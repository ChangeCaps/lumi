mod draw;
mod material;
mod prepare;
mod primitive;
mod standard;
mod unlit;

pub use draw::*;
use lumi_mesh::Mesh;
pub use material::*;
pub use prepare::*;
pub use primitive::*;
pub use standard::*;
pub use unlit::*;

use lumi_renderer::{
    ExtractStage, GlobalTransform, Renderer, RendererPlugin, Transform, ViewStage, ViewSystem,
};
use shiv::{
    bundle::Bundle,
    schedule::{IntoSystemDescriptor, SystemLabel},
};

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

#[derive(Clone, Debug, Default, Bundle)]
pub struct MaterialBundle<T: Material = StandardMaterial> {
    pub material: T,
    pub mesh: Mesh,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
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
