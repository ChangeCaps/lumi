use std::{any::TypeId, borrow::Cow, collections::HashMap};

use glam::Mat4;
use lumi_macro::Bind;
use wgpu::{
    util::BufferInitDescriptor, BlendState, BufferUsages, Color, ColorTargetState, ColorWrites,
    CompareFunction, DepthBiasState, DepthStencilState, Face, FragmentState, FrontFace,
    IndexFormat, LoadOp, MultisampleState, Operations, PipelineLayout, PipelineLayoutDescriptor,
    PolygonMode, PrimitiveState, PrimitiveTopology, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, StencilState, Texture, TextureFormat, VertexBufferLayout,
    VertexState, VertexStepMode,
};

use crate::{
    Bindings, BindingsLayout, CameraId, DynMaterial, FrameBuffer, LightBindings, Mesh, MeshId,
    NodeId, RawCamera, Shader, ShaderProcessor, SharedBuffer, SharedDevice, SharedQueue,
    UniformBuffer, World,
};

#[allow(unused)]
struct CachedMaterial {
    vertex_shader: Shader,
    fragment_shader: Shader,
    bindings_layout: BindingsLayout,
    pipeline_layout: PipelineLayout,
    pipeline: RenderPipeline,
}

struct CachedMesh {
    vertex_buffers: HashMap<String, SharedBuffer>,
    index_buffer: Option<SharedBuffer>,
    vertex_count: u32,
    index_count: u32,
}

#[derive(Bind)]
struct MeshBindings<'a> {
    #[uniform]
    transform: Mat4,
    #[uniform]
    camera: &'a UniformBuffer<RawCamera>,
}

pub struct RenderSettings {
    pub clear_color: [f32; 4],
    pub aspect_ratio: Option<f32>,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            clear_color: [0.0, 0.0, 0.0, 1.0],
            aspect_ratio: None,
        }
    }
}

pub struct Renderer {
    pub device: SharedDevice,
    pub queue: SharedQueue,
    pub settings: RenderSettings,
    material_cache: HashMap<TypeId, CachedMaterial>,
    mesh_cache: HashMap<MeshId, CachedMesh>,
    camera_cache: HashMap<CameraId, UniformBuffer<RawCamera>>,
    light_bindings: LightBindings,
    bindings: HashMap<NodeId, Bindings>,
    frame_buffer: FrameBuffer,
    shader_processor: ShaderProcessor,
}

impl Renderer {
    pub fn new(device: SharedDevice, queue: SharedQueue) -> Self {
        let frame_buffer = FrameBuffer::new(&device, 1, 1);

        Self {
            device,
            queue,
            settings: RenderSettings::default(),
            material_cache: HashMap::new(),
            mesh_cache: HashMap::new(),
            camera_cache: HashMap::new(),
            light_bindings: LightBindings::default(),
            bindings: HashMap::new(),
            shader_processor: ShaderProcessor::default(),
            frame_buffer,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.frame_buffer.resize(&self.device, width, height);
    }

    fn prepare_material(&mut self, material: &dyn DynMaterial) {
        let type_id = material.type_id();
        if self.material_cache.contains_key(&type_id) {
            return;
        }
        let vertex_source = self
            .shader_processor
            .process(material.vertex_shader())
            .unwrap();

        let fragment_source = self
            .shader_processor
            .process(material.fragment_shader())
            .unwrap();

        let mut vertex_shader = Shader::from_wgsl(&vertex_source, None).unwrap();
        let mut fragment_shader =
            Shader::from_wgsl(&fragment_source, Some(&vertex_shader)).unwrap();

        let bindings_layout = BindingsLayout::new()
            .with_shader(&mut vertex_shader)
            .with_shader(&mut fragment_shader)
            .append(material.entries())
            .bind::<MeshBindings>()
            .bind::<LightBindings>();

        let bind_group_layouts = bindings_layout.create_bind_group_layouts(&self.device);
        let bind_group_layouts = &bind_group_layouts.iter().collect::<Vec<_>>();

        let pipeline_layout = self
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts,
                push_constant_ranges: &[],
            });

