mod standard;
mod unlit;

use std::{any::TypeId, borrow::Cow};

use glam::{Mat4, Vec3};
use smallvec::SmallVec;
use wgpu::{
    BlendState, ColorTargetState, ColorWrites, CommandEncoder, CompareFunction, DepthBiasState,
    DepthStencilState, Device, FragmentState, IndexFormat, MultisampleState, PipelineLayout,
    PipelineLayoutDescriptor, PrimitiveState, Queue, RenderPass, RenderPipelineDescriptor,
    StencilState, TextureFormat, VertexAttribute, VertexBufferLayout, VertexFormat, VertexState,
    VertexStepMode,
};

use crate::{
    aabb::{Aabb, RenderFrustum},
    bind::Bind,
    binding::{Bindings, BindingsLayout},
    bloom::{BloomPipeline, MipChain},
    camera::RawCamera,
    environment::{EnvironmentBindings, PreparedEnvironment},
    frame_buffer::FrameBuffer,
    light::LightBindings,
    mesh::{Mesh, MeshBuffers, PrepareMeshFn},
    prelude::World,
    renderable::Renderable,
    renderer::{RenderSettings, RenderViewPhase, ViewPhaseContext},
    resources::Resources,
    shader::{DefaultShader, Shader, ShaderDefs, ShaderDefsHash, ShaderProcessor, ShaderRef},
    shadow::{ShadowFunctions, ShadowReceiverBindings},
    SharedBindGroup, SharedBuffer, SharedDevice, SharedRenderPipeline, SharedTextureView,
};

pub use standard::*;
pub use unlit::*;

#[derive(Bind)]
struct SsrBindings {
    #[texture]
    #[sampler(name = "ssr_sampler")]
    pub ssr_texture: SharedTextureView,
}

#[derive(Bind)]
pub struct MeshBindings {
    #[uniform]
    pub transform: Mat4,
    #[uniform]
    pub camera: RawCamera,
}

#[derive(Clone, Debug)]
pub struct MeshNode<T = StandardMaterial> {
    pub primitives: Vec<Primitive<T>>,
    pub transform: Mat4,
}

impl<T> Default for MeshNode<T> {
    fn default() -> Self {
        Self {
            primitives: Vec::new(),
            transform: Mat4::IDENTITY,
        }
    }
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

    pub fn inside_render_frustum(&self, frustum: &RenderFrustum, resources: &Resources) -> bool {
        for primitive in &self.primitives {
            if let Some(aabb) = resources.get_key(&primitive.mesh.id()) {
                if frustum.should_render(aabb, self.transform) {
                    return true;
                }
            }
        }

        false
    }
}

#[derive(Default)]
struct MaterialState {
    bindings: HashMap<CameraId, Vec<(Bindings, ShaderDefsHash)>>,
}

struct MaterialDraw {
    pipline: SharedRenderPipeline,
    bind_groups: SmallVec<[SharedBindGroup; 4]>,
    vertex_buffers: SmallVec<[(u32, SharedBuffer); 4]>,
    index_buffer: Option<SharedBuffer>,
    draw_command: DrawCommand,
    use_ssr: bool,
    aabb: Option<Aabb>,
    transform: Mat4,
}

impl MaterialDraw {
    fn draw<'a>(
        encoder: &mut CommandEncoder,
        target: &FrameBuffer,
        draws: impl Iterator<Item = &'a MaterialDraw>,
    ) {
        let mut render_pass = target.begin_hdr_render_pass(encoder, true);

        let mut current_pipeline = None;
        let mut current_vertex_buffers = [None; 8];
        let mut current_index_buffer = None;
        for draw in draws {
            if current_pipeline != Some(draw.pipline.id()) {
                render_pass.set_pipeline(&draw.pipline);
                current_pipeline = Some(draw.pipline.id());
            }

            for (i, bind_group) in draw.bind_groups.iter().enumerate() {
                render_pass.set_bind_group(i as u32, bind_group, &[]);
            }

            for (location, buffer) in draw.vertex_buffers.iter() {
                if current_vertex_buffers[*location as usize] != Some(buffer.id()) {
                    render_pass.set_vertex_buffer(*location, buffer.slice(..));
                    current_vertex_buffers[*location as usize] = Some(buffer.id());
                }
            }

            if let Some(index_buffer) = &draw.index_buffer {
                if current_index_buffer != Some(index_buffer.id()) {
                    render_pass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint32);
                    current_index_buffer = Some(index_buffer.id());
                }
            }

            draw.draw_command.draw(&mut render_pass);
        }
    }
}

