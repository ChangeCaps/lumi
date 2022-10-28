mod camera;
mod frame_buffer;
mod integrated_brdf;
mod mip_chain;
mod phase;
mod post_process;
mod prepare;
mod render_plugin;
mod sky;
mod tone_mapping;
mod view_phase;
mod world;

pub use camera::*;
pub use frame_buffer::*;
pub use integrated_brdf::*;
pub use mip_chain::*;
pub use phase::*;
pub use post_process::*;
pub use prepare::*;
pub use render_plugin::*;
pub use sky::*;
pub use tone_mapping::*;
pub use view_phase::*;
pub use world::*;

use std::{any::TypeId, sync::Arc};

use hyena::TaskPool;
use lumi_bounds::{BoundingShape, CameraFrustum, Frustum};
use lumi_core::{CommandEncoder, Device, Queue, RenderTarget, Resources, SharedBuffer};
use lumi_id::IdMap;
use lumi_shader::{FileShaderIo, ShaderProcessor};
use lumi_util::{math::Mat4, HashSet};
use lumi_world::{CameraId, CameraTarget, RawCamera, World, WorldChange, WorldChanges};

#[derive(Clone, Copy, Debug)]
pub struct RenderSettings {
    pub clear_color: [f32; 4],
    pub sample_count: u32,
    pub render_sky: bool,
    pub bloom_enabled: bool,
    pub bloom_threshold: f32,
    pub bloom_knee: f32,
    pub bloom_scale: f32,
    pub fxaa_enabled: bool,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            clear_color: [0.0, 0.0, 0.0, 1.0],
            sample_count: 4,
            render_sky: false,
            bloom_enabled: true,
            bloom_threshold: 3.5,
            bloom_knee: 1.0,
            bloom_scale: 1.0,
            fxaa_enabled: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct View {
    pub camera: CameraId,
    pub frustum: CameraFrustum,
    pub raw_camera: RawCamera,
    pub camera_buffer: SharedBuffer,
}

impl View {
    #[inline]
    pub fn intersects<T: BoundingShape>(&self, shape: &T, transform: Mat4) -> bool {
        self.frustum.intersects_shape(shape, transform)
    }

    pub fn frustum(&self) -> &Frustum {
        &self.frustum.frustum
    }
}

#[derive(Default)]
pub struct RendererBuilder {
    task_pool: Option<TaskPool>,
    phases: RenderPhases,
    view_phases: RenderViewPhases,
    plugins: Vec<Box<dyn RenderPlugin>>,
}

impl RendererBuilder {
    pub fn new() -> Self {
        let mut this = Self::default();
        this.add_plugin(CorePlugin);
        this
    }

    pub fn add_plugin<T: RenderPlugin + 'static>(&mut self, plugin: T) -> &mut Self {
        plugin.build(self);
        self.plugins.push(Box::new(plugin));
        self
    }

    pub fn add_phase<T: RenderPhase>(
        &mut self,
        label: impl Into<PhaseLabel>,
        phase: T,
    ) -> &mut Self {
        self.phases.push(label, phase);
        self
    }

    #[track_caller]
    pub fn add_phase_before<T: RenderPhase>(
        &mut self,
        before: impl Into<PhaseLabel>,
        label: impl Into<PhaseLabel>,
        phase: T,
    ) -> &mut Self {
        self.phases.insert_before(before, label, phase);
        self
    }

    #[track_caller]
    pub fn add_phase_after<T: RenderPhase>(
        &mut self,
        after: impl Into<PhaseLabel>,
        label: impl Into<PhaseLabel>,
        phase: T,
    ) -> &mut Self {
        self.phases.insert_after(after, label, phase);
        self
    }

    pub fn add_view_phase<T: RenderViewPhase>(
        &mut self,
        label: impl Into<PhaseLabel>,
        phase: T,
    ) -> &mut Self {
        self.view_phases.push(label, phase);
        self
    }

    #[track_caller]
    pub fn add_view_phase_before<T: RenderViewPhase>(
        &mut self,
        before: impl Into<PhaseLabel>,
        label: impl Into<PhaseLabel>,
        phase: T,
    ) -> &mut Self {
        self.view_phases.insert_before(before, label, phase);
        self
    }

    #[track_caller]
    pub fn add_view_phase_after<T: RenderViewPhase>(
        &mut self,
        after: impl Into<PhaseLabel>,
        label: impl Into<PhaseLabel>,
        phase: T,
    ) -> &mut Self {
        self.view_phases.insert_after(after, label, phase);
        self
    }

