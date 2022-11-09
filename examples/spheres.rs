mod util;

use std::time::Instant;

use lumi::prelude::*;
use util::{App, CameraController};
use winit::event::Event;

struct Spheres {
    camera: Entity,
    camera_controller: CameraController,
    material: StandardMaterial,
    start: Instant,
    last_frame: Instant,
    frame_times: Vec<f32>,
}

impl App for Spheres {
    fn init(world: &mut World, _renderer: &mut Renderer) -> Self {
        let mesh = shape::uv_sphere(1.0, 32);

        world.insert_resource(Environment::open("env.hdr").unwrap());

        for x in -20..=20 {
            for z in -20..=20 {
                let transform = GlobalTransform::from_xyz(x as f32 * 3.0, 0.0, z as f32 * 3.0);
                world
                    .spawn()
                    .insert(transform)
                    .insert(mesh.clone())
                    .insert(StandardMaterial::default());
            }
        }

        world.spawn().insert(DirectionalLight {
            color: Vec3::new(1.0, 1.0, 1.0),
            direction: Vec3::new(1.0, -1.0, 1.0),
            illuminance: 75_000.0,
            ..Default::default()
        });

        let camera = world
            .spawn()
            .insert(Camera {
                aperture: 16.0,
                shutter_speed: 1.0 / 125.0,
                ..Default::default()
            })
            .insert(GlobalTransform::from_xyz(0.0, 0.0, 10.0))
            .entity();

        let material = StandardMaterial {
            base_color_texture: Some(Image::open_srgb("examples/assets/texture.png").unwrap()),
            ..Default::default()
        };

        Self {
            camera,
            camera_controller: CameraController::default(),
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
        let mut camera = world.entity_mut(self.camera);
        let mut camera = camera.get_mut::<GlobalTransform>().unwrap();
        camera.rotation_scale = self.camera_controller.rotation();
        camera.translation = self.camera_controller.translation();

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
        let mut query = world.query::<(&mut GlobalTransform, &mut StandardMaterial)>();

        for (mut transform, mut material) in query.iter_mut(world) {
            material.set(self.material.clone());

            let translation = transform.translation;
            let x = (translation.x + t).sin() * 0.5 + 0.5;
            let z = (translation.z * 2.0 + t).sin() * 0.5 + 0.5;
            let h = x + z;
            transform.translation = Vec3::new(translation.x, h, translation.z);
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
