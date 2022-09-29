use std::borrow::Cow;

use glam::Mat4;
use wgpu::{
    BlendState, ColorTargetState, ColorWrites, CompareFunction, DepthBiasState, DepthStencilState,
    Device, FragmentState, IndexFormat, MultisampleState, PipelineLayoutDescriptor, PrimitiveState,
    Queue, RenderPass, RenderPipeline, RenderPipelineDescriptor, StencilState, TextureFormat,
    VertexAttribute, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
};

use crate::{
    bind::Bind,
    binding::{Bindings, BindingsLayout},
    camera::{CameraInfo, RawCamera},
    environment::{EnvironmentBindings, PreparedEnvironment},
    light::LightBindings,
    mesh::{Mesh, MeshBufferCache, MeshBuffers},
    renderable::{RenderContext, Renderable},
    resources::Resources,
    shader::{DefaultShader, Shader, ShaderProcessor, ShaderRef},
};

#[derive(Clone, Debug)]
pub struct MeshVertexLayout {
    pub attribute: Cow<'static, str>,
    pub format: VertexFormat,
    pub location: u32,
}

#[derive(Debug)]
pub struct MaterialPipeline {
    pub vertex_shader: Shader,
    pub fragment_shader: Shader,
    pub vertices: Vec<MeshVertexLayout>,
}

pub trait Material: Bind + 'static {
    fn vertex_shader() -> ShaderRef {
        ShaderRef::Default(DefaultShader::Vertex)
    }

    fn fragment_shader() -> ShaderRef {
        ShaderRef::Default(DefaultShader::Fragment)
    }

    fn specialize(pipeline: &mut MaterialPipeline) {
        pipeline.vertices = vec![
            MeshVertexLayout {
                attribute: Mesh::POSITION.into(),
                format: VertexFormat::Float32x3,
                location: 0,
            },
            MeshVertexLayout {
                attribute: Mesh::NORMAL.into(),
                format: VertexFormat::Float32x3,
                location: 1,
            },
            MeshVertexLayout {
                attribute: Mesh::TANGENT.into(),
                format: VertexFormat::Float32x4,
                location: 2,
            },
            MeshVertexLayout {
                attribute: Mesh::UV_0.into(),
                format: VertexFormat::Float32x2,
                location: 3,
            },
        ];
    }
}

pub struct Primitive<T> {
    pub material: T,
    pub mesh: Mesh,
}

impl<T> Primitive<T> {
    pub fn new(material: T, mesh: Mesh) -> Self {
        Self { material, mesh }
    }
}

pub struct MeshNodePipeline {
    pub bindings_layout: BindingsLayout,
    pub material_pipeline: MaterialPipeline,
    pub render_pipeline: RenderPipeline,
}

#[derive(Bind)]
struct MeshBindings {
    #[uniform]
    transform: Mat4,
    #[uniform]
    camera: RawCamera,
}

pub struct MeshNode<T> {
    pub primitives: Vec<Primitive<T>>,
    pub transform: Mat4,
}

impl<T> MeshNode<T> {
    pub fn new(material: T, mesh: Mesh, transform: Mat4) -> Self {
        Self {
            primitives: vec![Primitive::new(material, mesh)],
            transform,
        }
    }

    pub fn with_primitive(mut self, material: T, mesh: Mesh) -> Self {
        self.primitives.push(Primitive::new(material, mesh));
        self
    }
}

impl<T: Material + Send + Sync> Renderable for MeshNode<T> {
    type Resource = MeshNodePipeline;
    type State = Vec<Bindings>;

