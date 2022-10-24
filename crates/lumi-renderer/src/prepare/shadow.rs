use std::{
    num::NonZeroU32,
    ops::{Deref, DerefMut},
};

use lumi_bind::{Bind, Bindings, BindingsLayout};
use lumi_bounds::Aabb;
use lumi_core::{
    CommandEncoder, Device, DrawCommand, Extent3d, IndexFormat, LoadOp, Operations, RenderPass,
    RenderPassDepthStencilAttachment, RenderPassDescriptor, RenderPipelineDescriptor, Resources,
    SharedBindGroup, SharedBuffer, SharedDevice, SharedRenderPipeline, SharedTexture,
    SharedTextureView, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    TextureViewDescriptor, UniformBuffer, VertexBufferLayout, VertexState, VertexStepMode,
};
use lumi_id::{Id, IdMap};
use lumi_mesh::Mesh;
use lumi_shader::{DefaultShader, ShaderProcessor, ShaderRef};
use lumi_util::{math::Mat4, smallvec::SmallVec};
use lumi_world::{DirectionalLight, Extract, ExtractOne, Light, LightId, Node, World};

use crate::{PhaseContext, PreparedMesh, PreparedTransform, RenderPhase};

#[derive(Bind)]
pub struct PreparedShadows {
    pub cascade_texture: SharedTexture,
    #[texture(name = "directional_shadow_maps", dimension = d2_array, sample_type = depth)]
    #[sampler(name = "shadow_map_sampler")]
    pub cascade_view: SharedTextureView,
    pub cascade_view_proj_buffers: Vec<UniformBuffer<Mat4>>,
}

impl PreparedShadows {
    pub const SHADOW_TEXTURE_FORMAT: TextureFormat = TextureFormat::Depth32Float;

    pub fn new(device: &Device, cascade_count: u32) -> Self {
        let cascade_texture = Self::create_cascade_texture(device, cascade_count);
        let cascade_view = cascade_texture.create_view(&Default::default());

        let mut cascade_view_proj_buffers = Vec::with_capacity(cascade_count as usize);
        cascade_view_proj_buffers.resize_with(cascade_count as usize, Default::default);

        Self {
            cascade_texture,
            cascade_view,
            cascade_view_proj_buffers,
        }
    }

    pub fn create_cascade_texture(device: &Device, cascade_count: u32) -> SharedTexture {
        device.create_shared_texture(&TextureDescriptor {
            label: Some("Lumi Shadows Cascade Texture"),
            size: Extent3d {
                width: 2048,
                height: 2048,
                depth_or_array_layers: u32::max(cascade_count, 4),
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: Self::SHADOW_TEXTURE_FORMAT,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
        })
    }

    #[inline]
    pub fn resize_cascades(&mut self, device: &Device, cascade_count: u32) {
        if self.cascade_texture.size().depth_or_array_layers < cascade_count {
            self.cascade_texture = Self::create_cascade_texture(device, cascade_count);
            self.cascade_view = self.cascade_texture.create_view(&Default::default());
            self.cascade_view_proj_buffers
                .resize_with(cascade_count as usize, Default::default);
        }
    }

    #[inline]
    pub fn get_cascade_view(&self, index: u32) -> SharedTextureView {
        self.cascade_texture.create_view(&TextureViewDescriptor {
            label: Some("Lumi Shadows Cascade View"),
            base_array_layer: index,
            array_layer_count: NonZeroU32::new(1),
            ..Default::default()
        })
    }
}

#[derive(Bind)]
pub struct ShadowCasterBindings {
    #[uniform]
    pub view_proj: SharedBuffer,
}

pub struct ShadowPipeline {
    pub bindings_layout: BindingsLayout,
    pub render_pipeline: SharedRenderPipeline,
}

impl ShadowPipeline {
    pub fn new(device: &Device, shader_processor: &mut ShaderProcessor) -> Self {
        let mut vertex_shader = shader_processor
            .process(
                ShaderRef::Default(DefaultShader::ShadowVertex),
                &Default::default(),
            )
            .unwrap();
        vertex_shader.rebind().unwrap();

        let bindings_layout = BindingsLayout::new()
            .with_shader(&vertex_shader)
            .bind::<PreparedTransform>()
            .bind::<ShadowCasterBindings>();

        let pipeline_layout = bindings_layout.create_pipeline_layout(device);

        let render_pipeline = device.create_shared_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Lumi Shadows RenderPipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: vertex_shader.shader_module(device),
                entry_point: "vertex",
                buffers: &[VertexBufferLayout {
                    array_stride: 12,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &lumi_core::vertex_attr_array![0 => Float32x3],
                }],
            },
            fragment: None,
            primitive: Default::default(),
            depth_stencil: Some(lumi_core::DepthStencilState {
                format: PreparedShadows::SHADOW_TEXTURE_FORMAT,
                depth_write_enabled: true,
                depth_compare: lumi_core::CompareFunction::LessEqual,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            multiview: None,
        });

