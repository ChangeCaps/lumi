use std::{
    num::NonZeroU32,
    ops::{Deref, DerefMut},
};

use lumi_bind::{Bind, Bindings, BindingsLayout};
use lumi_bounds::CascadeFrustum;
use lumi_core::{
    CommandEncoder, Device, Extent3d, IndexFormat, LoadOp, Operations, Queue,
    RenderPassDepthStencilAttachment, RenderPassDescriptor, RenderPipelineDescriptor, SharedBuffer,
    SharedDevice, SharedRenderPipeline, SharedTexture, SharedTextureView, TextureDescriptor,
    TextureDimension, TextureFormat, TextureUsages, TextureViewDescriptor, UniformBuffer,
    VertexBufferLayout, VertexState, VertexStepMode,
};
use lumi_id::{Id, IdMap};
use lumi_mesh::Mesh;
use lumi_shader::{DefaultShader, ShaderProcessor, ShaderRef};
use lumi_util::math::Mat4;
use shiv::{
    query::{Changed, Query, Without},
    system::{Commands, Res, ResInit, ResMut, ResMutInit},
    world::{Component, Entity, FromWorld, World},
};
use shiv_transform::GlobalTransform;

use crate::{
    DirectionalLight, Extract, ExtractedMeshes, PreparedLights, PreparedMeshes, PreparedTransform,
    RenderDevice, RenderQueue,
};

#[derive(Bind)]
pub struct PreparedShadows {
    pub cascade_texture: SharedTexture,
    #[texture(name = "directional_shadow_maps", dimension = d2_array, sample_type = depth)]
    #[sampler(name = "shadow_map_sampler")]
    pub cascade_view: SharedTextureView,
    pub cascade_view_proj_buffers: Vec<UniformBuffer<Mat4>>,
    pub cascade_frustums: Vec<CascadeFrustum>,
    pub bindings_changed: bool,
}

impl FromWorld for PreparedShadows {
    fn from_world(world: &mut World) -> Self {
        let device = world.resource::<RenderDevice>();

        Self::new(device, 0)
    }
}

impl PreparedShadows {
    pub const SHADOW_TEXTURE_FORMAT: TextureFormat = TextureFormat::Depth32Float;

