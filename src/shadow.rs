use std::{
    collections::{HashMap, LinkedList},
    hash::{Hash, Hasher},
    num::NonZeroU32,
};

use glam::Mat4;
use smallvec::SmallVec;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages, CommandEncoder, CompareFunction, DepthStencilState, Device, Extent3d,
    FilterMode, IndexFormat, LoadOp, Operations, RenderPassDepthStencilAttachment,
    RenderPassDescriptor, RenderPipelineDescriptor, SamplerDescriptor, TextureDescriptor,
    TextureDimension, TextureFormat, TextureUsages, TextureViewDescriptor, TextureViewDimension,
    VertexAttribute, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
};

use crate::{
    aabb::{Aabb, RenderFrustum},
    bind::Bind,
    binding::{Bindings, BindingsLayout},
    id::{BufferId, LightId, RenderPipelineId},
    light::Light,
    material::{DrawCommand, Material, MeshNode},
    mesh::{Mesh, MeshBuffers},
    prelude::World,
    renderer::{PhaseContext, RenderPhase},
    resources::Resources,
    shader::{DefaultShader, ShaderProcessor, ShaderRef},
    SharedBindGroup, SharedBuffer, SharedDevice, SharedRenderPipeline, SharedSampler,
    SharedTexture, SharedTextureView,
};

pub struct ShadowMaps {
    pub directional: SharedTexture,
    pub directional_view: SharedTextureView,
    pub cascades: HashMap<LightId, u32>,
}

impl ShadowMaps {
    pub fn new(device: &Device) -> Self {
        let directional = device.create_shared_texture(&TextureDescriptor {
            label: Some("Directional Shadow Map"),
            size: Extent3d {
                width: 2048,
                height: 2048,
                depth_or_array_layers: 4,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
        });

        let directional_view = directional.create_view(&TextureViewDescriptor {
            dimension: Some(TextureViewDimension::D2Array),
            ..Default::default()
        });

        Self {
            directional,
            directional_view,
            cascades: HashMap::default(),
        }
    }

    pub fn get(&self, target: ShadowTarget) -> SharedTextureView {
        match target {
            ShadowTarget::Directional { cascade, .. } => {
                self.directional.create_view(&TextureViewDescriptor {
                    dimension: Some(TextureViewDimension::D2),
                    base_array_layer: cascade,
                    array_layer_count: NonZeroU32::new(1),
                    ..Default::default()
                })
            }
            _ => unimplemented!(),
        }
    }

    pub fn resize_directional(&mut self, device: &Device, cascades: u32) {
        let cascades = cascades.max(4);

        if cascades == self.directional.size().depth_or_array_layers {
            return;
        }

        self.directional = device.create_shared_texture(&TextureDescriptor {
            label: Some("Directional Shadow Map"),
            size: Extent3d {
                width: 2048,
                height: 2048,
                depth_or_array_layers: cascades,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
        });
        self.directional_view = self.directional.create_view(&TextureViewDescriptor {
            dimension: Some(TextureViewDimension::D2Array),
            ..Default::default()
        });
    }
}

struct DefaultShadowPipeline {
    bindings_layout: BindingsLayout,
    pipeline: SharedRenderPipeline,
}

impl DefaultShadowPipeline {
    pub fn new(device: &Device, shader_processor: &mut ShaderProcessor) -> Self {
        let mut vertex = shader_processor
            .process(
                ShaderRef::Default(DefaultShader::ShadowVertex),
                &Default::default(),
            )
            .unwrap();
        vertex.rebind().unwrap();

        let bindings_layout = BindingsLayout::new()
            .with_shader(&vertex)
            .bind::<ShadowCasterBindings>();

        let pipeline_layout = bindings_layout.create_pipeline_layout(device);

        let pipeline = device.create_shared_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: vertex.shader_module(device),
                entry_point: "vertex",
                buffers: &[
                    VertexBufferLayout {
                        array_stride: 12,
                        step_mode: VertexStepMode::Vertex,
                        attributes: &[VertexAttribute {
                            offset: 0,
                            format: VertexFormat::Float32x3,
                            shader_location: 0,
                        }],
                    },
                    VertexBufferLayout {
                        array_stride: 64,
                        step_mode: VertexStepMode::Instance,
                        attributes: &[
                            VertexAttribute {
                                offset: 0,
                                format: VertexFormat::Float32x4,
                                shader_location: 1,
                            },
                            VertexAttribute {
                                offset: 16,
                                format: VertexFormat::Float32x4,
                                shader_location: 2,
                            },
                            VertexAttribute {
                                offset: 32,
                                format: VertexFormat::Float32x4,
                                shader_location: 3,
                            },
                            VertexAttribute {
                                offset: 48,
                                format: VertexFormat::Float32x4,
                                shader_location: 4,
                            },
                        ],
                    },
                ],
            },
            fragment: None,
            primitive: Default::default(),
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Less,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            multiview: None,
        });