        Self {
            bindings_layout,
            render_pipeline,
        }
    }
}

#[derive(Clone, Copy, Debug, Hash)]
pub enum ShadowTarget {
    Cascade { light: LightId, cascade: u32 },
}

impl ShadowTarget {
    #[inline]
    pub fn id(&self) -> Id<ShadowTarget> {
        Id::from_hash(self)
    }
}

#[derive(Clone, Debug)]
pub struct ShadowDraw {
    pub render_pipeline: SharedRenderPipeline,
    pub bind_groups: SmallVec<[SharedBindGroup; 4]>,
    pub vertex_buffer: SharedBuffer,
    pub index_buffer: Option<SharedBuffer>,
    pub draw_command: DrawCommand,
    pub aabb: Option<Aabb>,
    pub transform: Mat4,
}

impl ShadowDraw {
    #[inline]
    pub fn draw<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        render_pass.set_pipeline(&self.render_pipeline);

        for (i, bind_group) in self.bind_groups.iter().enumerate() {
            render_pass.set_bind_group(i as u32, bind_group, &[]);
        }

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

        if let Some(index_buffer) = &self.index_buffer {
            render_pass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint32);
        }

        self.draw_command.draw(render_pass);
    }
}

pub struct ShadowRenderState {
    pub bindings: Bindings,
}

#[derive(Default)]
pub struct ShadowRenderStates {
    pub states: IdMap<ShadowTarget, ShadowRenderState>,
}

impl Deref for ShadowRenderStates {
    type Target = IdMap<ShadowTarget, ShadowRenderState>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.states
    }
}

impl DerefMut for ShadowRenderStates {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.states
    }
}

pub struct ShadowRenderFunction {
    prepare: fn(&PhaseContext, ShadowTarget, SharedBuffer, &World, &mut Resources),
    render: fn(&PhaseContext, ShadowTarget, &mut Vec<ShadowDraw>, &World, &Resources),
}

impl ShadowRenderFunction {
    #[inline]
    pub fn new<T>() -> Self
    where
        T: Node + Extract<Mesh> + ExtractOne<Mat4>,
    {
        Self {
            prepare: Self::prepare_default::<T>,
            render: Self::render_default::<T>,
        }
    }

    fn prepare_default<T>(
        context: &PhaseContext,
        target: ShadowTarget,
        view_proj: SharedBuffer,
        world: &World,
        resources: &mut Resources,
    ) where
        T: Node + ExtractOne<Mat4>,
    {
        let target_id = target.id();

        let pipeline = if let Some(pipeline) = resources.remove::<ShadowPipeline>() {
            pipeline
        } else {
            let shader_processor = resources.get_mut::<ShaderProcessor>().unwrap();
            ShadowPipeline::new(context.device, shader_processor)
        };

        for (id, _) in context.changes.changed_nodes::<T>(world) {
            let mut states = resources.remove_id_or_default::<ShadowRenderStates>(id);
            let state = states.get_or_insert_with(target_id, || ShadowRenderState {
                bindings: pipeline.bindings_layout.create_bindings(context.device),
            });

            let bindings = &mut state.bindings;

            let caster_bindings = ShadowCasterBindings {
                view_proj: view_proj.clone(),
            };

            let transform = if let Some(transform) = resources.get_id::<PreparedTransform>(id) {
                transform
            } else {
                continue;
            };

            bindings.bind(context.device, context.queue, &caster_bindings);
            bindings.bind(context.device, context.queue, transform);
            bindings.update_bind_groups(context.device);

            resources.insert_id(id.cast(), states);
        }

        resources.insert(pipeline);

        for id in context.changes.removed() {
            resources.remove_id::<ShadowRenderStates>(id);
        }
    }

