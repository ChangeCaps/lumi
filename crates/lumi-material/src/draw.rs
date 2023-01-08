use std::{iter, ops::Deref};

use deref_derive::{Deref, DerefMut};
use lumi_bind::Binding;
use lumi_id::Id;
use lumi_mesh::Mesh;
use lumi_renderer::{
    Draw, Entity, Extract, IntegratedBrdf, OpaqueDraws, PreparedCamera, PreparedEnvironment,
    PreparedLights, PreparedMeshes, PreparedShadows, PreparedTransform, Query, RenderDevice,
    RenderQueue, ScreenSpaceTarget, TransparentDraws, View, Without,
};
use lumi_shader::ShaderProcessor;
use lumi_util::{smallvec::SmallVec, HashMap};
use shiv::{
    query::{Changed, QueryItem, ReadOnlyWorldQuery},
    storage::SparseArray,
    system::{Commands, Res, ResMut, SystemParam},
    world::Component,
};

use crate::{
    Material, PreparedMaterialPipeline, PreparedMaterialPipelineKey, PreparedMaterialPipelines,
};

pub trait ExtractMaterials: Component {
    type Material: Material;
    type MeshQuery: ReadOnlyWorldQuery;
    type Iter<'w>: Iterator<Item = &'w Self::Material>;
    type MeshIter<'w>: Iterator<Item = (&'w Self::Material, &'w Mesh)>;

    fn extract(&self) -> Self;
    fn iter(&self) -> Self::Iter<'_>;
    fn mesh_iter<'w>(item: &'w QueryItem<Self::MeshQuery>) -> Self::MeshIter<'w>;
}

impl<T: Material> ExtractMaterials for T {
    type Material = T;
    type MeshQuery = (&'static T, &'static Mesh);
    type Iter<'w> = iter::Once<&'w Self::Material>;
    type MeshIter<'w> = iter::Once<(&'w Self::Material, &'w Mesh)>;

    #[inline]
    fn extract(&self) -> Self {
        self.clone()
    }

    #[inline]
    fn iter(&self) -> Self::Iter<'_> {
        iter::once(self)
    }

    #[inline]
    fn mesh_iter<'w>(&item: &'w QueryItem<Self::MeshQuery>) -> Self::MeshIter<'w> {
        iter::once(item)
    }
}

#[derive(Component, Default, Deref, DerefMut)]
pub struct MaterialRenderState {
    #[deref]
    pub bindings: HashMap<Entity, Binding>,
    pub pipeline: Id<PreparedMaterialPipeline>,
}

#[derive(Component, Default, Deref, DerefMut)]
pub struct MaterialRenderStates {
    pub states: SparseArray<MaterialRenderState>,
}

pub fn extract_material_system<T: ExtractMaterials>(
    mut commands: Commands,
    extract_query: Extract<Query<(Entity, &T), Changed<T>>>,
    mut material_query: Query<&mut T>,
    state_query: Query<(), Without<MaterialRenderStates>>,
) {
    for (entity, extract) in extract_query.iter() {
        let extracted = extract.extract();

        if let Some(mut material) = material_query.get_mut(entity) {
            *material = extracted;
        } else {
            commands.entity(entity).insert(extracted);
        }

        if state_query.contains(entity) {
            commands
                .entity(entity)
                .insert(MaterialRenderStates::default());
        }
    }
}

#[derive(SystemParam)]
pub struct PreparedParams<'w> {
    pub lights: Res<'w, PreparedLights>,
    pub shadows: Res<'w, PreparedShadows>,
    pub environment: Res<'w, PreparedEnvironment>,
    pub integrated_brdf: Res<'w, IntegratedBrdf>,
}

