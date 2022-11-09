use lumi_bind::{Bind, Bindings, BindingsLayout};
use lumi_core::{
    BlendState, Color, ColorTargetState, ColorWrites, CommandEncoder, FragmentState, LoadOp,
    Operations, RenderPassColorAttachment, RenderPassDescriptor, RenderPipelineDescriptor,
    SharedDevice, SharedRenderPipeline, SharedTextureView, TextureFormat, VertexState,
};
use lumi_shader::{ShaderProcessor, ShaderRef};
use lumi_util::HashMap;
use shiv::{
    system::{Local, Res, ResMut},
    world::{Entity, FromWorld, World},
};

use crate::{RenderDevice, RenderQueue, View};

#[derive(Bind)]
struct ToneMappingBindings {
    #[texture]
    #[sampler(name = "hdr_sampler")]
    hdr_texture: SharedTextureView,
}

pub struct ToneMappingPipeline {
    pub bindings_layout: BindingsLayout,
    pub render_pipeline: SharedRenderPipeline,
}

impl FromWorld for ToneMappingPipeline {
    fn from_world(world: &mut World) -> Self {
        let mut shader_processor = world.resource_mut::<ShaderProcessor>();
        let mut vertex = shader_processor
            .process(
                ShaderRef::module("lumi/fullscreen_vert.wgsl"),
                &Default::default(),
            )
            .unwrap();
        let mut fragment = shader_processor
            .process(
                ShaderRef::module("lumi/tonemapping_frag.wgsl"),
                &Default::default(),
            )
            .unwrap();
        vertex.rebind_with(&mut fragment).unwrap();

        let bindings_layout = BindingsLayout::new()
            .with_shader(&vertex)
            .with_shader(&fragment)
            .bind::<ToneMappingBindings>();

        let device = world.resource::<RenderDevice>();
        let pipeline_layout = bindings_layout.create_pipeline_layout(device);

        let render_pipeline = device.create_shared_render_pipeline(&RenderPipelineDescriptor {
            label: Some("ToneMapping"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &vertex.shader_module(device),
                entry_point: "vertex",
                buffers: &[],
            },
            fragment: Some(FragmentState {
                module: &fragment.shader_module(device),
                entry_point: "fragment",
                targets: &[Some(ColorTargetState {
                    format: TextureFormat::Bgra8UnormSrgb,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview: None,
        });

        Self {
            bindings_layout,
            render_pipeline,
        }
    }
}

pub fn tone_mapping_system(
    mut bindings: Local<HashMap<Entity, Bindings>>,
    pipeline: Local<ToneMappingPipeline>,
    mut encoder: ResMut<CommandEncoder>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    view: Res<View>,
) {
    let bindings = bindings
        .entry(view.camera)
        .or_insert_with(|| pipeline.bindings_layout.create_bindings(&device));

    let tone_mapping_bindings = ToneMappingBindings {
        hdr_texture: view.frame_buffer.hdr_view.clone(),
    };

    bindings.bind::<ToneMappingBindings>(&device, &queue, &tone_mapping_bindings);

    bindings.update_bind_groups(&device);

    let mut tonemap_pass = encoder.begin_render_pass(&RenderPassDescriptor {
        label: Some("Lumi Tonemap Pass"),
        color_attachments: &[Some(RenderPassColorAttachment {
            view: &view.target,
            resolve_target: None,
            ops: Operations {
                load: LoadOp::Clear(Color::TRANSPARENT),
                store: true,
            },
        })],
        depth_stencil_attachment: None,
    });

    tonemap_pass.set_pipeline(&pipeline.render_pipeline);
    bindings.apply(&mut tonemap_pass);

    tonemap_pass.draw(0..3, 0..1);
}
