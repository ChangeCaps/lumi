use std::{
    cmp::Ordering,
    ops::{Deref, DerefMut},
};

use lumi_bind::{Bind, Bindings};
use lumi_bounds::Aabb;
use lumi_core::{CommandEncoder, Device, Extent3d, Resources, SharedTextureView};
use lumi_id::{Id, IdMap};
use lumi_renderer::{
    CameraBindings, IntegratedBrdf, MipChain, MipChainPipeline, PreparedEnvironment,
    PreparedLights, PreparedMesh, PreparedShadows, PreparedTransform, RenderViewPhase,
    ViewPhaseContext,
};
use lumi_shader::ShaderProcessor;
use lumi_util::{math::Mat4, smallvec::SmallVec};
use lumi_world::{Extract, ExtractOne, Node, World};

use crate::{
    Material, MaterialDraw, PreparedMaterialPipeline, PreparedMaterialPipelineKey, Primitive,
};

#[derive(Bind)]
pub struct SsrBindings {
    #[texture]
    #[sampler(name = "ssr_sampler")]
    pub ssr_texture: SharedTextureView,
}

pub struct Ssr {
    pub mip_chain: MipChain,
}

impl Ssr {
    pub fn new(device: &Device, pipeline: &MipChainPipeline, size: Extent3d) -> Self {
        let mip_chain = MipChain::new(device, &pipeline.down_layout, size.width, size.height, None);

        Self { mip_chain }
    }

    #[inline]
    pub fn bindings(&self) -> SsrBindings {
        SsrBindings {
            ssr_texture: self.mip_chain.view.clone(),
        }
    }
}

pub struct MaterialState {
    pub bindings: Bindings,
    pub pipeline_id: Id<PreparedMaterialPipeline>,
}

#[derive(Default)]
pub struct MaterialStates {
    pub bindings: SmallVec<[MaterialState; 4]>,
}

impl Deref for MaterialStates {
    type Target = SmallVec<[MaterialState; 4]>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.bindings
    }
}

impl DerefMut for MaterialStates {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bindings
    }
}

pub struct MaterialRenderFunction {
    prepare: fn(&ViewPhaseContext, &World, &mut Resources),
    render: fn(&ViewPhaseContext, &mut Vec<MaterialDraw>, &World, &Resources),
}

impl MaterialRenderFunction {
    #[inline]
    pub fn new<T, U>() -> Self
    where
        T: Node + Extract<Primitive<U>> + ExtractOne<Mat4>,
        U: Material,
    {
        Self {
            prepare: Self::prepare_mesh_node::<T, U>,
            render: Self::render_mesh_node::<T, U>,
        }
    }

