use std::{collections::HashMap, time::Instant};

use wgpu::{CommandEncoder, PipelineLayout, Queue, RenderPipeline, TextureView};

use crate::{
    binding::BindingsLayout,
    bloom::Bloom,
    camera::{Camera, CameraInfo, CameraTarget},
    environment::{PreparedEnvironment, Sky},
    frame_buffer::FrameBuffer,
    id::{CameraId, NodeId},
    light::LightBindings,
    renderable::RenderContext,
    resources::{Resource, Resources},
    shader::{Shader, ShaderProcessor},
    tone_mapping::ToneMapping,
    world::World,
    Device,
};

#[allow(unused)]
struct CachedMaterial {
    vertex_shader: Shader,
    fragment_shader: Shader,
    bindings_layout: BindingsLayout,
    pipeline_layout: PipelineLayout,
    pipeline: RenderPipeline,
}

pub struct RenderSettings {
    pub clear_color: [f32; 4],
    pub aspect_ratio: Option<f32>,
    pub sample_count: u32,
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
    renderable_state: HashMap<NodeId, Box<dyn Resource>>,
}

impl PreparedCamera {
    fn new(
        device: &Device,
        shader_processor: &mut ShaderProcessor,
        camera: &Camera,
        target: &RenderTarget<'_>,
    ) -> Self {
        let width = camera.target.get_width(target.width);
        let height = camera.target.get_height(target.height);
        let frame_buffer = FrameBuffer::new(device, width, height, 4);
        Self {
            frame_buffer,
            bloom: Bloom::new(device, shader_processor, width, height),
            renderable_state: HashMap::new(),
        }
    }

    fn prepare(&mut self, device: &Device, camera: &Camera, target: &RenderTarget<'_>) {
        let width = camera.target.get_width(target.width);
        let height = camera.target.get_height(target.height);
        self.frame_buffer.resize(device, width, height);
        self.bloom.resize(device, width, height);
    }
}

pub struct Renderer {
    pub settings: RenderSettings,
    pub resources: Resources,
    prepared_cameras: HashMap<CameraId, PreparedCamera>,
    sky: Sky,
    tone_mapping: ToneMapping,
}

impl Renderer {
    pub fn new(device: &Device, queue: &Queue) -> Self {
        let mut shader_processor = ShaderProcessor::default();
        let sky = Sky::new(device, queue, &mut shader_processor);
        let tone_mapping = ToneMapping::new(&device, &mut shader_processor);

        let mut resources = Resources::new();
        resources.insert(shader_processor);

        Self {
            settings: RenderSettings::default(),
            resources,
            prepared_cameras: HashMap::new(),
            sky,
            tone_mapping,
        }
    }

    pub fn register(&mut self, device: &Device, queue: &Queue, world: &World) {
        for dynamic_renderable in world.node_storage().dynamic_renderables() {
            dynamic_renderable.register(device, queue, &mut self.resources);
        }
    }

    pub fn prepare_view(
        &mut self,
        device: &Device,
        queue: &Queue,
        world: &World,
        target: &RenderTarget,
        camera_id: CameraId,
    ) {
        let prepared_camera = self.prepared_cameras.get_mut(&camera_id).unwrap();
        let camera = world.camera(camera_id);

        self.resources.insert(camera.info(target));

        for (node_id, renderable) in world.iter_renderables() {
            let context = RenderContext {
                device,
                queue,
                view: camera_id,
                node: node_id,
            };

            let state = prepared_camera
                .renderable_state
                .entry(node_id)
                .or_insert_with(|| renderable.init(&context, &mut self.resources));

            unsafe { renderable.prepare(&context, &mut self.resources, state.as_mut()) };
        }

        self.resources.remove::<CameraInfo>();
    }

    pub fn prepare_lights(&mut self, world: &World) {
        let light_bindings = self.resources.get_mut_or_default::<LightBindings>();
        light_bindings.clear();

        for light in world.lights() {
            light_bindings.push(light.clone());
        }
    }

    pub fn prepare_cameras(&mut self, device: &Device, world: &World, target: &RenderTarget<'_>) {
        for (camera_id, camera) in world.iter_cameras() {
            if let Some(prepared_camera) = self.prepared_cameras.get_mut(&camera_id) {
                prepared_camera.prepare(device, camera, target);
            } else {
                let shader_processor = self.resources.get_mut_or_default::<ShaderProcessor>();
                let camera = PreparedCamera::new(device, shader_processor, camera, target);

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
        }
    }

    pub fn prepare_sky(
        &mut self,
        device: &Device,
        queue: &Queue,
        world: &World,
        target: &RenderTarget<'_>,
    ) {
        let prepared_environment = self.resources.get::<PreparedEnvironment>().unwrap();
        self.sky
            .prepare(device, queue, world, target, prepared_environment);
    }

    pub fn prepare(
        &mut self,
        device: &Device,
        queue: &Queue,
        world: &World,
        target: &RenderTarget<'_>,
    ) {
        self.register(device, queue, world);
        self.prepare_lights(world);
        self.prepare_cameras(device, world, target);
        self.prepare_environment(device, queue, world);
        self.prepare_sky(device, queue, world, target);

        for (camera_id, _camera) in world.iter_cameras() {
            self.prepare_view(device, queue, world, target, camera_id);
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
        log::trace!("Rendering view: {}", camera_id.uuid());
        let view_start = Instant::now();

        let prepared_camera = self.prepared_cameras.get_mut(&camera_id).unwrap();
        let camera = world.camera(camera_id);
        let mut render_pass = prepared_camera.frame_buffer.begin_hdr_render_pass(encoder);

        self.sky.render(&mut render_pass, camera_id);

        let t = Instant::now();
        self.resources.insert(camera.info(target));

        for (node_id, renderable) in world.iter_renderables() {
            let context = RenderContext {
                device,
                queue,
                view: camera_id,
                node: node_id,
            };

            let state = prepared_camera.renderable_state.get(&node_id).unwrap();

            unsafe {
                renderable.render(&context, &mut render_pass, &self.resources, state.as_ref())
            };
        }

        drop(render_pass);
        log::trace!("Main Pass took: {:?}", t.elapsed());

        self.resources.remove::<CameraInfo>();

        if self.settings.bloom_enabled {
            let t = Instant::now();
            prepared_camera.bloom.render(
                device,
                queue,
                encoder,
                &prepared_camera.frame_buffer.hdr_view,
                self.settings.bloom_threshold,
                self.settings.bloom_knee,
                self.settings.bloom_scale,
            );
            log::trace!("Bloom Pass took: {:?}", t.elapsed());
        }

        let t = Instant::now();
        self.tone_mapping.run(
            device,
            queue,
            encoder,
            &prepared_camera.frame_buffer.hdr_view,
            target.view,
        );
        log::trace!("Tone Mapping Pass took: {:?}", t.elapsed());

        log::trace!(
            "Finished view: {} took: {:?}",
            camera_id.uuid(),
            view_start.elapsed()
        );
    }

    pub fn render(
        &mut self,
        device: &Device,
        queue: &Queue,
        world: &World,
        target: &RenderTarget<'_>,
    ) {
        log::trace!("Starting Frame");
        let start_frame = Instant::now();

        let start_prepare = Instant::now();
        self.prepare(device, queue, world, target);
        log::trace!("Preparation took: {:?}", start_prepare.elapsed());

        let mut encoder = device.create_command_encoder(&Default::default());

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

        log::trace!("Finished Frame took: {:?}", start_frame.elapsed());
    }
}
