mod util;

use lumi::prelude::*;
use util::App;
use winit::event::{DeviceEvent, ElementState, Event, MouseButton, MouseScrollDelta, WindowEvent};

struct Spheres {
    camera: CameraId,
    material: StandardMaterial,
    spheres: Vec<NodeId>,
    rotate: bool,
    rotation: Vec2,
    distance: f32,
}

impl App for Spheres {
    fn init(world: &mut World, renderer: &mut Renderer) -> Self {
        renderer.settings_mut().render_sky = true;
        renderer.settings_mut().sample_count = 4;

        let mesh = MeshNode::new(
            StandardMaterial::default(),
            shape::uv_sphere(1.0, 32),
            Mat4::IDENTITY,
        );

        let mut suzannes = Vec::new();
        for x in -20..=20 {
            for z in -20..=20 {
                let mut mesh = mesh.clone();
                mesh.transform =
                    Mat4::from_translation(Vec3::new(x as f32 * 3.0, 0.0, z as f32 * 3.0));
                let sphere = world.add(mesh);
                suzannes.push(sphere);
            }
        }

        world.add(MeshNode::new(
            StandardMaterial {
                roughness: 0.7,
                ..Default::default()
            },
            shape::cube(1000.0, 1.0, 1000.0),
            Mat4::from_translation(Vec3::new(0.0, -2.0, 0.0)),
        ));

        world.ambient_mut().intensity = 15_000.0;

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

        let material = StandardMaterial::default();

        Self {
            camera,
            spheres: suzannes,
            material,
            rotate: false,
            rotation: Vec2::ZERO,
            distance: 4.0,
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

                    self.distance += delta * 0.005;
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

    fn render(&mut self, world: &mut World, _renderer: &mut Renderer, ctx: &egui::Context) {
        let camera = world.camera_mut(self.camera);
        let mut rotation = Mat4::from_rotation_y(self.rotation.x);
        rotation *= Mat4::from_rotation_x(self.rotation.y);

        camera.view = rotation * Mat4::from_translation(Vec3::new(0.0, 0.0, self.distance));

        egui::Window::new("World").show(ctx, |ui| {
            egui::Grid::new("world").show(ui, |ui| {
                ui.label("Ambient");
                ui.add(egui::DragValue::new(&mut world.ambient_mut().intensity));
                ui.end_row();
            });
        });

        egui::Window::new("Material").show(ctx, |ui| {
            egui::Grid::new("grid").show(ui, |ui| {
                ui.label("Thickness");
                ui.add(egui::Slider::new(&mut self.material.thickness, 0.0..=1.0));
                ui.end_row();

                ui.label("Subsurface Power");
                ui.add(egui::Slider::new(
                    &mut self.material.subsurface_power,
                    0.0..=10.0,
                ));
                ui.end_row();

                ui.label("Subsurface Color");
                egui::color_picker::color_edit_button_rgb(
                    ui,
                    bytemuck::cast_mut(&mut self.material.subsurface_color),
                );
                ui.end_row();

                ui.label("Base Color");
                egui::color_picker::color_edit_button_rgba(
                    ui,
                    bytemuck::cast_mut(&mut self.material.base_color),
                    egui::color_picker::Alpha::OnlyBlend,
                );
                ui.end_row();

                ui.label("Metallic");
                ui.add(egui::Slider::new(&mut self.material.metallic, 0.0..=1.0));
                ui.end_row();

                ui.label("Roughness");
                ui.add(egui::Slider::new(&mut self.material.roughness, 0.0..=1.0));
                ui.end_row();

                ui.label("Clearcoat");
                ui.add(egui::Slider::new(&mut self.material.clearcoat, 0.0..=1.0));
                ui.end_row();

                ui.label("Clearcoat Roughness");
                ui.add(egui::Slider::new(
                    &mut self.material.clearcoat_roughness,
                    0.0..=1.0,
                ));
                ui.end_row();

                ui.label("Emissive");
                egui::color_picker::color_edit_button_rgb(
                    ui,
                    bytemuck::cast_mut(&mut self.material.emissive),
                );
                ui.end_row();

                ui.label("Transmission");
                ui.add(egui::Slider::new(
                    &mut self.material.transmission,
                    0.0..=1.0,
                ));
                ui.end_row();

                ui.label("Ior");
                ui.add(egui::Slider::new(&mut self.material.ior, 1.0..=4.0));
                ui.end_row();

                ui.label("Absorption");
                egui::color_picker::color_edit_button_rgb(
                    ui,
                    bytemuck::cast_mut(&mut self.material.absorption),
                );
                ui.end_row();
            });
        });

        for &sphere in &self.spheres {
            world.node_mut::<MeshNode>(sphere).primitives[0].material = self.material.clone();
        }
    }
}

fn main() {
    util::framework::<Spheres>();
}