    pub fn new(device: &Device, cascade_count: u32) -> Self {
        let cascade_count = u32::max(cascade_count, 4);

        let cascade_texture = Self::create_cascade_texture(device, cascade_count);
        let cascade_view = cascade_texture.create_view(&Default::default());

        let mut cascade_view_proj_buffers = Vec::with_capacity(cascade_count as usize);
        cascade_view_proj_buffers.resize_with(cascade_count as usize, Default::default);

        let mut cascade_frustums = Vec::with_capacity(cascade_count as usize);
        cascade_frustums.resize_with(cascade_count as usize, Default::default);

        Self {
            cascade_texture,
            cascade_view,
            cascade_view_proj_buffers,
            cascade_frustums,
            bindings_changed: true,
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
        if self.cascade_texture.size().depth_or_array_layers < u32::max(cascade_count, 4) {
            self.cascade_texture = Self::create_cascade_texture(device, cascade_count);
            self.cascade_view = self.cascade_texture.create_view(&Default::default());
            self.cascade_view_proj_buffers
                .resize_with(cascade_count as usize, Default::default);

            self.bindings_changed = true;
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

    #[inline]
    pub fn get_target_view(&self, target: &ShadowTarget) -> SharedTextureView {
        match target.kind {
            ShadowKind::Directional => self.get_cascade_view(target.index),
        }
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

impl FromWorld for ShadowPipeline {
    fn from_world(world: &mut World) -> Self {
        let mut shader_processor = world.resource_mut::<ShaderProcessor>();

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

        let device = world.resource::<RenderDevice>();
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
pub enum ShadowKind {
    Directional,
}

#[derive(Clone, Copy, Debug, Hash)]
pub struct ShadowTarget {
    pub kind: ShadowKind,
    pub entity: Entity,
    pub index: u32,
}

impl ShadowTarget {
    #[inline]
    pub fn id(&self) -> Id<Self> {
        Id::from_hash(self)
    }
}

#[derive(Clone, Debug, Default)]
pub struct ShadowTargets {
    pub targets: Vec<ShadowTarget>,
}

impl Deref for ShadowTargets {
    type Target = Vec<ShadowTarget>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.targets
    }
}

impl DerefMut for ShadowTargets {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.targets
    }
}

#[derive(Component, Default)]
pub struct ShadowRenderState {
    pub bindings: IdMap<ShadowTarget, Bindings>,
    pub transform: Mat4,
}

pub fn insert_state_system(
    mut commands: Commands,
    query: Query<Entity, Without<ShadowRenderState>>,
) {
    for entity in query.iter() {
        commands.entity(entity).insert(ShadowRenderState::default());
    }
}

pub fn extract_directional_shadow_system(
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    shadow_pipeline: ResInit<ShadowPipeline>,
    mut prepared_shadows: ResMutInit<PreparedShadows>,
    mut shadow_targets: ResMut<ShadowTargets>,
    prepared_lights: Res<PreparedLights>,
    light_query: Extract<Query<(Entity, &DirectionalLight, Option<&GlobalTransform>)>>,
    mut prepared_query: Query<
        (&PreparedTransform, &mut ShadowRenderState),
        Changed<PreparedTransform>,
    >,
) {
    shadow_targets.clear();

    prepared_shadows.bindings_changed = false;
    prepared_shadows.resize_cascades(&device, prepared_lights.next_cascade_index);

    let mut cascade_index = 0;
    for (entity, light, transform) in light_query.iter() {
        if !light.shadows {
            continue;
        }

        let transform = transform.copied().unwrap_or_default();
        let mut light = light.clone();

        light.direction = transform.matrix.mul_vec3(light.direction);
        light.direction = light.direction.normalize_or_zero();

        for cascade in 0..DirectionalLight::CASCADES {
            let target = ShadowTarget {
                kind: ShadowKind::Directional,
                entity,
                index: cascade_index,
            };

            shadow_targets.push(target);

            let frustum = light.cascade_frustum(cascade);
            prepared_shadows.cascade_frustums[cascade_index as usize] = frustum;

            let view_proj = &mut prepared_shadows.cascade_view_proj_buffers[cascade_index as usize];
            view_proj.set(light.view_proj(cascade));

            let caster_bindings = ShadowCasterBindings {
                view_proj: view_proj.buffer(&device, &queue),
            };

            prepare_target(
                &device,
                &queue,
                &caster_bindings,
                &shadow_pipeline,
                target,
                &mut prepared_query,
            );

            cascade_index += 1;
        }
    }
}

fn prepare_target(
    device: &Device,
    queue: &Queue,
    caster_bindings: &ShadowCasterBindings,
    shadow_pipeline: &ShadowPipeline,
    target: ShadowTarget,
    prepared_query: &mut Query<
        (&PreparedTransform, &mut ShadowRenderState),
        Changed<PreparedTransform>,
    >,
) {
    let target_id = target.id();

    for (transform, mut state) in prepared_query.iter_mut() {
        let bindings = state.bindings.get_or_insert_with(target_id, || {
            shadow_pipeline.bindings_layout.create_bindings(&device)
        });

        bindings.bind(device, queue, caster_bindings);
        bindings.bind(device, queue, transform);

        bindings.update_bind_groups(device);

        state.transform = transform.transform;
    }
}

pub fn render_shadow_system(
    mut encoder: ResMut<CommandEncoder>,
    prepared_meshes: Res<PreparedMeshes>,
    prepared_shadows: Res<PreparedShadows>,
    shadow_pipeline: Res<ShadowPipeline>,
    shadow_targets: Res<ShadowTargets>,
    render_query: Query<(&ExtractedMeshes, &ShadowRenderState)>,
) {
    for target in shadow_targets.iter() {
        let target_id = target.id();
        let target_view = prepared_shadows.get_target_view(target);

        let frustum = prepared_shadows.cascade_frustums[target.index as usize];

        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Lumi Shadow Pass"),
            color_attachments: &[],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: &target_view,
                depth_ops: Some(Operations {
                    load: LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        render_pass.set_pipeline(&shadow_pipeline.render_pipeline);

        for (meshes, state) in render_query.iter() {
            let bindings = if let Some(state) = state.bindings.get(target_id) {
                state
            } else {
                continue;
            };

            for mesh_id in meshes.iter().copied() {
                let prepared_mesh = prepared_meshes.get(mesh_id).unwrap();

                if let Some(aabb) = prepared_mesh.aabb {
                    if !frustum.intersects_shape(&aabb, state.transform) {
                        continue;
                    }
                }

                bindings.apply(&mut render_pass);

                if let Some(vertex_buffer) = prepared_mesh.attributes.get(Mesh::POSITION) {
                    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                }

                if let Some(index_buffer) = prepared_mesh.indices.as_ref() {
                    render_pass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint32);
                }

                prepared_mesh.draw.draw(&mut render_pass);
            }
        }
    }
}
