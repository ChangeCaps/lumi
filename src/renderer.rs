use std::collections::HashMap;

use wgpu::{CommandEncoder, PipelineLayout, RenderPipeline, TextureView};

use crate::{
    binding::BindingsLayout,
    bloom::Bloom,
    camera::{Camera, CameraInfo, CameraTarget},
    frame_buffer::FrameBuffer,
    id::{CameraId, NodeId},
    light::LightBindings,
    renderable::RenderContext,
    resources::{Resource, Resources},
    shader::{Shader, ShaderProcessor},
    tone_mapping::ToneMapping,
    world::World,
    SharedDevice, SharedQueue,
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
            bloom_threshold: 1.5,
            bloom_knee: 0.5,
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
        device: &SharedDevice,
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

    fn prepare(&mut self, device: &SharedDevice, camera: &Camera, target: &RenderTarget<'_>) {
        let width = camera.target.get_width(target.width);
        let height = camera.target.get_height(target.height);
        self.frame_buffer.resize(device, width, height);
        self.bloom.resize(device, width, height);
    }
}

pub struct Renderer {
    pub device: SharedDevice,
    pub queue: SharedQueue,
    pub settings: RenderSettings,
    resources: Resources,
    prepared_cameras: HashMap<CameraId, PreparedCamera>,
    tone_mapping: ToneMapping,
}

impl Renderer {
    pub fn new(device: SharedDevice, queue: SharedQueue) -> Self {
        let mut shader_processor = ShaderProcessor::default();
        let tone_mapping = ToneMapping::new(&device, &mut shader_processor);

        let mut resources = Resources::new();
        resources.insert(shader_processor);

        Self {
            device,
            queue,
            settings: RenderSettings::default(),
            resources,
            prepared_cameras: HashMap::new(),
            tone_mapping,
        }
    }

    pub const fn context(&self) -> RenderContext<'_> {
        RenderContext {
            device: &self.device,
            queue: &self.queue,
        }
    }

    pub fn register(&mut self, world: &World) {
        let context = RenderContext {
            device: &self.device,
            queue: &self.queue,
        };

        for dynamic_renderable in world.node_storage().dynamic_renderables() {
            dynamic_renderable.register(&context, &mut self.resources);
        }
    }

    pub fn prepare_view(&mut self, world: &World, target: &RenderTarget, camera_id: CameraId) {
        let context = RenderContext {
            device: &self.device,
            queue: &self.queue,
        };

        let prepared_camera = self.prepared_cameras.get_mut(&camera_id).unwrap();
        let camera = world.camera(camera_id);

        self.resources.insert(camera.info(target));

        for (node_id, renderable) in world.iter_renderables() {
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

    pub fn prepare_cameras(&mut self, world: &World, target: &RenderTarget<'_>) {
        for (camera_id, camera) in world.iter_cameras() {
            if let Some(prepared_camera) = self.prepared_cameras.get_mut(&camera_id) {
                prepared_camera.prepare(&self.device, camera, target);
            } else {
                let shader_processor = self.resources.get_mut_or_default::<ShaderProcessor>();
                let camera = PreparedCamera::new(&self.device, shader_processor, camera, target);

                self.prepared_cameras.insert(camera_id, camera);
            }
        }
    }

    pub fn prepare(&mut self, world: &World, target: &RenderTarget<'_>) {
        self.register(world);
        self.prepare_lights(world);
        self.prepare_cameras(world, target);

        for (camera_id, _camera) in world.iter_cameras() {
            self.prepare_view(world, target, camera_id);
        }
    }

    pub fn render_view(
        &mut self,
        world: &World,
        encoder: &mut CommandEncoder,
        target: &RenderTarget,
        camera_id: CameraId,
    ) {
        let context = RenderContext {
            device: &self.device,
            queue: &self.queue,
        };

        let prepared_camera = self.prepared_cameras.get_mut(&camera_id).unwrap();
        let camera = world.camera(camera_id);
        let mut render_pass = prepared_camera.frame_buffer.begin_hdr_render_pass(encoder);

        self.resources.insert(camera.info(target));

        for (node_id, renderable) in world.iter_renderables() {
            let state = prepared_camera.renderable_state.get(&node_id).unwrap();

            unsafe {
                renderable.render(&context, &mut render_pass, &self.resources, state.as_ref())
            };
        }

        drop(render_pass);

        self.resources.remove::<CameraInfo>();

        if self.settings.bloom_enabled {
            prepared_camera.bloom.render(
                &self.device,
                &self.queue,
                encoder,
                &prepared_camera.frame_buffer.hdr_view,
                self.settings.bloom_threshold,
                self.settings.bloom_knee,
                self.settings.bloom_scale,
            );
        }

        self.tone_mapping.run(
            &self.device,
            &self.queue,
            encoder,
            &prepared_camera.frame_buffer.hdr_view,
            target.view,
        );
    }

    pub fn render(&mut self, world: &World, target: &RenderTarget<'_>) {
        self.prepare(world, target);

        let mut encoder = self.device.create_command_encoder(&Default::default());

        let mut sorted_cameras = world.iter_cameras().collect::<Vec<_>>();
        sorted_cameras.sort_unstable_by_key(|(_, camera)| camera.priority);

        for (camera_id, camera) in sorted_cameras.into_iter().rev() {
            match camera.target {
                CameraTarget::Main => {
                    self.render_view(world, &mut encoder, target, camera_id);
                }
                CameraTarget::Texture(ref texture_view) => {
                    let target = RenderTarget {
                        view: texture_view.view(),
                        width: texture_view.size().width,
                        height: texture_view.size().height,
                    };

                    self.render_view(world, &mut encoder, &target, camera_id);
                }
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
    }
}