    pub fn set_task_pool(&mut self, task_pool: TaskPool) -> &mut Self {
        self.task_pool = Some(task_pool);
        self
    }

    pub fn build(&mut self, device: &Device) -> Renderer {
        let builder = std::mem::take(self);

        Renderer::new_internal(builder, device)
    }
}

pub struct Renderer {
    pub resources: Resources,
    registered_renderables: HashSet<TypeId>,
    phases: RenderPhases,
    view_phases: RenderViewPhases,
    prepared_cameras: IdMap<PreparedCamera>,
    prepared_worlds: PreparedWorlds,
    world_changes: WorldChanges,
    tone_mapping: ToneMapping,
}

impl Renderer {
    pub fn builder() -> RendererBuilder {
        RendererBuilder::new()
    }

    pub fn new(device: &Device) -> Self {
        RendererBuilder::new().build(device)
    }

    fn new_internal(builder: RendererBuilder, device: &Device) -> Self {
        let task_pool = builder
            .task_pool
            .unwrap_or_else(|| TaskPool::new().expect("Failed to create task pool"));

        let settings = RenderSettings::default();

        let shader_io = FileShaderIo::new(".");
        let mut shader_processor = ShaderProcessor::new(Arc::new(shader_io));

        let tone_mapping = ToneMapping::new(device, &mut shader_processor);

        let mut resources = Resources::new();
        resources.insert(task_pool);
        resources.insert(settings);
        resources.insert(shader_processor);

        let mut renderer = Self {
            resources,
            registered_renderables: HashSet::default(),
            phases: builder.phases,
            view_phases: builder.view_phases,
            prepared_cameras: IdMap::default(),
            prepared_worlds: PreparedWorlds::default(),
            world_changes: WorldChanges::default(),
            tone_mapping,
        };

        for plugin in builder.plugins {
            plugin.init(&mut renderer, device);
        }

        renderer
    }

    pub fn phases(&self) -> &RenderPhases {
        &self.phases
    }

    pub fn phases_mut(&mut self) -> &mut RenderPhases {
        &mut self.phases
    }

    #[track_caller]
    pub fn phase<T: RenderPhase>(&self, label: impl Into<PhaseLabel>) -> &T {
        self.phases.get::<T>(label).expect("RenderPhase not found")
    }

    #[track_caller]
    pub fn phase_mut<T: RenderPhase>(&mut self, label: impl Into<PhaseLabel>) -> &mut T {
        self.phases
            .get_mut::<T>(label)
            .expect("RenderPhase not found")
    }

    pub fn view_phases(&self) -> &RenderViewPhases {
        &self.view_phases
    }

    pub fn view_phases_mut(&mut self) -> &mut RenderViewPhases {
        &mut self.view_phases
    }

    #[track_caller]
    pub fn view_phase<T: RenderViewPhase>(&self, label: impl Into<PhaseLabel>) -> &T {
        self.view_phases
            .get::<T>(label)
            .expect("RenderViewPhase not found")
    }

    #[track_caller]
    pub fn view_phase_mut<T: RenderViewPhase>(&mut self, label: impl Into<PhaseLabel>) -> &mut T {
        self.view_phases
            .get_mut::<T>(label)
            .expect("RenderViewPhase not found")
    }

    #[track_caller]
    pub fn settings(&self) -> &RenderSettings {
        self.resources.get().unwrap()
    }

    #[track_caller]
    pub fn settings_mut(&mut self) -> &mut RenderSettings {
        self.resources.get_mut().unwrap()
    }

    #[inline]
    #[track_caller]
    pub fn register(&mut self, device: &Device, queue: &Queue, world: &World) {
        for (type_id, register_fn) in world.register_fns() {
            if self.registered_renderables.contains(type_id) {
                continue;
            }

            register_fn(device, queue, &mut self.resources);

            self.registered_renderables.insert(*type_id);
        }
    }

    pub fn prepare_changes(&mut self, world: &World) {
        let prepared_world = self.prepared_worlds.subscribe(world);

        self.world_changes.clear();

        for node_id in world.node_ids() {
            if prepared_world.objects.insert(node_id.cast()) {
                self.world_changes.added.insert(node_id.cast());
                self.world_changes.changed.insert(node_id.cast());
            }
        }

        for camera_id in world.camera_ids() {
            if prepared_world.objects.insert(camera_id.cast()) {
                self.world_changes.added.insert(camera_id.cast());
                self.world_changes.changed.insert(camera_id.cast());
            }
        }

        for light_id in world.light_ids() {
            if prepared_world.objects.insert(light_id.cast()) {
                self.world_changes.added.insert(light_id.cast());
                self.world_changes.changed.insert(light_id.cast());
            }
        }

        for change in prepared_world.changes.try_iter() {
            match change {
                WorldChange::Changed(node_id) => {
                    self.world_changes.changed.insert(node_id);
                }
                WorldChange::Removed(node_id) => {
                    prepared_world.objects.remove(&node_id);
                    self.world_changes.removed.insert(node_id);
                }
            }

            self.world_changes.push(change);
        }
    }