#[derive(Clone, Copy)]
struct MaterialFunctions {
    prepare: fn(
        phase: &mut MaterialPhase,
        context: &ViewPhaseContext,
        ssr: &SharedTextureView,
        world: &World,
        resources: &mut Resources,
    ),
    render: fn(
        phase: &MaterialPhase,
        context: &ViewPhaseContext,
        world: &World,
        resources: &Resources,
        draws: &mut Vec<MaterialDraw>,
    ),
}

impl MaterialFunctions {
    fn new<T: Material + Send + Sync>() -> Self {
        Self {
            prepare: |phase, context, ssr, world, resources| {
                let mut pipelines =
                    resources.remove_key_or_default::<MeshNodePipelines>(&TypeId::of::<T>());
                let sample_count = resources.get::<RenderSettings>().unwrap().sample_count;

                for (node_id, node) in world.iter_nodes::<MeshNode<T>>() {
                    let state = phase.node_state.entry(node_id).or_default();

                    let bindings = state.bindings.entry(context.view.camera).or_default();

                    for (i, primitive) in node.primitives.iter().enumerate() {
                        let shader_defs = primitive.material.shader_defs();
                        let hash = shader_defs.hash();

                        if !pipelines.contains(&hash) {
                            let shader_processor = resources.get_mut::<ShaderProcessor>().unwrap();

                            let pipeline = MeshNodePipeline::new::<T>(
                                context.device,
                                &shader_defs,
                                shader_processor,
                                sample_count,
                            );

                            pipelines.push(hash, pipeline);
                        }

                        let pipeline = pipelines.get(&hash).unwrap();

                        if bindings.len() <= i {
                            let mut _bindings =
                                pipeline.bindings_layout.create_bindings(context.device);

                            bindings.push((_bindings, hash));
                        } else {
                            if bindings[i].1 != hash {
                                bindings[i] = (
                                    pipeline.bindings_layout.create_bindings(context.device),
                                    hash,
                                );
                            }
                        }

                        if let Some(aabb) = resources.get_key::<Aabb>(&primitive.mesh.id()) {
                            if !context.view.intersects(aabb, node.transform) {
                                continue;
                            }
                        }

                        let bindings = &mut bindings[i].0;
                        bindings.bind(&context.device, &context.queue, &primitive.material);
                    }

                    let light_bindings = resources.get::<LightBindings>().unwrap();
                    let shadow_bindings = resources.get::<ShadowReceiverBindings>().unwrap();
                    let prepared_environment = resources.get::<PreparedEnvironment>().unwrap();

                    let mesh_bindings = MeshBindings {
                        transform: node.transform,
                        camera: context.view.raw_camera,
                    };

                    let ssr_bindings = SsrBindings {
                        ssr_texture: ssr.clone(),
                    };

                    for (primitive, (bindings, _)) in
                        node.primitives.iter().zip(bindings.iter_mut())
                    {
                        if let Some(aabb) = resources.get_key::<Aabb>(&primitive.mesh.id()) {
                            if !context.view.intersects(aabb, node.transform) {
                                continue;
                            }
                        }

                        bindings.bind(&context.device, &context.queue, &mesh_bindings);
                        bindings.bind(&context.device, &context.queue, light_bindings);
                        bindings.bind(&context.device, &context.queue, shadow_bindings);
                        bindings.bind(&context.device, &context.queue, &ssr_bindings);
                        bindings.bind(
                            &context.device,
                            &context.queue,
                            &prepared_environment.bindings(),
                        );
                        bindings.update_bind_groups(&context.device);
                    }
                }

                resources.insert_key(TypeId::of::<T>(), pipelines);
            },
            render: |phase, context, world, resources, draws| {
                let pipelines = resources
                    .get_key::<MeshNodePipelines>(&TypeId::of::<T>())
                    .unwrap();

                for (node_id, node) in world.iter_nodes::<MeshNode<T>>() {
                    let state = phase.node_state.get(&node_id).unwrap();

                    let bindings = state.bindings.get(&context.view.camera).unwrap();

                    for (primitive, (bindings, _)) in node.primitives.iter().zip(bindings.iter()) {
                        let hash = primitive.material.shader_defs_hash();
                        let pipeline = pipelines.get(&hash).unwrap();

                        if let Some(aabb) = resources.get_key::<Aabb>(&primitive.mesh.id()) {
                            if !context.view.intersects(aabb, node.transform) {
                                continue;
                            }
                        }

                        let mesh_buffers = resources
                            .get_key::<MeshBuffers>(&primitive.mesh.id())
                            .unwrap();

                        let aabb = resources.get_key::<Aabb>(&primitive.mesh.id()).cloned();

                        let mut vertex_buffers = SmallVec::new();
                        for vertex_layout in pipeline.material_pipeline.vertices.iter() {
                            let buffer = mesh_buffers
                                .attributes
                                .get(vertex_layout.attribute.as_ref())
                                .unwrap();
                            vertex_buffers.push((vertex_layout.location, buffer.clone()));
                        }

                        let bind_groups = bindings.bind_groups().cloned().collect();

                        let draw = MaterialDraw {
                            pipline: pipeline.render_pipeline.clone(),
                            bind_groups,
                            vertex_buffers,
                            index_buffer: mesh_buffers.index_buffer.clone(),
                            draw_command: primitive.mesh.draw_command(),
                            use_ssr: primitive.material.use_ssr(),
                            aabb,
                            transform: node.transform,
                        };

                        draws.push(draw);
                    }
                }
            },
        }
    }
}