        Self {
            bindings_layout,
            pipeline,
        }
    }
}

#[derive(Hash)]
struct InstanceKey {
    pipeline: RenderPipelineId,
    vertex_buffers: LinkedList<(u32, BufferId)>,
    index_buffer: Option<BufferId>,
    draw: DrawCommand,
}

impl InstanceKey {
    fn hash(&self) -> u64 {
        let mut hasher = ahash::AHasher::default();
        <Self as Hash>::hash(self, &mut hasher);
        hasher.finish()
    }
}

pub struct ShadowDraw {
    pub pipeline: SharedRenderPipeline,
    pub bind_groups: SmallVec<[SharedBindGroup; 4]>,
    pub vertex_buffers: SmallVec<[(u32, SharedBuffer); 2]>,
    pub index_buffer: Option<SharedBuffer>,
    pub draw_command: DrawCommand,
    pub aabb: Option<Aabb>,
    pub transform: Mat4,
}

impl ShadowDraw {
    pub fn instance_key(&self) -> u64 {
        let instance_key = InstanceKey {
            pipeline: self.pipeline.id(),
            vertex_buffers: self
                .vertex_buffers
                .iter()
                .map(|(stride, buffer)| (*stride, buffer.id()))
                .collect(),
            index_buffer: self.index_buffer.as_ref().map(|buffer| buffer.id()),
            draw: self.draw_command,
        };

        instance_key.hash()
    }
}

#[derive(Bind)]
pub struct ShadowReceiverBindings {
    #[texture(dimension = d2_array, sample_type = depth)]
    pub directional_shadow_maps: SharedTextureView,
    #[sampler]
    pub shadow_map_sampler: SharedSampler,
}

#[derive(Bind)]
pub struct ShadowCasterBindings {
    #[uniform]
    pub view_proj: Mat4,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ShadowTarget {
    Directional { view_proj: Mat4, cascade: u32 },
    Point { view_proj: Mat4, face: u32 },
}

pub struct ShadowFunctions {
    prepare: fn(&PhaseContext, &World, &RenderFrustum, LightId, ShadowTarget, &mut Resources),
    render: fn(
        &PhaseContext,
        &World,
        &RenderFrustum,
        LightId,
        ShadowTarget,
        &Resources,
        &mut Vec<ShadowDraw>,
    ),
}

impl ShadowFunctions {
    pub fn new(
        prepare: fn(&PhaseContext, &World, &RenderFrustum, LightId, ShadowTarget, &mut Resources),
        render: fn(
            &PhaseContext,
            &World,
            &RenderFrustum,
            LightId,
            ShadowTarget,
            &Resources,
            &mut Vec<ShadowDraw>,
        ),
    ) -> Self {
        Self { prepare, render }
    }