        let pipeline = self
            .device
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: VertexState {
                    module: &vertex_shader.shader_module(&self.device),
                    entry_point: "vertex",
                    buffers: &[
                        VertexBufferLayout {
                            array_stride: 12,
                            step_mode: VertexStepMode::Vertex,
                            attributes: &wgpu::vertex_attr_array![0 => Float32x3],
                        },
                        VertexBufferLayout {
                            array_stride: 12,
                            step_mode: VertexStepMode::Vertex,
                            attributes: &wgpu::vertex_attr_array![1 => Float32x3],
                        },
                        VertexBufferLayout {
                            array_stride: 16,
                            step_mode: VertexStepMode::Vertex,
                            attributes: &wgpu::vertex_attr_array![2 => Float32x3],
                        },
                        VertexBufferLayout {
                            array_stride: 8,
                            step_mode: VertexStepMode::Vertex,
                            attributes: &wgpu::vertex_attr_array![3 => Float32x2],
                        },
                    ],
                },
                fragment: Some(FragmentState {
                    module: &fragment_shader.shader_module(&self.device),
                    entry_point: "fragment",
                    targets: &[Some(ColorTargetState {
                        format: TextureFormat::Bgra8UnormSrgb,
                        blend: Some(BlendState::ALPHA_BLENDING),
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                primitive: PrimitiveState {
                    topology: PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: FrontFace::Ccw,
                    cull_mode: Some(Face::Back),
                    polygon_mode: PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: Some(DepthStencilState {
                    format: TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: CompareFunction::Less,
                    stencil: StencilState::default(),
                    bias: DepthBiasState::default(),
                }),
                multisample: MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            });

        let cached_material = CachedMaterial {
            vertex_shader,
            fragment_shader,
            bindings_layout,
            pipeline_layout,
            pipeline,
        };

        self.material_cache.insert(type_id, cached_material);
    }

    fn prepare_materials(&mut self, world: &World) {
        for node in world.nodes() {
            self.prepare_material(node.material.as_ref());
        }
    }

    fn prepare_mesh(&mut self, mesh: &Mesh) {
        if self.mesh_cache.contains_key(&mesh.id()) {
            return;
        }

        let id = mesh.id();

        let mesh = if !mesh.has_attribute(Mesh::NORMAL) {
            let mut mesh = mesh.clone();
            mesh.generate_tangents();
            Cow::Owned(mesh)
        } else {
            Cow::Borrowed(mesh)
        };

        let mesh = if !mesh.has_attribute(Mesh::TANGENT) {
            let mut mesh = mesh.into_owned();
            mesh.generate_tangents();
            Cow::Owned(mesh)
        } else {
            mesh
        };

        let mut vertex_buffers = HashMap::new();
        let mut vertex_count = 0;
        for (name, attribute) in mesh.attributes() {
            let buffer = self
                .device
                .create_shared_buffer_init(&BufferInitDescriptor {
                    label: None,
                    contents: attribute.data(),
                    usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                });

            vertex_buffers.insert(name.to_string(), buffer);
            vertex_count = attribute.len();
        }

        let mut index_count = 0;
        let index_buffer = if let Some(indices) = mesh.indices() {
            let data = bytemuck::cast_slice(indices);

            let buffer = self
                .device
                .create_shared_buffer_init(&BufferInitDescriptor {
                    label: None,
                    contents: data,
                    usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
                });

            index_count = indices.len();

            Some(buffer)
        } else {
            None
        };

        let cached_mesh = CachedMesh {
            vertex_buffers,
            index_buffer,
            vertex_count: vertex_count as u32,
            index_count: index_count as u32,
        };

        self.mesh_cache.insert(id, cached_mesh);
    }

    fn prepare_meshes(&mut self, world: &World) {
        for node in world.nodes() {
            self.prepare_mesh(&node.mesh);
        }
    }

    fn prepare_cameras(&mut self, world: &World) {
        for (id, camera) in world.iter_cameras() {
            if let Some(cached_camera) = self.camera_cache.get_mut(&id) {
                **cached_camera = camera.raw_aspect(self.frame_buffer.aspect_ratio());
            } else {
                let camera = camera.raw_aspect(self.frame_buffer.aspect_ratio());
                self.camera_cache.insert(id, UniformBuffer::new(camera));
            }
        }
    }

    fn prepare_lights(&mut self, world: &World) {
        self.light_bindings.clear();

        for light in world.lights() {
            self.light_bindings.push(light.clone());
        }
    }

    fn prepare_bindings(&mut self, world: &World, camera: CameraId) {
        for (id, node) in world.iter_nodes() {
            let cached_material = self.material_cache.get(&node.material.type_id()).unwrap();

            let bindings = self
                .bindings
                .entry(id)
                .or_insert_with(|| Bindings::new(&self.device, &cached_material.bindings_layout));

            let camera = self.camera_cache.get(&camera).unwrap();

            let mesh_bindings = MeshBindings {
                transform: node.transform,
                camera: &camera,
            };

            bindings.bind(&self.device, &self.queue, node.material.as_ref());
            bindings.bind(&self.device, &self.queue, &mesh_bindings);
            bindings.bind(&self.device, &self.queue, &self.light_bindings);
            bindings.update_bind_groups(&self.device);
        }
    }

    pub fn render_camera(&mut self, world: &World, target: &Texture, camera: CameraId) {
        self.prepare_materials(world);
        self.prepare_meshes(world);
        self.prepare_cameras(world);
        self.prepare_lights(world);
        self.prepare_bindings(world, camera);

        let target_view = target.create_view(&Default::default());
        let depth_view = self.frame_buffer.depth.create_view(&Default::default());

        let mut encoder = self.device.create_command_encoder(&Default::default());
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Lumi HDR Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &target_view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color::BLACK),
                    store: true,
                },
            })],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: &depth_view,
                depth_ops: Some(Operations {
                    load: LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        for (id, node) in world.iter_nodes() {
            let material = self.material_cache.get(&node.material.type_id()).unwrap();
            let mesh = self.mesh_cache.get(&node.mesh.id()).unwrap();
            let bindings = self.bindings.get(&id).unwrap();

            render_pass.set_pipeline(&material.pipeline);

            for (i, bind_group) in bindings.bind_groups().enumerate() {
                render_pass.set_bind_group(i as u32, bind_group, &[]);
            }

            if let Some(buffer) = mesh.vertex_buffers.get(Mesh::POSITION) {
                render_pass.set_vertex_buffer(0, buffer.slice(..));
            }

            if let Some(buffer) = mesh.vertex_buffers.get(Mesh::NORMAL) {
                render_pass.set_vertex_buffer(1, buffer.slice(..));
            }

            if let Some(buffer) = mesh.vertex_buffers.get(Mesh::TANGENT) {
                render_pass.set_vertex_buffer(2, buffer.slice(..));
            }

            if let Some(buffer) = mesh.vertex_buffers.get(Mesh::UV_0) {
                render_pass.set_vertex_buffer(3, buffer.slice(..));
            }

            if let Some(buffer) = mesh.index_buffer.as_ref() {
                render_pass.set_index_buffer(buffer.slice(..), IndexFormat::Uint32);
                render_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
            } else {
                render_pass.draw(0..mesh.vertex_count, 0..1);
            }
        }

        drop(render_pass);
        self.queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn render(&mut self, world: &World, target: &Texture) {
        for (id, _camera) in world.iter_cameras() {
            self.render_camera(world, target, id);
        }
    }
}
