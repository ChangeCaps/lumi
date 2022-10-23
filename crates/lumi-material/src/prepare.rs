use std::any::type_name;

use lumi_bind::BindingsLayout;
use lumi_core::{
    BlendState, ColorTargetState, ColorWrites, CompareFunction, DepthBiasState, DepthStencilState,
    Device, FragmentState, MultisampleState, PipelineLayout, PrimitiveState,
    RenderPipelineDescriptor, SharedDevice, SharedRenderPipeline, StencilState, TextureFormat,
    VertexAttribute, VertexBufferLayout, VertexState, VertexStepMode,
};
use lumi_shader::{ShaderDefs, ShaderProcessor};

use crate::{Material, MaterialPipeline};

pub struct PreparedMaterialPipeline {
    pub bindings_layout: BindingsLayout,
    pub material_pipeline: MaterialPipeline,
    pub pipeline_layout: PipelineLayout,
    pub render_pipeline: SharedRenderPipeline,
}

impl PreparedMaterialPipeline {
    pub fn new<T: Material>(
        device: &Device,
        shader_defs: &ShaderDefs,
        shader_processor: &mut ShaderProcessor,
        sample_count: u32,
    ) -> Self {
        let vertex_shader = shader_processor
            .process(T::vertex_shader(), shader_defs)
            .unwrap();
        let fragment_shader = shader_processor
            .process(T::fragment_shader(), shader_defs)
            .unwrap();

        let mut material_pipeline = MaterialPipeline {
            vertex_shader,
            fragment_shader,
            vertices: Vec::new(),
        };

        T::specialize(&mut material_pipeline);

        material_pipeline.rebind();

        let bindings_layout = BindingsLayout::new()
            .with_shader(&material_pipeline.vertex_shader)
            .with_shader(&material_pipeline.fragment_shader);

        let pipeline_layout = bindings_layout.create_pipeline_layout(device);

        let render_pipeline = Self::create_render_pipeline::<T>(
            device,
            &pipeline_layout,
            &mut material_pipeline,
            sample_count,
        );

        Self {
            bindings_layout,
            material_pipeline,
            pipeline_layout,
            render_pipeline,
        }
    }

    pub fn create_render_pipeline<T>(
        device: &Device,
        pipeline_layout: &PipelineLayout,
        material_pipeline: &mut MaterialPipeline,
        sample_count: u32,
    ) -> SharedRenderPipeline {
        let vertex_attributes = material_pipeline
            .vertices
            .iter()
            .map(|vertex| {
                [VertexAttribute {
                    offset: 0,
                    shader_location: vertex.location,
                    format: vertex.format,
                }]
            })
            .collect::<Vec<_>>();

        let vertex_buffers = material_pipeline
            .vertices
            .iter()
            .enumerate()
            .map(|(i, vertex)| VertexBufferLayout {
                array_stride: vertex.format.size(),
                step_mode: VertexStepMode::Vertex,
                attributes: &vertex_attributes[i],
            })
            .collect::<Vec<_>>();

        device.create_shared_render_pipeline(&RenderPipelineDescriptor {
            label: Some(&format!("Lumi {} RenderPipeline", type_name::<T>())),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: material_pipeline.vertex_shader.shader_module(device),
                entry_point: "vertex",
                buffers: &vertex_buffers,
            },
            fragment: Some(FragmentState {
                module: material_pipeline.fragment_shader.shader_module(device),
                entry_point: "fragment",
                targets: &[Some(ColorTargetState {
                    format: TextureFormat::Rgba16Float,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Less,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState {
                count: sample_count,
                ..Default::default()
            },
            multiview: None,
        })
    }
}
