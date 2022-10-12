mod phase;
mod view_phase;

use std::any::TypeId;

use glam::Mat4;
use wgpu::{CommandEncoder, Queue, TextureView};

use crate::{
    aabb::{Aabb, Frustum},
    bloom::{Bloom, BloomPipeline},
    camera::{Camera, CameraTarget, RawCamera},
    environment::{PreparedEnvironment, Sky},
    frame_buffer::FrameBuffer,
    id::CameraId,
    light::PrepareLightsPhase,
    material::MaterialPhase,
    mesh::PrepareMeshPhase,
    resources::Resources,
    shader::ShaderProcessor,
    shadow::ShadowPhase,
    tone_mapping::ToneMapping,
    util::{HashMap, HashSet},
    world::World,
    Device,
};

pub use phase::*;
pub use view_phase::*;

pub struct RenderSettings {
    pub clear_color: [f32; 4],
    pub aspect_ratio: Option<f32>,
    pub sample_count: u32,
    pub render_sky: bool,
    pub bloom_enabled: bool,
    pub bloom_threshold: f32,
    pub bloom_knee: f32,
    pub bloom_scale: f32,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            clear_color: [0.0, 0.0, 0.0, 1.0],
            aspect_ratio: None,
            sample_count: 1,
            render_sky: false,
            bloom_enabled: true,
            bloom_threshold: 3.5,
            bloom_knee: 1.0,
            bloom_scale: 1.0,
        }
    }
}

pub struct RenderTarget<'a> {
    pub view: &'a TextureView,
    pub width: u32,
    pub height: u32,
}

pub struct PreparedCamera {
    frame_buffer: FrameBuffer,
    bloom: Bloom,
}

impl PreparedCamera {
    fn new(
        device: &Device,
        bloom_pipeline: &BloomPipeline,
        camera: &Camera,
        sample_count: u32,
        target: &RenderTarget<'_>,
    ) -> Self {
        let width = camera.target.get_width(target.width);
        let height = camera.target.get_height(target.height);
        let frame_buffer = FrameBuffer::new(device, width, height, sample_count);
        Self {
            frame_buffer,
            bloom: Bloom::new(device, bloom_pipeline, width, height),
        }
    }

    fn prepare(
        &mut self,
        device: &Device,
        bloom_pipeline: &BloomPipeline,
        camera: &Camera,
        sample_count: u32,
        target: &RenderTarget<'_>,
    ) {
        let width = camera.target.get_width(target.width);
        let height = camera.target.get_height(target.height);

        self.frame_buffer.resize(device, width, height);
        self.bloom.resize(device, bloom_pipeline, width, height);

        self.frame_buffer.set_sample_count(device, sample_count);
    }
}

#[derive(Clone, Copy, Debug)]
pub struct View {
    pub camera: CameraId,
    pub raw_camera: RawCamera,
    pub frustum: Frustum,
    pub has_far_plane: bool,
}

impl View {
    #[inline]
    pub fn intersects(&self, aabb: &Aabb, transform: Mat4) -> bool {
        self.frustum
            .intersects_obb(&aabb, transform, self.has_far_plane)
    }
}

pub struct Renderer {
    pub resources: Resources,
    registered_renderables: HashSet<TypeId>,
    phases: RenderPhases,
    view_phases: RenderViewPhases,
    prepared_cameras: HashMap<CameraId, PreparedCamera>,
    sky: Sky,
    tone_mapping: ToneMapping,
}

impl Renderer {
    pub fn new(device: &Device, queue: &Queue) -> Self {
        let mut shader_processor = ShaderProcessor::default();
        let sky = Sky::new(device, queue, &mut shader_processor, 1);
        let tone_mapping = ToneMapping::new(&device, &mut shader_processor);
        let bloom_pipeline = BloomPipeline::new(device, &mut shader_processor);

        let mut resources = Resources::new();
        resources.insert(RenderSettings::default());
        resources.insert(shader_processor);
        resources.insert(bloom_pipeline);

        let mut this = Self {
            resources,
            registered_renderables: HashSet::default(),
            phases: RenderPhases::new(),
            view_phases: RenderViewPhases::new(),
            prepared_cameras: HashMap::default(),
            sky,
            tone_mapping,
        };

        this.insert_default_phases();

        this
    }