    pub fn prepare_view(&mut self, device: &Device, queue: &Queue, world: &World, view: &View) {
        let prepared_camera = self.prepared_cameras.get(&view.camera).unwrap();
        self.view_phases.prepare(
            device,
            queue,
            &prepared_camera.frame_buffer,
            view,
            world,
            &self.world_changes,
            &mut self.resources,
        );
    }

    #[track_caller]
    pub fn prepare_cameras(&mut self, device: &Device, world: &World, target: &RenderTarget<'_>) {
        let sample_count = self.settings().sample_count;

        for (camera_id, camera) in world.iter_cameras() {
            if let Some(prepared_camera) = self.prepared_cameras.get_mut(&camera_id) {
                prepared_camera.prepare(device, camera, target, sample_count);
            } else {
                let camera = PreparedCamera::new(device, camera, target, sample_count);

                self.prepared_cameras.insert(camera_id.cast(), camera);
            }
        }
    }

    pub fn prepare(
        &mut self,
        device: &Device,
        queue: &Queue,
        world: &World,
        target: &RenderTarget<'_>,
    ) {
        self.prepare_changes(world);
        self.register(device, queue, world);
        self.prepare_cameras(device, world, target);

        self.phases.prepare(
            device,
            queue,
            world,
            &self.world_changes,
            &mut self.resources,
        );

        for (camera_id, camera) in world.iter_cameras() {
            let aspect = camera.target.get_aspect(target);

            let prepared_camera = self.prepared_cameras.get_mut(&camera_id).unwrap();
            let camera_buffer = prepared_camera.camera.buffer(device, queue);

            let view = View {
                camera: camera_id,
                frustum: camera.camera_frustum(aspect),
                raw_camera: camera.raw_with_aspect(aspect),
                camera_buffer,
            };

            self.prepare_view(device, queue, world, &view);
        }
    }

    pub fn render_view(
        &mut self,
        device: &Device,
        queue: &Queue,
        world: &World,
        encoder: &mut CommandEncoder,
        target: &RenderTarget,
        camera_id: CameraId,
    ) {
        let prepared_camera = self.prepared_cameras.get_mut(&camera_id).unwrap();
        let camera = world.camera(camera_id);
        let camera_buffer = prepared_camera.camera.buffer(device, queue);

        let aspect = camera.target.get_aspect(target);
        let view = View {
            camera: camera_id,
            frustum: camera.camera_frustum(aspect),
            raw_camera: camera.raw_with_aspect(aspect),
            camera_buffer,
        };

        self.view_phases.render(
            device,
            queue,
            encoder,
            &prepared_camera.frame_buffer,
            &view,
            world,
            &self.world_changes,
            &self.resources,
        );

        self.tone_mapping.run(
            device,
            queue,
            encoder,
            &prepared_camera.frame_buffer.hdr_view,
            target.view,
        );
    }

    pub fn render(
        &mut self,
        device: &Device,
        queue: &Queue,
        world: &World,
        target: &RenderTarget<'_>,
    ) {
        self.prepare(device, queue, world, target);

        let mut encoder = device.create_command_encoder(&Default::default());

        self.phases.render(
            device,
            queue,
            &mut encoder,
            world,
            &self.world_changes,
            &self.resources,
        );

        let mut sorted_cameras = world.iter_cameras().collect::<Vec<_>>();
        sorted_cameras.sort_unstable_by_key(|(_, camera)| camera.priority);

        for (camera_id, camera) in sorted_cameras.into_iter().rev() {
            match camera.target {
                CameraTarget::Main => {
                    self.render_view(device, queue, world, &mut encoder, target, camera_id);
                }
                CameraTarget::Texture(ref texture_view) => {
                    let target = RenderTarget {
                        view: texture_view.view(),
                        width: texture_view.size().width,
                        height: texture_view.size().height,
                    };

                    self.render_view(device, queue, world, &mut encoder, &target, camera_id);
                }
            }
        }

        queue.submit(std::iter::once(encoder.finish()));
    }
}
