use std::any::TypeId;

use lumi_bind::BindingsLayout;
use lumi_core::{
    BlendState, ColorTargetState, ColorWrites, CompareFunction, DepthBiasState, DepthStencilState,
    Device, FragmentState, MultisampleState, PipelineLayout, PrimitiveState,
    RenderPipelineDescriptor, SharedDevice, SharedRenderPipeline, StencilState, TextureFormat,
    VertexAttribute, VertexBufferLayout, VertexState, VertexStepMode,
};
use lumi_id::Id;
use lumi_renderer::{
    CameraBindings, IntegratedBrdf, PreparedEnvironment, PreparedLights, PreparedShadows,
    PreparedTransform,
};
use lumi_shader::{ShaderDefs, ShaderProcessor};

use crate::{Material, MaterialPipeline, SsrBindings};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PreparedMaterialPipelineKey {
    pub material_type: TypeId,
    pub shader_defs: ShaderDefs,
}

impl PreparedMaterialPipelineKey {
    #[inline]
    pub fn new<T: Material>(material: &T) -> Self {
        Self {
            material_type: TypeId::of::<T>(),
            shader_defs: material.shader_defs(),
        }
    }

    #[inline]
    pub fn id(&self) -> Id<PreparedMaterialPipeline> {
        Id::from_hash(self)
    }
}

#[derive(Debug)]
pub struct PreparedMaterialPipeline {
    pub bindings_layout: BindingsLayout,
    pub material_pipeline: MaterialPipeline,
    pub pipeline_layout: PipelineLayout,
    pub prepass_pipeline: SharedRenderPipeline,
    pub opaque_pipeline: SharedRenderPipeline,
    pub transparent_pipeline: SharedRenderPipeline,
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
            .with_shader(&material_pipeline.fragment_shader)
            .bind::<CameraBindings>()
            .bind::<PreparedTransform>()
            .bind::<IntegratedBrdf>()
            .bind::<PreparedLights>()
            .bind::<PreparedEnvironment>()
            .bind::<PreparedShadows>()
            .bind::<SsrBindings>()
            .bind::<T>();

        let pipeline_layout = bindings_layout.create_pipeline_layout(device);

        let prepass_pipeline = Self::create_prepass_pipeline(
            device,
            &pipeline_layout,
            &mut material_pipeline,
            sample_count,
        );

        let opaque_pipeline = Self::create_opaque_pipeline(
            device,
            &pipeline_layout,
            &mut material_pipeline,
            sample_count,
        );

        let transparent_pipeline = Self::create_transparent_pipeline(
            device,
            &pipeline_layout,
            &mut material_pipeline,
            sample_count,
        );

        Self {
            bindings_layout,
            material_pipeline,
            pipeline_layout,
            prepass_pipeline,
            opaque_pipeline,
            transparent_pipeline,
        }
    }

    pub fn recreate_pipeline(&mut self, device: &Device, sample_count: u32) {
        self.prepass_pipeline = Self::create_prepass_pipeline(
            device,
            &self.pipeline_layout,
            &mut self.material_pipeline,
            sample_count,
        );

        self.opaque_pipeline = Self::create_opaque_pipeline(
            device,
            &self.pipeline_layout,
            &mut self.material_pipeline,
            sample_count,
        );

        self.transparent_pipeline = Self::create_transparent_pipeline(
            device,
            &self.pipeline_layout,
            &mut self.material_pipeline,
            sample_count,
        );
    }

    pub fn create_prepass_pipeline(
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
            label: Some("Lumi Material Depth Prepass RenderPipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: material_pipeline.vertex_shader.shader_module(device),
                entry_point: "vertex",
                buffers: &vertex_buffers,
            },
            fragment: None,
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

    pub fn create_opaque_pipeline(
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
            label: Some("Lumi Material RenderPipeline"),
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
                depth_write_enabled: false,
                depth_compare: CompareFunction::LessEqual,
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

    pub fn create_transparent_pipeline(
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
            label: Some("Lumi Material RenderPipeline"),
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
                depth_compare: CompareFunction::LessEqual,
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
