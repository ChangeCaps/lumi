use lumi_bind::{Bind, Bindings, BindingsLayout};
use lumi_core::{
    BlendState, Color, ColorTargetState, ColorWrites, CommandEncoder, CompareFunction,
    DepthStencilState, Device, FragmentState, MultisampleState, PipelineLayout,
    RenderPipelineDescriptor, Resources, SharedDevice, SharedRenderPipeline, SharedTextureView,
    TextureFormat, VertexState,
};
use lumi_macro::PhaseLabel;
use lumi_shader::{DefaultShader, Shader, ShaderProcessor, ShaderRef};
use lumi_world::{RawCamera, World};

use crate::{
    CorePhase, PreparedEnvironment, RenderPlugin, RenderViewPhase, RendererBuilder,
    ViewPhaseContext,
};

#[derive(Bind)]
pub struct SkyBindings {
    #[uniform]
    pub camera: RawCamera,
    #[texture(dimension = cube)]
    #[sampler(name = "sky_sampler")]
    pub sky_texture: SharedTextureView,
}

pub struct SkyPipeline {
    pub vertex_shader: Shader,
    pub fragment_shader: Shader,
    pub bindings_layout: BindingsLayout,
    pub pipeline_layout: PipelineLayout,
    pub render_pipeline: SharedRenderPipeline,
    pub sample_count: u32,
}

impl SkyPipeline {
    pub fn new(device: &Device, shader_processor: &mut ShaderProcessor, sample_count: u32) -> Self {
        let mut vertex_shader = shader_processor
            .process(ShaderRef::module("lumi/sky_vert.wgsl"), &Default::default())
            .unwrap();
        let mut fragment_shader = shader_processor
            .process(ShaderRef::Default(DefaultShader::Sky), &Default::default())
            .unwrap();
        vertex_shader.rebind_with(&mut fragment_shader).unwrap();

        vertex_shader.compile(device).unwrap();
        fragment_shader.compile(device).unwrap();

        let bindings_layout = BindingsLayout::new()
            .with_shader(&vertex_shader)
            .with_shader(&fragment_shader)
            .bind::<SkyBindings>();

        let pipeline_layout = bindings_layout.create_pipeline_layout(device);
        let render_pipeline = Self::create_render_pipeline(
            device,
            &vertex_shader,
            &fragment_shader,
            &pipeline_layout,
            sample_count,
        );

        Self {
            vertex_shader,
            fragment_shader,
            bindings_layout,
            pipeline_layout,
            render_pipeline,
            sample_count,
        }
    }

    fn create_render_pipeline(
        device: &Device,
        vertex_shader: &Shader,
        fragment_shader: &Shader,
        pipeline_layout: &PipelineLayout,
        sample_count: u32,
    ) -> SharedRenderPipeline {
        device.create_shared_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Sky Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: vertex_shader.get_shader_module().unwrap(),
                entry_point: "vertex",
                buffers: &[],
            },
            fragment: Some(FragmentState {
                module: fragment_shader.get_shader_module().unwrap(),
                entry_point: "fragment",
                targets: &[Some(ColorTargetState {
                    format: TextureFormat::Rgba16Float,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: Default::default(),
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: CompareFunction::Always,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: MultisampleState {
                count: sample_count,
                ..Default::default()
            },
            multiview: Default::default(),
        })
    }

    pub fn recreate_pipeline(&mut self, device: &Device, sample_count: u32) {
        if self.sample_count != sample_count {
            self.sample_count = sample_count;

            self.render_pipeline = Self::create_render_pipeline(
                device,
                &self.vertex_shader,
                &self.fragment_shader,
                &self.pipeline_layout,
                sample_count,
            );
        }
    }
}

pub struct SkyViewState {
    pub bindings: Bindings,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct RenderSky;

impl RenderViewPhase for RenderSky {
    fn prepare(&mut self, context: &ViewPhaseContext, _world: &World, resources: &mut Resources) {
        let sky_pipeline = if let Some(sky_pipeline) = resources.remove::<SkyPipeline>() {
            sky_pipeline
        } else {
            let mut shader_processor = resources.get_mut::<ShaderProcessor>().unwrap();

            SkyPipeline::new(
                context.device,
                &mut shader_processor,
                context.target.sample_count(),
            )
        };

        let sky_texture = if let Some(environment) = resources.get::<PreparedEnvironment>() {
            environment.sky.clone()
        } else {
            return;
        };

        let state =
            resources.get_id_or_insert_with::<SkyViewState>(context.view.camera.cast(), || {
                SkyViewState {
                    bindings: sky_pipeline.bindings_layout.create_bindings(context.device),
                }
            });

        let bindings = SkyBindings {
            camera: context.view.raw_camera,
            sky_texture,
        };

        state
            .bindings
            .bind(context.device, context.queue, &bindings);
        state.bindings.update_bind_groups(context.device);

        resources.insert(sky_pipeline);
    }

    fn render(
        &self,
        context: &ViewPhaseContext,
        encoder: &mut CommandEncoder,
        _world: &World,
        resources: &Resources,
    ) {
        let mut render_pass = context
            .target
            .begin_hdr_clear_pass(encoder, Color::TRANSPARENT);

        if let Some(state) = resources.get_id::<SkyViewState>(context.view.camera) {
            let pipeline = resources.get::<SkyPipeline>().unwrap();

            render_pass.set_pipeline(&pipeline.render_pipeline);

            state.bindings.apply(&mut render_pass);

            render_pass.draw(0..3, 0..1);
        }
    }
}

#[derive(Clone, Copy, Debug, PhaseLabel)]
pub enum SkyPhase {
    Render,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SkyPlugin;

impl RenderPlugin for SkyPlugin {
    fn build(&self, builder: &mut RendererBuilder) {
        builder.add_view_phase_after(CorePhase::Clear, SkyPhase::Render, RenderSky);
    }
}