    pub fn new_mesh_node<T: Material + Send + Sync>() -> Self {
        Self::new(
            |context, world, frustum, id, target, resources| {
                let pipeline = resources.remove::<DefaultShadowPipeline>().unwrap();

                match target {
                    ShadowTarget::Directional { view_proj, cascade } => {
                        for (node_id, node) in world.iter_nodes::<MeshNode<T>>() {
                            if !node.inside_render_frustum(frustum, resources) {
                                continue;
                            }

                            let bindings_id = (id, cascade, node_id);

                            if !resources.contains_key::<Bindings>(&bindings_id) {
                                let bindings =
                                    pipeline.bindings_layout.create_bindings(context.device);
                                resources.insert_key(bindings_id, bindings);
                            }

                            let bindings = resources.get_key_mut::<Bindings>(&bindings_id).unwrap();

                            let shadow_bindings = ShadowCasterBindings { view_proj };

                            bindings.bind(&context.device, &context.queue, &shadow_bindings);
                            bindings.update_bind_groups(&context.device);
                        }
                    }
                    _ => unimplemented!(),
                }

                resources.insert(pipeline);
            },
            |_, world, frustum, id, target, resources, draws| {
                let pipeline = resources.get::<DefaultShadowPipeline>().unwrap();

                match target {
                    ShadowTarget::Directional { cascade, .. } => {
                        for (node_id, node) in world.iter_nodes::<MeshNode<T>>() {
                            let bindings_id = (id, cascade, node_id);
                            let bindings = if let Some(bindings) =
                                resources.get_key::<Bindings>(&bindings_id)
                            {
                                bindings
                            } else {
                                continue;
                            };

                            for primitive in node.primitives.iter() {
                                let aabb = resources.get_key::<Aabb>(&primitive.mesh.id()).cloned();

                                if let Some(aabb) = aabb {
                                    if !frustum.should_render(&aabb, node.transform) {
                                        continue;
                                    }
                                }

                                let mesh_buffers = resources
                                    .get_key::<MeshBuffers>(&primitive.mesh.id())
                                    .unwrap();

                                let mut vertex_buffers = SmallVec::new();
                                vertex_buffers.push((
                                    0,
                                    mesh_buffers.attributes.get(Mesh::POSITION).unwrap().clone(),
                                ));

                                let draw = ShadowDraw {
                                    pipeline: pipeline.pipeline.clone(),
                                    bind_groups: bindings.bind_groups().cloned().collect(),
                                    vertex_buffers,
                                    index_buffer: mesh_buffers.index_buffer.clone(),
                                    draw_command: primitive.mesh.draw_command(),
                                    aabb,
                                    transform: node.transform,
                                };

                                draws.push(draw);
                            }
                        }
                    }
                    _ => unimplemented!(),
                }
            },
        )
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ShadowPhase;

impl RenderPhase for ShadowPhase {
    fn prepare(&mut self, context: &PhaseContext, world: &World, resources: &mut Resources) {
        if !resources.contains::<DefaultShadowPipeline>() {
            let pipeline = DefaultShadowPipeline::new(context.device, resources.get_mut().unwrap());
            resources.insert(pipeline);
        }

        let functions = resources.remove_keyed::<ShadowFunctions>();
        let shadow_maps = resources
            .remove()
            .unwrap_or_else(|| ShadowMaps::new(context.device));

        for (light_id, light) in world.iter_lights() {
            if !light.shadows() {
                continue;
            }

            match light {
                Light::Directional(directional) => {
                    for cascade in 0..4u32 {
                        let view_proj = directional.view_proj(cascade);

                        let target = ShadowTarget::Directional {
                            view_proj,
                            cascade: shadow_maps.cascades[&light_id] + cascade,
                        };

                        let frustum = directional.render_frustum(cascade);
                        for function in functions.values() {
                            (function.prepare)(
                                context, world, &frustum, light_id, target, resources,
                            );
                        }
                    }
                }
                _ => {}
            }
        }

        let directional_view = shadow_maps.directional_view.clone();

        if let Some(bindings) = resources.get_mut::<ShadowReceiverBindings>() {
            bindings.directional_shadow_maps = directional_view;
        } else {
            let sampler = context.device.create_shared_sampler(&SamplerDescriptor {
                min_filter: FilterMode::Linear,
                mag_filter: FilterMode::Linear,
                ..Default::default()
            });

            let bindings = ShadowReceiverBindings {
                directional_shadow_maps: directional_view,
                shadow_map_sampler: sampler,
            };
            resources.insert(bindings);
        }

        resources.insert(functions);
        resources.insert(shadow_maps);
    }

    fn render(
        &self,
        context: &PhaseContext,
        encoder: &mut CommandEncoder,
        world: &World,
        resources: &Resources,
    ) {
        let functions = resources.get_keyed::<ShadowFunctions>().unwrap();

        for (light_id, light) in world.iter_lights() {
            if !light.shadows() {
                continue;
            }

            match light {
                Light::Directional(directional) => {
                    for cascade in 0..4u32 {
                        let view_proj = directional.view_proj(cascade);

                        let shadow_maps = resources.get::<ShadowMaps>().unwrap();

                        let target = ShadowTarget::Directional {
                            view_proj,
                            cascade: shadow_maps.cascades[&light_id] + cascade,
                        };
                        let shadow_map = shadow_maps.get(target);

                        let mut draws = Vec::new();

                        let frustum = directional.render_frustum(cascade);
                        for function in functions.values() {
                            (function.render)(
                                context, world, &frustum, light_id, target, resources, &mut draws,
                            );
                        }

                        struct Instance {
                            pipeline: SharedRenderPipeline,
                            bind_groups: SmallVec<[SharedBindGroup; 4]>,
                            vertex_buffers: SmallVec<[(u32, SharedBuffer); 2]>,
                            index_buffer: Option<SharedBuffer>,
                            draw_command: DrawCommand,
                            transforms: Vec<Mat4>,
                        }

                        let mut instance_buffers = HashMap::<u64, Instance>::new();

                        for draw in draws {
                            let instance_key = draw.instance_key();

                            let instances =
                                instance_buffers
                                    .entry(instance_key)
                                    .or_insert_with(|| Instance {
                                        pipeline: draw.pipeline,
                                        bind_groups: draw.bind_groups,
                                        vertex_buffers: draw.vertex_buffers,
                                        index_buffer: draw.index_buffer,
                                        draw_command: draw.draw_command,
                                        transforms: Vec::new(),
                                    });

                            instances.transforms.push(draw.transform);
                        }

                        let instances = instance_buffers
                            .iter()
                            .map(|(&key, instance)| {
                                let buffer =
                                    context.device.create_buffer_init(&BufferInitDescriptor {
                                        label: Some("Shadow Instance Buffer"),
                                        contents: bytemuck::cast_slice(&instance.transforms),
                                        usage: BufferUsages::VERTEX,
                                    });

                                (key, buffer)
                            })
                            .collect::<HashMap<u64, Buffer>>();

                        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                            label: None,
                            color_attachments: &[],
                            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                                view: &shadow_map,
                                depth_ops: Some(Operations {
                                    load: LoadOp::Clear(1.0),
                                    store: true,
                                }),
                                stencil_ops: None,
                            }),
                        });

                        for (key, instance) in instance_buffers.iter() {
                            render_pass.set_pipeline(&instance.pipeline);

                            for (i, bind_group) in instance.bind_groups.iter().enumerate() {
                                render_pass.set_bind_group(i as u32, bind_group, &[]);
                            }

                            for (i, buffer) in instance.vertex_buffers.iter() {
                                render_pass.set_vertex_buffer(*i, buffer.slice(..));
                            }

                            render_pass.set_vertex_buffer(1, instances[key].slice(..));

                            if let Some(index_buffer) = &instance.index_buffer {
                                render_pass
                                    .set_index_buffer(index_buffer.slice(..), IndexFormat::Uint32);
                            }

                            instance
                                .draw_command
                                .draw_instanced(&mut render_pass, instance.transforms.len() as u32);
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