    fn register(device: &Device, _queue: &Queue, resources: &mut Resources) -> Self::Resource {
        let shader_processor = resources.get_mut_or_default::<ShaderProcessor>();

        let vertex = shader_processor.process(T::vertex_shader()).unwrap();
        let fragment = shader_processor.process(T::fragment_shader()).unwrap();

        let mut material_pipeline = MaterialPipeline {
            vertex_shader: vertex,
            fragment_shader: fragment,
            vertices: Vec::new(),
        };

        T::specialize(&mut material_pipeline);

        material_pipeline
            .vertex_shader
            .rebind(&mut material_pipeline.fragment_shader)
            .unwrap();

        let layout = BindingsLayout::new()
            .with_shader(&material_pipeline.vertex_shader)
            .with_shader(&material_pipeline.fragment_shader)
            .bind::<MeshBindings>()
            .bind::<LightBindings>()
            .bind::<EnvironmentBindings>()
            .bind::<T>();

        let bind_group_layouts = layout.create_bind_group_layouts(device);
        let bind_group_layouts = bind_group_layouts.iter().collect::<Vec<_>>();

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &bind_group_layouts,
            push_constant_ranges: &[],
        });

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

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Lumi MaterialPipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &material_pipeline.vertex_shader.shader_module(device),
                entry_point: "vertex",
                buffers: &vertex_buffers,
            },
            fragment: Some(FragmentState {
                module: &material_pipeline.fragment_shader.shader_module(device),
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
                count: 4,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        MeshNodePipeline {
            bindings_layout: layout,
            material_pipeline,
            render_pipeline,
        }
    }

    fn init(_context: &RenderContext<'_>, _resource: &Self::Resource) -> Self::State {
        Vec::new()
    }

    fn prepare(
        &self,
        context: &RenderContext<'_>,
        resources: &mut Resources,
        resource: &Self::Resource,
        state: &mut Self::State,
    ) {
        state.resize_with(self.primitives.len(), || {
            resource.bindings_layout.create_bindings(&context.device)
        });

        let mesh_cache = resources.get_mut_or_default::<MeshBufferCache>();

        for (primitive, bindings) in self.primitives.iter().zip(state.iter_mut()) {
            if !mesh_cache.contains(&primitive.mesh) {
                let mut mesh = if !primitive.mesh.has_attribute(Mesh::NORMAL) {
                    Cow::Owned(primitive.mesh.clone().with_normals())
                } else {
                    Cow::Borrowed(&primitive.mesh)
                };

                if !mesh.has_attribute(Mesh::TANGENT) {
                    mesh = Cow::Owned(mesh.into_owned().with_tangents());
                }

                let buffers = MeshBuffers::new(&context.device, &mesh);
                mesh_cache.insert(&primitive.mesh, buffers);
            }

            bindings.bind(&context.device, &context.queue, &primitive.material);
        }

        let camera = resources.get::<CameraInfo>().unwrap();
        let light_bindings = resources.get::<LightBindings>().unwrap();
        let prepared_environment = resources.get::<PreparedEnvironment>().unwrap();

        let mesh_bindings = MeshBindings {
            transform: self.transform,
            camera: camera.raw(),
        };

        for bindings in state.iter_mut() {
            bindings.bind(&context.device, &context.queue, &mesh_bindings);
            bindings.bind(&context.device, &context.queue, light_bindings);
            bindings.bind(
                &context.device,
                &context.queue,
                &prepared_environment.bindings(),
            );
            bindings.update_bind_groups(&context.device);
        }
    }

    fn render<'a>(
        &self,
        _context: &RenderContext<'_>,
        render_pass: &mut RenderPass<'a>,
        resources: &'a Resources,
        resource: &'a Self::Resource,
        state: &'a Self::State,
    ) {
        render_pass.set_pipeline(&resource.render_pipeline);

        let mesh_cache = resources.get::<MeshBufferCache>().unwrap();

        for (primitive, bindings) in self.primitives.iter().zip(state.iter()) {
            let mesh_buffers = mesh_cache.get(&primitive.mesh).unwrap();

            for vertex_layout in resource.material_pipeline.vertices.iter() {
                let buffer = mesh_buffers
                    .attributes
                    .get(vertex_layout.attribute.as_ref())
                    .unwrap();
                render_pass.set_vertex_buffer(vertex_layout.location, buffer.slice(..));
            }

            if let Some(index_buffer) = &mesh_buffers.index_buffer {
                render_pass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint32);
            }

            bindings.bind_pass(render_pass);

            if let Some(indices) = primitive.mesh.indices() {
                render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
            } else {
                let len = primitive.mesh.positions().unwrap().len() as u32;
                render_pass.draw(0..len, 0..1);
            }
        }
    }
}
