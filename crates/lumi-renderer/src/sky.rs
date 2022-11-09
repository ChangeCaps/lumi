use lumi_bind::{Bind, Bindings, BindingsLayout};
use lumi_core::{
    BlendState, Color, ColorTargetState, ColorWrites, CommandEncoder, CompareFunction,
    DepthStencilState, Device, FragmentState, MultisampleState, PipelineLayout,
    RenderPipelineDescriptor, SharedDevice, SharedRenderPipeline, SharedTextureView, TextureFormat,
    VertexState,
};
use lumi_shader::{DefaultShader, Shader, ShaderProcessor, ShaderRef};
use lumi_util::HashMap;
use shiv::{
    query::Query,
    system::{Local, Res, ResMut},
    world::{Entity, FromWorld, World},
};

use crate::{PreparedCamera, PreparedEnvironment, RenderDevice, RenderQueue, View};

#[derive(Bind)]
pub struct SkyBindings {
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

impl FromWorld for SkyPipeline {
    fn from_world(world: &mut World) -> Self {
        let mut shader_processor = world.resource_mut::<ShaderProcessor>();
        let mut vertex_shader = shader_processor
            .process(ShaderRef::module("lumi/sky_vert.wgsl"), &Default::default())
            .unwrap();
        let mut fragment_shader = shader_processor
            .process(ShaderRef::Default(DefaultShader::Sky), &Default::default())
            .unwrap();
        vertex_shader.rebind_with(&mut fragment_shader).unwrap();

        let device = world.resource::<RenderDevice>();
        vertex_shader.compile(device).unwrap();
        fragment_shader.compile(device).unwrap();

        let bindings_layout = BindingsLayout::new()
            .with_shader(&vertex_shader)
            .with_shader(&fragment_shader)
            .bind::<PreparedCamera>()
            .bind::<SkyBindings>();

        let pipeline_layout = bindings_layout.create_pipeline_layout(device);
        let render_pipeline = Self::create_render_pipeline(
            device,
            &vertex_shader,
            &fragment_shader,
            &pipeline_layout,
            1,
        );

        Self {
            vertex_shader,
            fragment_shader,
            bindings_layout,
            pipeline_layout,
            render_pipeline,
            sample_count: 1,
        }
    }
}

impl SkyPipeline {
    fn recreate_pipeline(&mut self, device: &Device, sample_count: u32) {
        if self.sample_count == sample_count {
            return;
        }

        self.render_pipeline = Self::create_render_pipeline(
            device,
            &self.vertex_shader,
            &self.fragment_shader,
            &self.pipeline_layout,
            sample_count,
        );
        self.sample_count = sample_count;
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
}

pub fn sky_render_system(
    mut bindings: Local<HashMap<Entity, Bindings>>,
    mut pipeline: Local<SkyPipeline>,
    mut encoder: ResMut<CommandEncoder>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    view: Res<View>,
    environment: Res<PreparedEnvironment>,
    camera_query: Query<&PreparedCamera>,
) {
    pipeline.recreate_pipeline(&device, view.frame_buffer.sample_count());

    let bindings = bindings
        .entry(view.camera)
        .or_insert_with(|| pipeline.bindings_layout.create_bindings(&device));

    let camera = camera_query.get(view.camera).unwrap();

    let sky_bindings = SkyBindings {
        sky_texture: environment.sky.clone(),
    };

    bindings.bind(&device, &queue, camera);
    bindings.bind(&device, &queue, &sky_bindings);

    bindings.update_bind_groups(&device);

    let mut render_pass = view
        .frame_buffer
        .begin_hdr_clear_pass(&mut encoder, Color::TRANSPARENT);

    render_pass.set_pipeline(&pipeline.render_pipeline);

    bindings.apply(&mut render_pass);

    render_pass.draw(0..3, 0..1);
}
