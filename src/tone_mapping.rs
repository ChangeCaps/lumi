use lumi_macro::Bind;
use wgpu::{
    AddressMode, BlendState, ColorTargetState, ColorWrites, CommandEncoder, FilterMode,
    FragmentState, LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor,
    RenderPipeline, TextureFormat, TextureView, VertexState,
};

use crate::{
    Bindings, BindingsLayout, Shader, ShaderProcessor, ShaderRef, SharedDevice, SharedQueue,
    SharedSampler, SharedTextureView,
};

#[derive(Bind)]
struct ToneMappingBindings<'a> {
    #[texture]
    hdr_texture: &'a SharedTextureView,
    #[sampler(filtering = false)]
    hdr_sampler: &'a SharedSampler,
}

pub struct ToneMapping {
    pub bindings: Bindings,
    pub sampler: SharedSampler,
    pub pipeline: RenderPipeline,
}

impl ToneMapping {
    pub fn new(device: &SharedDevice, shader_processor: &mut ShaderProcessor) -> Self {
        let vertex = shader_processor
            .process(ShaderRef::module("lumi/fullscreen_vert.wgsl"))
            .unwrap();
        let fragment = shader_processor
            .process(ShaderRef::module("lumi/tonemapping_frag.wgsl"))
            .unwrap();

        let mut vertex = Shader::from_wgsl(&vertex).unwrap();
        let mut fragment = Shader::from_wgsl(&fragment).unwrap();
        vertex.rebind(&mut fragment).unwrap();

        let bindings_layout = BindingsLayout::new()
            .with_shader(&vertex)
            .with_shader(&fragment)
            .bind::<ToneMappingBindings>();

        let bindings = bindings_layout.create_bindings(device);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ToneMapping"),
            bind_group_layouts: &bindings.layouts(),
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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

        let sampler = device.create_shared_sampler(&wgpu::SamplerDescriptor {
            label: Some("ToneMapping"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            bindings,
            pipeline,
            sampler,
        }
    }

    pub fn run(
        &mut self,
        device: &SharedDevice,
        queue: &SharedQueue,
        encoder: &mut CommandEncoder,
        hdr: &SharedTextureView,
        target: &TextureView,
    ) {
        self.bindings.bind(
            device,
            queue,
            &ToneMappingBindings {
                hdr_texture: hdr,
                hdr_sampler: &self.sampler,
            },
        );

        self.bindings.update_bind_groups(device);

        let mut tonemap_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Lumi Tonemap Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        tonemap_pass.set_pipeline(&self.pipeline);

        for (i, group) in self.bindings.bind_groups().enumerate() {
            tonemap_pass.set_bind_group(i as u32, group, &[]);
        }

        tonemap_pass.draw(0..3, 0..1);
    }
}