    fn prepare_mesh_node<T, U>(context: &ViewPhaseContext, world: &World, resources: &mut Resources)
    where
        T: Node + Extract<Primitive<U>> + ExtractOne<Mat4>,
        U: Material,
    {
        resources.register::<IntegratedBrdf>();

        for (id, node) in context.changes.changed_nodes::<T>(world) {
            let mut states = resources.remove_id_or_default::<MaterialStates>(id);

            node.extract_enumerated(&mut |i, primitive| {
                let key = PreparedMaterialPipelineKey::new(&primitive.material);
                let pipeline_id = key.id();

                let pipeline = if let Some(pipeline) =
                    resources.get_id::<PreparedMaterialPipeline>(pipeline_id)
                {
                    pipeline
                } else {
                    let shader_processor = resources.get_mut::<ShaderProcessor>().unwrap();

                    let pipeline = PreparedMaterialPipeline::new::<U>(
                        context.device,
                        &key.shader_defs,
                        shader_processor,
                        context.target.sample_count(),
                    );

                    resources.insert_id(pipeline_id, pipeline);

                    resources
                        .get_id::<PreparedMaterialPipeline>(pipeline_id)
                        .unwrap()
                };

                let new_bindings = |resources: &Resources| {
                    let mut new_bindings = pipeline.bindings_layout.create_bindings(context.device);

                    let camera_bindings = CameraBindings {
                        camera: context.view.camera_buffer.clone(),
                    };

                    let prepared_lights = resources.get::<PreparedLights>().unwrap();
                    let prepared_environment = resources.get::<PreparedEnvironment>().unwrap();
                    let prepared_shadows = resources.get::<PreparedShadows>().unwrap();
                    let prepared_transforms = resources.get_id::<PreparedTransform>(id).unwrap();
                    let integrated_brdf = resources.get::<IntegratedBrdf>().unwrap();
                    let ssr = resources.get_id::<Ssr>(context.view.camera).unwrap();

                    new_bindings.bind(context.device, context.queue, &camera_bindings);
                    new_bindings.bind(context.device, context.queue, prepared_lights);
                    new_bindings.bind(context.device, context.queue, prepared_environment);
                    new_bindings.bind(context.device, context.queue, prepared_shadows);
                    new_bindings.bind(context.device, context.queue, prepared_transforms);
                    new_bindings.bind(context.device, context.queue, integrated_brdf);
                    new_bindings.bind(context.device, context.queue, &ssr.bindings());

                    new_bindings
                };

                if states.len() <= i {
                    let state = MaterialState {
                        bindings: new_bindings(resources),
                        pipeline_id,
                    };

                    states.push(state);
                }

                let state = &mut states[i];

                if state.pipeline_id != pipeline_id {
                    state.bindings = new_bindings(resources);
                    state.pipeline_id = pipeline_id;
                }

                state
                    .bindings
                    .bind(context.device, context.queue, &primitive.material);
                state.bindings.update_bind_groups(context.device);
            });

            resources.insert_id(id.cast(), states);
        }
    }

    fn render_mesh_node<T, U>(
        _context: &ViewPhaseContext,
        draws: &mut Vec<MaterialDraw>,
        world: &World,
        resources: &Resources,
    ) where
        T: Node + Extract<Primitive<U>> + ExtractOne<Mat4>,
        U: Material,
    {
        for (id, node) in world.iter_nodes::<T>() {
            let states = resources.get_id::<MaterialStates>(id).unwrap();

            let transform = if let Some(&transform) = node.extract_one() {
                transform
            } else {
                continue;
            };

            node.extract_enumerated(&mut |i, primitive| {
                let state = &states[i];
                let mesh_id = primitive.mesh.id();

                let pipeline = resources
                    .get_id::<PreparedMaterialPipeline>(state.pipeline_id)
                    .unwrap();

                let prepared_mesh = resources.get_id::<PreparedMesh>(mesh_id).unwrap();
                let aabb = resources.get_id::<Aabb>(mesh_id).copied();

                let mut vertex_buffers = SmallVec::new();
                for vertex_layout in pipeline.material_pipeline.vertices.iter() {
                    let buffer = prepared_mesh
                        .attributes
                        .get(vertex_layout.attribute.as_ref())
                        .unwrap();

                    vertex_buffers.push((vertex_layout.location, buffer.clone()));
                }

                let bind_groups = state.bindings.bind_groups().cloned().collect();

                let render_pipeline = if primitive.material.is_translucent() {
                    pipeline.transparent_pipeline.clone()
                } else {
                    pipeline.opaque_pipeline.clone()
                };

                let draw = MaterialDraw {
                    prepass_pipeline: pipeline.prepass_pipeline.clone(),
                    render_pipeline,
                    bind_groups,
                    vertex_buffers,
                    index_buffer: prepared_mesh.indices.clone(),
                    draw_command: primitive.mesh.draw_command(),
                    ssr: primitive.material.use_ssr(),
                    aabb,
                    transform,
                };

                draws.push(draw);
            });
        }
    }

    #[inline]
    pub fn prepare(&self, context: &ViewPhaseContext, world: &World, resources: &mut Resources) {
        (self.prepare)(context, world, resources)
    }