    pub fn insert_default_phases(&mut self) {
        self.phases
            .push(DefaultPhases::PrepareMeshes, PrepareMeshPhase);
        self.phases
            .push(DefaultPhases::PrepareLights, PrepareLightsPhase);
        self.phases.push(DefaultPhases::Shadow, ShadowPhase);

        self.view_phases
            .push(DefaultPhases::Material, MaterialPhase::default());
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

    pub fn settings(&self) -> &RenderSettings {
        self.resources.get().unwrap()
    }

    pub fn settings_mut(&mut self) -> &mut RenderSettings {
        self.resources.get_mut().unwrap()
    }

    pub fn register(&mut self, device: &Device, queue: &Queue, world: &World) {
        for (type_id, register_fn) in world.register_fns() {
            if self.registered_renderables.contains(type_id) {
                continue;
            }

            register_fn(device, queue, &mut self.resources);

            self.registered_renderables.insert(*type_id);
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
            &mut self.resources,
        );
    }

    pub fn prepare_cameras(&mut self, device: &Device, world: &World, target: &RenderTarget<'_>) {
        let sample_count = self.settings().sample_count;
        let bloom_pipeline = self.resources.get::<BloomPipeline>().unwrap();

        for (camera_id, camera) in world.iter_cameras() {
            if let Some(prepared_camera) = self.prepared_cameras.get_mut(&camera_id) {
                prepared_camera.prepare(device, bloom_pipeline, camera, sample_count, target);
            } else {
                let camera =
                    PreparedCamera::new(device, bloom_pipeline, camera, sample_count, target);

                self.prepared_cameras.insert(camera_id, camera);
            }
        }
    }

    pub fn prepare_environment(&mut self, device: &Device, queue: &Queue, world: &World) {
        if !self.resources.contains::<PreparedEnvironment>() {
            let environment = world.environment();
            let prepared_environment = PreparedEnvironment::new(
                device,
                queue,
                environment,
                self.sky.integrated_brdf.clone(),
            );
            self.resources.insert(prepared_environment);
        } else {
            let prepared_environment = self.resources.get_mut::<PreparedEnvironment>().unwrap();
            prepared_environment.prepare(device, queue, world.environment());
        }
    }

    pub fn prepare_sky(
        &mut self,
        device: &Device,
        queue: &Queue,
        world: &World,
        target: &RenderTarget<'_>,
    ) {
        let sample_count = self.settings().sample_count;
        let prepared_environment = self.resources.get_mut::<PreparedEnvironment>().unwrap();
        self.sky.prepare(
            device,
            queue,
            world,
            sample_count,
            target,
            prepared_environment,
        );
    }

    pub fn prepare(
        &mut self,
        device: &Device,
        queue: &Queue,
        world: &World,
        target: &RenderTarget<'_>,
    ) {
        self.register(device, queue, world);
        self.prepare_cameras(device, world, target);
        self.prepare_environment(device, queue, world);

        if self.settings().render_sky {
            self.prepare_sky(device, queue, world, target);
        }

        self.phases
            .prepare(device, queue, world, &mut self.resources);

        for (camera_id, camera) in world.iter_cameras() {
            let aspect = camera.target.get_aspect(target);

            let view = View {
                camera: camera_id,
                raw_camera: camera.raw_with_aspect(aspect),
                frustum: camera.frustum_with_aspect(aspect),
                has_far_plane: camera.has_far_plane(),
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
        let render_sky = self.settings().render_sky;
        let prepared_camera = self.prepared_cameras.get_mut(&camera_id).unwrap();
        let camera = world.camera(camera_id);

        let mut sky_pass = prepared_camera
            .frame_buffer
            .begin_hdr_render_pass(encoder, false);
        if render_sky {
            self.sky.render(&mut sky_pass, camera_id);
        }

        drop(sky_pass);

        let aspect = camera.target.get_aspect(target);
        let view = View {
            camera: camera_id,
            raw_camera: camera.raw_with_aspect(aspect),
            frustum: camera.frustum_with_aspect(aspect),
            has_far_plane: camera.has_far_plane(),
        };

        self.view_phases.render(
            device,
            queue,
            encoder,
            &prepared_camera.frame_buffer,
            &view,
            world,
            &self.resources,
        );

        let settings = self.resources.get::<RenderSettings>().unwrap();

        if settings.bloom_enabled {
            let bloom_pipeline = self.resources.get::<BloomPipeline>().unwrap();
            prepared_camera.bloom.render(
                device,
                queue,
                bloom_pipeline,
                encoder,
                &prepared_camera.frame_buffer.hdr_view,
                settings.bloom_threshold,
                settings.bloom_knee,
                settings.bloom_scale,
            );
        }

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

        self.phases
            .render(device, queue, &mut encoder, world, &self.resources);

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
