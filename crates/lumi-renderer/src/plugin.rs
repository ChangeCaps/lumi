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
pub enum RenderStage {
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
pub enum RenderSystem {
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
            .render
            .add_stage(RenderStage::PrePrepare, SystemStage::parallel())
            .add_stage(RenderStage::Prepare, SystemStage::parallel())
            .add_stage(RenderStage::PostPrepare, SystemStage::parallel())
            .add_stage(RenderStage::Draw, SystemStage::parallel())
            .add_stage(RenderStage::Clear, SystemStage::parallel())
            .add_stage(RenderStage::PreRender, SystemStage::parallel())
            .add_stage(RenderStage::PrepareOpaque, SystemStage::parallel())
            .add_stage(RenderStage::RenderOpaque, SystemStage::parallel())
            .add_stage(RenderStage::PrepareTransparent, SystemStage::parallel())
            .add_stage(RenderStage::RenderTransparent, SystemStage::parallel())
            .add_stage(RenderStage::PostRender, SystemStage::parallel())
            .add_stage(RenderStage::ToneMapping, SystemStage::parallel());

        renderer
            .extract
            .add_system_to_stage(DefaultStage::First, Extracted::spawn_system)
            .add_system_to_stage(DefaultStage::First, Extracted::despawn_system)
            .add_system_to_stage(ExtractStage::Extract, extract_bloom_settings_system);

        renderer
            .render
            .add_system_to_stage(
                RenderStage::PrePrepare,
                clear_draws_system.label(RenderSystem::ClearDraw),
            )
            .add_system_to_stage(
                RenderStage::PrePrepare,
                prepare_camera_system.label(RenderSystem::PrepareCamera),
            )
            .add_system_to_stage(
                RenderStage::PrePrepare,
                screen_space_resize_system.label(RenderSystem::ScreenSpaceResize),
            )
            .add_system_to_stage(
                RenderStage::Clear,
                sky_render_system.label(RenderSystem::RenderSky),
            )
            .add_system_to_stage(RenderStage::PreRender, draw_system)
            .add_system_to_stage(
                RenderStage::RenderOpaque,
                render_opaque_system.label(RenderSystem::RenderOpaque),
            )
            .add_system_to_stage(
                RenderStage::PrepareTransparent,
                screen_space_render_system.label(RenderSystem::ScreenSpaceRender),
            )
            .add_system_to_stage(
                RenderStage::RenderTransparent,
                render_transparent_system.label(RenderSystem::RenderTransparent),
            )
            .add_system_to_stage(
                RenderStage::PostRender,
                render_bloom_system.label(RenderSystem::RenderBloom),
            )
            .add_system_to_stage(
                RenderStage::ToneMapping,
                tone_mapping_system.label(RenderSystem::ToneMapping),
            );
    }
}