pub struct MaterialPhase {
    ssr_mip_chain: Option<MipChain>,
    node_state: HashMap<NodeId, MaterialState>,
    sample_count: u32,
}

impl Default for MaterialPhase {
    fn default() -> Self {
        Self {
            ssr_mip_chain: None,
            node_state: HashMap::default(),
            sample_count: 4,
        }
    }
}

impl RenderViewPhase for MaterialPhase {
    fn prepare(
        &mut self,
        context: &ViewPhaseContext,
        target: &FrameBuffer,
        world: &World,
        resources: &mut Resources,
    ) {
        let sample_count = resources.get::<RenderSettings>().unwrap().sample_count;

        if self.sample_count != sample_count {
            self.sample_count = sample_count;
            for pipeline in resources.iter_mut::<MeshNodePipelines>() {
                for pipeline in pipeline.iter_mut() {
                    pipeline.render_pipeline = MeshNodePipeline::create_render_pipeline(
                        &context.device,
                        &pipeline.pipeline_layout,
                        &mut pipeline.material_pipeline,
                        sample_count,
                    );
                }
            }
        }

        let bloom_pipeline = resources.get::<BloomPipeline>().unwrap();
        if self.ssr_mip_chain.is_none() {
            self.ssr_mip_chain = Some(MipChain::new(
                &context.device,
                &bloom_pipeline.down_layout,
                target.width(),
                target.height(),
                None,
            ));
        }
        let ssr = self.ssr_mip_chain.as_mut().unwrap();

        if ssr.width() != target.width() || ssr.height() != target.height() {
            *ssr = MipChain::new(
                &context.device,
                &bloom_pipeline.down_layout,
                target.width(),
                target.height(),
                None,
            );
        }
        ssr.prepare_downsample_bindings(&context.device, &context.queue, &target.hdr_view, 4.0);

        let material_functions = resources.remove_keyed::<MaterialFunctions>();

        let ssr = self.ssr_mip_chain.as_ref().unwrap().view.clone();
        for material_function in material_functions.values() {
            (material_function.prepare)(self, context, &ssr, world, resources);
        }

        resources.insert(material_functions);
    }

