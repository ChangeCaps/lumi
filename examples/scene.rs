mod util;

use std::time::Instant;

use lumi::prelude::*;
use util::{App, CameraController};
use winit::event::Event;

struct Scene {
    camera: CameraId,
    camera_controller: CameraController,
    last_frame: Instant,
    frame_times: Vec<f32>,
}

impl App for Scene {
    fn init(world: &mut World, renderer: &mut Renderer) -> Self {
        renderer.settings_mut().render_sky = true;

        let scene = MeshNode::open_gltf("examples/assets/scene.glb").unwrap();
        world.add(scene);

        *world.environment_mut() = Environment::open("env.hdr").unwrap();

        world.add_light(DirectionalLight {
            color: Vec3::new(1.0, 1.0, 1.0),
            direction: Vec3::new(1.0, -1.0, 1.0),
            illuminance: 75_000.0,
            ..Default::default()
        });

        let camera = world.add_camera(Camera {
            view: Mat4::from_translation(Vec3::new(0.0, 0.0, 4.0)),
            aperture: 16.0,
            shutter_speed: 1.0 / 125.0,
            ..Default::default()
        });

        Self {
            camera,
            camera_controller: CameraController::default(),
            last_frame: Instant::now(),
            frame_times: Vec::new(),
        }
    }

    fn event(&mut self, _world: &mut World, event: &Event<()>) {
        self.camera_controller.event(event);
    }

    fn render(&mut self, world: &mut World, _renderer: &mut Renderer, ctx: &egui::Context) {
        let mut camera = world.camera_mut(self.camera);
        camera.view = self.camera_controller.view();

        egui::Window::new("Information").show(ctx, |ui| {
            let frame_time = self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;

            ui.label(format!("FPS: {:.2}", 1.0 / frame_time));
        });

        self.frame_times
            .push(self.last_frame.elapsed().as_secs_f32());

        if self.frame_times.len() > 60 {
            self.frame_times.remove(0);
        }

        self.last_frame = Instant::now();
    }
}

fn main() {
    util::framework::<Scene>();
}
