mod util;

use std::time::Instant;

use lumi::prelude::*;
use util::{App, CameraController};
use winit::event::Event;

struct Spheres {
    camera: CameraId,
    camera_controller: CameraController,
    material: StandardMaterial,
    spheres: Vec<NodeId>,
    start: Instant,
    last_frame: Instant,
    frame_times: Vec<f32>,
}

impl App for Spheres {
    fn init(world: &mut World, renderer: &mut Renderer) -> Self {
        renderer.settings_mut().render_sky = true;

        *world.environment_mut() = Environment::open("env.hdr").unwrap();

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

        let material = StandardMaterial {
            base_color_texture: Some(Image::open_srgb("examples/assets/texture.png").unwrap()),
            ..Default::default()
        };

        Self {
            camera,
            camera_controller: CameraController::default(),
            spheres: suzannes,
            material,
            start: Instant::now(),
            last_frame: Instant::now(),
            frame_times: Vec::new(),
        }
    }

    fn event(&mut self, _world: &mut World, event: &Event<()>) {
        self.camera_controller.event(event);
    }

    fn render(&mut self, world: &mut World, _renderer: &mut Renderer, ctx: &egui::Context) {
        let camera = world.camera_mut(self.camera);
        camera.view = self.camera_controller.view();

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
                    lumi_util::bytemuck::cast_mut(&mut self.material.subsurface_color),
                );
                ui.end_row();

                ui.label("Base Color");
                egui::color_picker::color_edit_button_rgba(
                    ui,
                    lumi_util::bytemuck::cast_mut(&mut self.material.base_color),
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
                    lumi_util::bytemuck::cast_mut(&mut self.material.emissive),
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
                    lumi_util::bytemuck::cast_mut(&mut self.material.absorption),
                );
                ui.end_row();
            });
        });

        let t = self.start.elapsed().as_secs_f32();
        for &sphere in &self.spheres {
            let spheres = world.node_mut::<MeshNode>(sphere);
            spheres.primitives[0].material = self.material.clone();

            let (_, _, translation) = spheres.transform.to_scale_rotation_translation();
            let x = (translation.x + t).sin() * 0.5 + 0.5;
            let z = (translation.z * 2.0 + t).sin() * 0.5 + 0.5;
            let h = x + z;
            spheres.transform = Mat4::from_translation(Vec3::new(translation.x, h, translation.z));
        }

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
    util::framework::<Spheres>();
}