    fn render_default<T>(
        _context: &PhaseContext,
        target: ShadowTarget,
        draws: &mut Vec<ShadowDraw>,
        world: &World,
        resources: &Resources,
    ) where
        T: Node + Extract<Mesh> + ExtractOne<Mat4>,
    {
        let pipeline = resources.get::<ShadowPipeline>().unwrap();

        let target_id = target.id();

        for (id, node) in world.iter_nodes::<T>() {
            let states = resources.get_id::<ShadowRenderStates>(id).unwrap();
            let state = states.get(target_id).unwrap();
            let bindings = &state.bindings;

            let transform = if let Some(&transform) = node.extract_one() {
                transform
            } else {
                continue;
            };

            let mut i = 0;
            node.extract(&mut |mesh| {
                let mesh_id = mesh.id();

                let prepared_mesh = resources.get_id::<PreparedMesh>(mesh_id).unwrap();
                let aabb = resources.get_id::<Aabb>(mesh_id).cloned();

                let vertex_buffer =
                    if let Some(vertex_buffer) = prepared_mesh.attributes.get(Mesh::POSITION) {
                        vertex_buffer.clone()
                    } else {
                        return;
                    };

                let draw = ShadowDraw {
                    render_pipeline: pipeline.render_pipeline.clone(),
                    bind_groups: bindings.bind_groups().cloned().collect(),
                    vertex_buffer,
                    index_buffer: prepared_mesh.indices.clone(),
                    draw_command: mesh.draw_command(),
                    aabb,
                    transform,
                };

                draws.push(draw);

                i += 1;
            });
        }
    }

    #[inline]
    pub fn prepare(
        &self,
        context: &PhaseContext,
        target: ShadowTarget,
        view_proj: SharedBuffer,
        world: &World,
        resources: &mut Resources,
    ) {
        (self.prepare)(context, target, view_proj, world, resources);
    }

    #[inline]
    pub fn render(
        &self,
        context: &PhaseContext,
        target: ShadowTarget,
        draws: &mut Vec<ShadowDraw>,
        world: &World,
        resources: &Resources,
    ) {
        (self.render)(context, target, draws, world, resources);
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PrepareShadows;

impl RenderPhase for PrepareShadows {
    fn prepare(&mut self, context: &PhaseContext, world: &World, resources: &mut Resources) {
        let mut cascade_count = 0;
        for light in world.lights() {
            if let Light::Directional(directional) = light {
                if directional.shadows {
                    cascade_count += 4;
                }
            }
        }

        let mut prepared_shadows = if let Some(shadows) = resources.remove::<PreparedShadows>() {
            shadows
        } else {
            PreparedShadows::new(context.device, cascade_count)
        };

        let mut cascade_index = 0;
        for (id, light) in world.iter_lights() {
            match light {
                Light::Directional(directional) => {
                    if !directional.shadows {
                        continue;
                    }

                    for cascade in 0..DirectionalLight::CASCADES {
                        let target = ShadowTarget::Cascade { light: id, cascade };
                        let view_proj =
                            &mut prepared_shadows.cascade_view_proj_buffers[cascade_index];

                        view_proj.set(directional.view_proj(cascade));

                        let view_proj_buffer = view_proj.buffer(context.device, context.queue);

                        resources.scope(
                            |resources: &mut Resources, render_functions: &mut IdMap<ShadowRenderFunction>| {
                                for render_function in render_functions.values() {
                                    render_function.prepare(context, target, view_proj_buffer.clone(), world, resources);
                                }
                            },
                        );

                        cascade_index += 1;
                    }
                }
                _ => {}
            }
        }

        resources.insert(prepared_shadows);
    }

    fn render(
        &self,
        context: &PhaseContext,
        encoder: &mut CommandEncoder,
        world: &World,
        resources: &Resources,
    ) {
        let prepared_shadow = resources.get::<PreparedShadows>().unwrap();

        let mut cascade_index = 0;
        for (id, light) in world.iter_lights() {
            match light {
                Light::Directional(directional) => {
                    if !directional.shadows {
                        continue;
                    }

                    for cascade in 0..DirectionalLight::CASCADES {
                        let target = ShadowTarget::Cascade { light: id, cascade };
                        let frustum = directional.cascade_frustum(cascade);

                        let mut draws = Vec::new();
                        for render_function in resources.values_id::<ShadowRenderFunction>() {
                            render_function.render(context, target, &mut draws, world, resources);
                        }

                        draws.retain(|draw| {
                            if let Some(aabb) = &draw.aabb {
                                frustum.intersects_shape(aabb, draw.transform)
                            } else {
                                true
                            }
                        });

                        let view = prepared_shadow.get_cascade_view(cascade_index);

                        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                            label: Some("Shadow Pass"),
                            color_attachments: &[],
                            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                                view: &view,
                                depth_ops: Some(Operations {
                                    load: LoadOp::Clear(1.0),
                                    store: true,
                                }),
                                stencil_ops: None,
                            }),
                        });

                        for draw in draws.iter() {
                            draw.draw(&mut render_pass);
                        }

                        cascade_index += 1;
                    }
                }
                _ => {}
            }
        }
    }
}
