mod util;

use lumi::prelude::*;
use util::App;
use winit::event::{DeviceEvent, ElementState, Event, MouseButton, MouseScrollDelta, WindowEvent};

struct Scene {
    camera: CameraId,
    rotate: bool,
    position: Vec3,
    rotation: Vec2,
}

impl Scene {
    pub fn rotation(&self) -> Mat4 {
        Mat4::from_rotation_y(self.rotation.x) * Mat4::from_rotation_x(self.rotation.y)
    }
}

impl App for Scene {
    fn init(world: &mut World, renderer: &mut Renderer) -> Self {
        renderer.settings_mut().render_sky = true;
        renderer.settings_mut().sample_count = 4;

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
            rotate: false,
            position: Vec3::ZERO,
            rotation: Vec2::ZERO,
        }
    }

    fn event(&mut self, _world: &mut World, event: &Event<()>) {
        match event {
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::MouseMotion { delta } => {
                    if self.rotate {
                        self.rotation -= Vec2::new(delta.0 as f32, delta.1 as f32) * 0.001;
                    }
                }
                DeviceEvent::MouseWheel { delta } => {
                    let delta = match delta {
                        MouseScrollDelta::LineDelta(_, y) => *y,
                        MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
                    };

                    let direction = self.rotation().transform_vector3(Vec3::new(0.0, 0.0, -1.0));

                    self.position -= direction * delta * 0.001;
                }
                _ => {}
            },
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::MouseInput { state, button, .. } => {
                    if *button == MouseButton::Right {
                        self.rotate = *state == ElementState::Pressed;
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn render(&mut self, world: &mut World, _renderer: &mut Renderer, _ctx: &egui::Context) {
        let mut camera = world.camera_mut(self.camera);
        camera.view = Mat4::from_translation(self.position)
            * Mat4::from_rotation_y(self.rotation.x)
            * Mat4::from_rotation_x(self.rotation.y);
    }
}

fn main() {
    util::framework::<Scene>();
}
