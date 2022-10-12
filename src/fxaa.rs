use wgpu::{
    BlendState, ColorTargetState, ColorWrites, CommandEncoder, Device, FragmentState, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipelineDescriptor, TextureFormat,
    VertexState,
};

use crate::{
    bind::Bind,
    binding::{Bindings, BindingsLayout},
    frame_buffer::FrameBuffer,
    id::CameraId,
    shader::{ShaderProcessor, ShaderRef},
    util::HashMap,
    SharedDevice, SharedRenderPipeline, SharedTextureView,
};

#[derive(Bind)]
struct FxaaBindings {
    #[texture]
    #[sampler(name = "source_sampler")]
    source: SharedTextureView,
}

pub struct Fxaa {
    pub bindings_layout: BindingsLayout,
    pub bindings: HashMap<CameraId, Bindings>,
    pub pipeline: SharedRenderPipeline,
}

impl Fxaa {
    pub fn new(device: &Device, shader_processor: &mut ShaderProcessor) -> Self {
        let defs = Default::default();
        let mut vertex = shader_processor
            .process(ShaderRef::module("lumi/fullscreen_vert.wgsl"), &defs)
            .unwrap();
        let mut fragment = shader_processor
            .process(ShaderRef::module("lumi/fxaa_frag.wgsl"), &defs)
            .unwrap();
        vertex.rebind_with(&mut fragment).unwrap();

        let bindings_layout = BindingsLayout::new()
            .with_shader(&vertex)
            .with_shader(&fragment)
            .bind::<FxaaBindings>();

        let pipline_layout = bindings_layout.create_pipeline_layout(device);

        let pipeline = device.create_shared_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Lumi fxaa pipeline"),
            layout: Some(&pipline_layout),
            vertex: VertexState {
                module: &vertex.shader_module(device),
                entry_point: "vertex",
                buffers: &[],
            },
            fragment: Some(FragmentState {
                module: &fragment.shader_module(device),
                entry_point: "fragment",
                targets: &[Some(ColorTargetState {
                    format: TextureFormat::Rgba16Float,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: Default::default(),
            depth_stencil: Default::default(),
            multisample: Default::default(),
            multiview: Default::default(),
        });

        Self {
            bindings_layout,
            bindings: HashMap::default(),
            pipeline,
        }
    }

    pub fn render(
        &mut self,
        device: &Device,
        queue: &Queue,
        camera: CameraId,
        encoder: &mut CommandEncoder,
        target: &FrameBuffer,
    ) {
        target.copy_offscreen(encoder);

        let bindings = self
            .bindings
            .entry(camera)
            .or_insert_with(|| self.bindings_layout.create_bindings(device));

        bindings.bind(
            device,
            queue,
            &FxaaBindings {
                source: target.offscreen_hdr_view.clone(),
            },
        );

        bindings.update_bind_groups(device);

        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &target.hdr_view,
                resolve_target: None,
                ops: Default::default(),
            })],
            depth_stencil_attachment: None,
        });
        render_pass.set_pipeline(&self.pipeline);
        bindings.apply(&mut render_pass);
        render_pass.draw(0..3, 0..1);
    }
}