    #[inline]
    pub fn render(
        &self,
        context: &ViewPhaseContext,
        draws: &mut Vec<MaterialDraw>,
        world: &World,
        resources: &Resources,
    ) {
        (self.render)(context, draws, world, resources)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct RenderMaterials {
    sample_count: u32,
}

impl RenderViewPhase for RenderMaterials {
    fn prepare(&mut self, context: &ViewPhaseContext, world: &World, resources: &mut Resources) {
        if self.sample_count != context.target.sample_count() {
            self.sample_count = context.target.sample_count();

            for pipeline in resources.values_id_mut::<PreparedMaterialPipeline>() {
                pipeline.recreate_pipeline(context.device, self.sample_count);
            }
        }

        let pipeline = resources.remove::<MipChainPipeline>().unwrap();

        if let Some(ssr) = resources.get_id_mut::<Ssr>(context.view.camera) {
            if ssr.mip_chain.size() != context.target.size() {
                *ssr = Ssr::new(context.device, &pipeline, context.target.size());

                ssr.mip_chain.prepare_downsample_bindings(
                    context.device,
                    context.queue,
                    &context.target.hdr_view,
                    4.0,
                );

                let bindings = ssr.bindings();

                for states in resources.values_id_mut::<MaterialStates>() {
                    for state in states.iter_mut() {
                        state
                            .bindings
                            .bind(context.device, context.queue, &bindings);
                        state.bindings.update_bind_groups(context.device);
                    }
                }
            }
        } else {
            let mut ssr = Ssr::new(context.device, &pipeline, context.target.size());

            ssr.mip_chain.prepare_downsample_bindings(
                context.device,
                context.queue,
                &context.target.hdr_view,
                4.0,
            );

            resources.insert_id(context.view.camera.cast(), ssr);
        }

        resources.insert(pipeline);

        resources.register_id::<MaterialRenderFunction>();

        resources.scope(
            |resources: &mut Resources, functions: &mut IdMap<MaterialRenderFunction>| {
                for function in functions.values() {
                    function.prepare(context, world, resources);
                }
            },
        );
    }

    fn render(
        &self,
        context: &ViewPhaseContext,
        encoder: &mut CommandEncoder,
        world: &World,
        resources: &Resources,
    ) {
        let mut draws = Vec::new();

        for function in resources.values_id::<MaterialRenderFunction>() {
            function.render(context, &mut draws, world, resources);
        }

        draws.retain(|draw| {
            if let Some(ref aabb) = draw.aabb {
                context.view.intersects(aabb, draw.transform)
            } else {
                true
            }
        });

        draws.sort_by(|a, b| {
            let a = a.distance(context.view.frustum());
            let b = b.distance(context.view.frustum());

            a.partial_cmp(&b).unwrap_or(Ordering::Equal)
        });

        let mut depth_prepass = context.target.begin_depth_prepass(encoder);

        for draw in draws.iter() {
            if draw.ssr {
                continue;
            }

            draw.prepass_draw(&mut depth_prepass);
        }

        drop(depth_prepass);

        if let Some(first_ssr) = draws.iter().rposition(|draw| draw.ssr) {
            let mut render_pass = context.target.begin_hdr_opaque_resolve_pass(encoder);
            for draw in draws[0..first_ssr].iter().filter(|draw| !draw.ssr) {
                draw.draw(&mut render_pass);
            }

            drop(render_pass);

            let pipeline = resources.get::<MipChainPipeline>().unwrap();
            let ssr = resources.get_id::<Ssr>(context.view.camera).unwrap();
            ssr.mip_chain.downsample(pipeline, encoder);

            let mut render_pass = context.target.begin_hdr_resolve_pass(encoder);

            let draws = draws.iter().enumerate().filter_map(|(i, draw)| {
                if i >= first_ssr || draw.ssr {
                    Some(draw)
                } else {
                    None
                }
            });

            for draw in draws {
                draw.draw(&mut render_pass);
            }
        } else {
            let mut render_pass = context.target.begin_hdr_resolve_pass(encoder);

            for draw in draws.iter() {
                draw.draw(&mut render_pass);
            }
        }
    }
}
