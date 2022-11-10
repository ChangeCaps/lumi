#![deny(unsafe_op_in_unsafe_fn)]

mod bloom;
mod camera;
mod draw;
mod environment;
mod extract;
mod frame_buffer;
mod integrated_brdf;
mod light;
mod mip_chain;
mod plugin;
mod prepare;
mod resource;
mod screen_space;
mod sky;
mod tone_mapping;

pub use bloom::*;
pub use camera::*;
pub use draw::*;
pub use environment::*;
pub use extract::*;
pub use frame_buffer::*;
pub use integrated_brdf::*;
pub use light::*;
pub use mip_chain::*;
pub use plugin::*;
pub use prepare::*;
pub use resource::*;
pub use screen_space::*;
pub use sky::*;
pub use tone_mapping::*;

pub use shiv::{
    change_detection::Mut,
    query::{Query, QueryState, With, Without},
    world::{Entity, EntityMut, EntityRef, World},
};
pub use shiv_transform::*;

use lumi_core::{CommandEncoder, Device, Queue, RenderTarget, TextureView};
use lumi_util::HashMap;

use shiv::schedule::Schedule;

pub type RenderDevice = OwnedPtr<Device>;
pub type RenderQueue = OwnedPtr<Queue>;

#[derive(Clone, Debug)]
pub struct View {
    pub camera: Entity,
    pub frame_buffer: FrameBuffer,
    pub target: OwnedPtr<TextureView>,
}

pub struct Renderer {
    pub world: World,
    /// This schedule is run during [`Renderer::extract`] and should update [`Renderer::world`]
    /// to prepare it for rendering.
    ///
    /// This has access to [`RenderDevice`] and [`RenderQueue`] as well as [`Extract`].
    pub extract: Schedule,
    /// This schedule is run once for each enabled camera.
    ///
    /// This has access to [`RenderDevice`], [`RenderQueue`], [`CommandEncoder`] and [`View`].
    /// Where [`View`] contains information about the current camera and [`FrameBuffer`].
    pub view: Schedule,
    frame_buffers: HashMap<Entity, FrameBuffer>,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            extract: Schedule::default(),
            view: Schedule::default(),
            frame_buffers: HashMap::default(),
        }
    }

    pub fn add_plugin(&mut self, plugin: impl RendererPlugin) -> &mut Self {
        plugin.build(self);

        self
    }

    pub fn extract(&mut self, device: &Device, queue: &Queue, world: &mut World) {
        guard!(device);
        guard!(queue);
        guard_mut!(world);

        let encoder = device.create_command_encoder(&Default::default());

        self.world.insert_resource(encoder);
        self.world.insert_resource::<OwnedPtr<Device>>(device);
        self.world.insert_resource::<OwnedPtr<Queue>>(queue);
        self.world.insert_resource::<MainWorld>(world);

        self.extract.run_once(&mut self.world);

        self.world.remove_resource::<OwnedPtr<Device>>();
        self.world.remove_resource::<OwnedPtr<Queue>>();
        self.world.remove_resource::<MainWorld>();
    }

    fn prepare_frame_buffers(&mut self, device: &Device, target: &RenderTarget<'_>) {
        let query = self.world.query::<(Entity, &Camera)>();

        self.frame_buffers
            .retain(|&entity, _| query.contains(&self.world, entity));

        for (entity, camera) in query.iter(&self.world) {
            let width = camera.target.get_width(&target);
            let height = camera.target.get_height(&target);
            let sample_count = camera.sample_count();

            let frame_buffer = self
                .frame_buffers
                .entry(entity)
                .or_insert_with(|| FrameBuffer::new(device, width, height, sample_count));

            frame_buffer.resize(device, width, height, sample_count);
        }
    }

    pub fn render(&mut self, device: &Device, queue: &Queue, camera: Entity, target: RenderTarget) {
        let camera_entity = camera;

        self.prepare_frame_buffers(device, &target);

        guard!(device);
        guard!(queue);

        self.world.insert_resource::<OwnedPtr<Device>>(device);
        self.world.insert_resource::<OwnedPtr<Queue>>(queue);

        let camera = self.world.entity(camera_entity);
        let camera = camera.get::<Camera>().expect("camera not found");

        let target = camera.target.get_view(&target);
        guard!(target);

        let frame_buffer = self.frame_buffers[&camera_entity].clone();

        let view = View {
            camera: camera_entity,
            frame_buffer,
            target,
        };

        self.world.insert_resource(view);
        self.view.run_once(&mut self.world);
        self.world.remove_resource::<View>();

        self.world.remove_resource::<OwnedPtr<Device>>();
        let queue = self
            .world
            .remove_resource::<OwnedPtr<Queue>>()
            .expect("Queue removed by render system");

        let encoder = self
            .world
            .remove_resource::<CommandEncoder>()
            .expect("CommandEncoder removed by render system");

        queue.submit(std::iter::once(encoder.finish()));
    }
}
