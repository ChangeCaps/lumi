use std::sync::Arc;

use lumi_shader::{FileShaderIo, ShaderProcessor};
use shiv::schedule::{DefaultStage, IntoSystemDescriptor, StageLabel, SystemLabel, SystemStage};

use crate::{
    clear_draws_system, draw_system, extract_bloom_settings_system, prepare_camera_system,
    render_bloom_system, render_opaque_system, render_transparent_system,
    screen_space_render_system, screen_space_resize_system, sky_render_system, tone_mapping_system,
    DrawKeys, Extracted, IntegratedBrdf, OpaqueDraws, Renderer, TransparentDraws,
};

pub trait RendererPlugin {
    fn build(&self, renderer: &mut Renderer);
}

#[derive(StageLabel)]
pub enum ExtractStage {
    PreExtract,
    Extract,
    PostExtract,
    Prepare,
}

#[derive(StageLabel)]
pub enum ViewStage {
    /// Before preparation.
    PrePrepare,
    /// Prepare for rendering.
    Prepare,
    /// When preparation is done.
    PostPrepare,
    /// Queue [`Draw`]s for rendering.
    Draw,
    /// Clear the screen.
    Clear,
    /// Before rendering.
    PreRender,
    /// Prepare rendering of opaque objects.
    PrepareOpaque,
    /// Render opaque objects.
    RenderOpaque,
    /// Prepare rendering of transparent objects.
    PrepareTransparent,
    /// Render transparent objects.
    RenderTransparent,
    /// Render screen space effects.
    PostRender,
    /// Tone map the final image.
    ToneMapping,
}

#[derive(SystemLabel)]
pub enum ViewSystem {
    ClearDraw,
    Draw,
    PrepareCamera,
    ScreenSpaceRender,
    ScreenSpaceResize,
    RenderSky,
    RenderOpaque,
    RenderTransparent,
    RenderBloom,
    ToneMapping,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct CorePlugin;

impl RendererPlugin for CorePlugin {
    #[inline]
    fn build(&self, renderer: &mut Renderer) {
        let shader_io = Arc::new(FileShaderIo::new("."));
        let shader_processor = ShaderProcessor::new(shader_io);

        renderer.world.insert_resource(shader_processor);
        renderer.world.init_resource::<OpaqueDraws>();
        renderer.world.init_resource::<TransparentDraws>();
        renderer.world.init_resource::<DrawKeys>();
        renderer.world.init_resource::<IntegratedBrdf>();

        renderer
            .extract
            .add_stage(ExtractStage::PreExtract, SystemStage::parallel())
            .add_stage(ExtractStage::Extract, SystemStage::parallel())
            .add_stage(ExtractStage::PostExtract, SystemStage::parallel())
            .add_stage(ExtractStage::Prepare, SystemStage::parallel());

        renderer
            .view
            .add_stage(ViewStage::PrePrepare, SystemStage::parallel())
            .add_stage(ViewStage::Prepare, SystemStage::parallel())
            .add_stage(ViewStage::PostPrepare, SystemStage::parallel())
            .add_stage(ViewStage::Draw, SystemStage::parallel())
            .add_stage(ViewStage::Clear, SystemStage::parallel())
            .add_stage(ViewStage::PreRender, SystemStage::parallel())
            .add_stage(ViewStage::PrepareOpaque, SystemStage::parallel())
            .add_stage(ViewStage::RenderOpaque, SystemStage::parallel())
            .add_stage(ViewStage::PrepareTransparent, SystemStage::parallel())
            .add_stage(ViewStage::RenderTransparent, SystemStage::parallel())
            .add_stage(ViewStage::PostRender, SystemStage::parallel())
            .add_stage(ViewStage::ToneMapping, SystemStage::parallel());

        renderer
            .extract
            .add_system_to_stage(DefaultStage::First, Extracted::spawn_system)
            .add_system_to_stage(DefaultStage::First, Extracted::despawn_system)
            .add_system_to_stage(DefaultStage::Last, Extracted::despawn_system)
            .add_system_to_stage(ExtractStage::Extract, extract_bloom_settings_system);

        renderer
            .view
            .add_system_to_stage(
                ViewStage::PrePrepare,
                clear_draws_system.label(ViewSystem::ClearDraw),
            )
            .add_system_to_stage(
                ViewStage::PrePrepare,
                prepare_camera_system.label(ViewSystem::PrepareCamera),
            )
            .add_system_to_stage(
                ViewStage::PrePrepare,
                screen_space_resize_system.label(ViewSystem::ScreenSpaceResize),
            )
            .add_system_to_stage(
                ViewStage::Clear,
                sky_render_system.label(ViewSystem::RenderSky),
            )
            .add_system_to_stage(ViewStage::PreRender, draw_system)
            .add_system_to_stage(
                ViewStage::RenderOpaque,
                render_opaque_system.label(ViewSystem::RenderOpaque),
            )
            .add_system_to_stage(
                ViewStage::PrepareTransparent,
                screen_space_render_system.label(ViewSystem::ScreenSpaceRender),
            )
            .add_system_to_stage(
                ViewStage::RenderTransparent,
                render_transparent_system.label(ViewSystem::RenderTransparent),
            )
            .add_system_to_stage(
                ViewStage::PostRender,
                render_bloom_system.label(ViewSystem::RenderBloom),
            )
            .add_system_to_stage(
                ViewStage::ToneMapping,
                tone_mapping_system.label(ViewSystem::ToneMapping),
            );
    }
}