pub fn prepare_material_system<T: ExtractMaterials>(
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    view: Res<View>,
    prepared: PreparedParams,
    mut shader_processor: ResMut<ShaderProcessor>,
    mut pipelines: ResMut<PreparedMaterialPipelines>,
    camera_query: Query<(&PreparedCamera, &ScreenSpaceTarget)>,
    query: Query<(Entity, &T, &PreparedTransform), Changed<T>>,
    mut state_query: Query<&mut MaterialRenderStates>,
    changed_screen_space: Query<Entity, Changed<ScreenSpaceTarget>>,
) {
    let (prepared_camera, screen_space_target) = camera_query.get(view.camera).unwrap();
    let screen_space_bindings = screen_space_target.bindings();

    let screen_space_changed = changed_screen_space.contains(view.camera);

    for (entity, extract, transform) in query.iter() {
        for (i, material) in extract.iter().enumerate() {
            let key = PreparedMaterialPipelineKey::new(material, view.frame_buffer.sample_count());

            let pipeline =
                pipelines.get_or_create::<T::Material>(&device, &key, &mut shader_processor);

            let mut states = state_query.get_mut(entity).unwrap();
            let state = states.get_or_default(i);

            if !state.contains_key(&view.camera) || state.pipeline != key.id() {
                let mut bindings = pipeline.bindings_layout.create_bindings(&device);

                bindings.bind(&device, &queue, prepared_camera);
                bindings.bind(&device, &queue, prepared.integrated_brdf.deref());
                bindings.bind(&device, &queue, transform);

                bindings.bind(&device, &queue, prepared.lights.deref());
                bindings.bind(&device, &queue, prepared.environment.deref());
                bindings.bind(&device, &queue, prepared.shadows.deref());
                bindings.bind(&device, &queue, &screen_space_bindings);

                state.insert(view.camera, bindings);
                state.pipeline = key.id();
            }

            let bindings = state.get_mut(&view.camera).unwrap();

            bindings.bind(&device, &queue, material);
        }
    }

    let mut update_bindings = false;

    let lights_changed = prepared.lights.bindings_changed;
    let shadows_changed = prepared.shadows.bindings_changed;
    let environment_changed = prepared.environment.is_changed();

    update_bindings |= lights_changed;
    update_bindings |= shadows_changed;
    update_bindings |= environment_changed;
    update_bindings |= screen_space_changed;

    if update_bindings {
        for mut states in state_query.iter_mut() {
            for (_, state) in states.iter_mut() {
                let bindings = state.get_mut(&view.camera).unwrap();

                if lights_changed {
                    bindings.bind(&device, &queue, prepared.lights.deref());
                }

                if shadows_changed {
                    bindings.bind(&device, &queue, prepared.shadows.deref());
                }

                if environment_changed {
                    bindings.bind(&device, &queue, prepared.environment.deref());
                }

                if screen_space_changed {
                    bindings.bind(&device, &queue, &screen_space_bindings);
                }
            }
        }
    }
}

pub fn update_bindings_system(
    device: Res<RenderDevice>,
    view: Res<View>,
    mut query: Query<&mut MaterialRenderStates>,
) {
    for mut states in query.iter_mut() {
        for (_, state) in states.iter_mut() {
            if !state.contains_key(&view.camera) {
                continue;
            }

            let bindings = state.get_mut(&view.camera).unwrap();
            bindings.update_bind_groups(&device);
        }
    }
}

pub fn draw_material_system<T: ExtractMaterials>(
    view: Res<View>,
    prepared_meshes: Res<PreparedMeshes>,
    mut opaque_draws: ResMut<OpaqueDraws>,
    mut transparent_draws: ResMut<TransparentDraws>,
    pipelines: Res<PreparedMaterialPipelines>,
    query: Query<(T::MeshQuery, &PreparedTransform, &MaterialRenderStates)>,
) {
    for (extract, transform, states) in query.iter() {
        for (i, (material, mesh)) in T::mesh_iter(&extract).enumerate() {
            let state = states.get(i).unwrap();

            let key = PreparedMaterialPipelineKey::new(material, view.frame_buffer.sample_count());
            let pipeline = pipelines.get(key.id()).unwrap();

            let resolve_pipeline = if material.is_translucent() {
                pipeline.transparent_pipeline.clone()
            } else {
                pipeline.opaque_pipeline.clone()
            };

            let bindings = state.get(&view.camera).unwrap();
            let bind_groups: SmallVec<_> = bindings.bind_groups().cloned().collect();

            let prepared_mesh = prepared_meshes.get(mesh.id()).unwrap();

            let mut vertex_buffers = SmallVec::new();
            for vertex_layout in pipeline.material_pipeline.vertices.iter() {
                let buffer = prepared_mesh
                    .attributes
                    .get(vertex_layout.attribute.as_ref())
                    .expect("Missing vertex attribute");

                vertex_buffers.push((vertex_layout.location, buffer.clone()));
            }

            let draw = Draw {
                prepass_pipeline: pipeline.prepass_pipeline.clone(),
                resolve_pipeline: resolve_pipeline.clone(),
                bind_groups: bind_groups.clone(),
                vertex_buffers,
                index_buffer: prepared_mesh.indices.clone(),
                draw_command: prepared_mesh.draw.clone(),
                aabb: prepared_mesh.aabb,
                transform: transform.transform,
            };

            if material.is_translucent() {
                transparent_draws.push(draw);
            } else {
                opaque_draws.push(draw);
            }
        }
    }
}