    fn render(
        &self,
        context: &ViewPhaseContext,
        encoder: &mut CommandEncoder,
        target: &FrameBuffer,
        world: &World,
        resources: &Resources,
    ) {
        let mut draws = Vec::with_capacity(1024);

        for material_function in resources.iter::<MaterialFunctions>() {
            (material_function.render)(self, context, world, resources, &mut draws);
        }

        draws.sort_unstable_by(|a, b| {
            let center_a = a.aabb.map_or(Vec3::ZERO, |a| a.center().into());
            let center_b = b.aabb.map_or(Vec3::ZERO, |a| a.center().into());

            let view_a = context.view.raw_camera.view_proj * a.transform * center_a.extend(1.0);
            let view_b = context.view.raw_camera.view_proj * b.transform * center_b.extend(1.0);

            view_b.z.partial_cmp(&view_a.z).unwrap()
        });

        let first_ssr = draws.iter().rposition(|draw| draw.use_ssr);

        if let Some(i) = first_ssr {
            // opaque pass
            MaterialDraw::draw(
                encoder,
                target,
                draws[..i].iter().filter(|draw| !draw.use_ssr),
            );

            let bloom_pipeline = resources.get::<BloomPipeline>().unwrap();
            self.ssr_mip_chain
                .as_ref()
                .unwrap()
                .downsample(bloom_pipeline, encoder);

            // refraction pass
            MaterialDraw::draw(
                encoder,
                target,
                draws.iter().enumerate().filter_map(|(i, draw)| {
                    if i >= first_ssr.unwrap() || draw.use_ssr {
                        Some(draw)
                    } else {
                        None
                    }
                }),
            );
        } else {
            MaterialDraw::draw(encoder, target, draws.iter());
        }
    }
}

impl<T: Material + Send + Sync> Renderable for MeshNode<T> {
    fn register(device: &Device, _queue: &Queue, resources: &mut Resources) {
        let prepare_mesh_fn = PrepareMeshFn::new(|device, world, resources| {
            for node in world.nodes::<Self>() {
                for primitive in node.primitives.iter() {
                    if resources.contains_key::<MeshBuffers>(&primitive.mesh.id()) {
                        continue;
                    }

                    let mut mesh = if !primitive.mesh.has_attribute(Mesh::NORMAL) {
                        Cow::Owned(primitive.mesh.clone().with_normals())
                    } else {
                        Cow::Borrowed(&primitive.mesh)
                    };

                    if !mesh.has_attribute(Mesh::TANGENT) {
                        mesh = Cow::Owned(mesh.into_owned().with_tangents());
                    }

                    let buffers = MeshBuffers::new(device, &mesh);
                    resources.insert_key(primitive.mesh.id(), buffers);

                    if let Some(aabb) = mesh.aabb() {
                        resources.insert_key(primitive.mesh.id(), aabb);
                    }
                }
            }
        });

        let sample_count = resources.get::<RenderSettings>().unwrap().sample_count;
        let shader_processor = resources.get_mut::<ShaderProcessor>().unwrap();
        let shader_defs = ShaderDefs::default();
        let pipeline =
            MeshNodePipeline::new::<T>(device, &shader_defs, shader_processor, sample_count);

        let hash = shader_defs.hash();

        let mut mesh_node_pipelines = MeshNodePipelines::default();
        mesh_node_pipelines.push(hash, pipeline);

        resources.insert_key(TypeId::of::<T>(), mesh_node_pipelines);
        resources.insert_key(TypeId::of::<T>(), MaterialFunctions::new::<T>());
        resources.insert_key(TypeId::of::<T>(), prepare_mesh_fn);
        resources.insert_key(TypeId::of::<T>(), ShadowFunctions::new_mesh_node::<T>());
    }
}
