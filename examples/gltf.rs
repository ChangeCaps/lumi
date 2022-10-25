mod util;

use lumi::prelude::*;
use util::{App, CameraController, FpsCounter};
use winit::event::Event;

struct GltfViewer {
    camera: CameraId,
    camera_controller: CameraController,
    model: NodeId,
    fps_counter: FpsCounter,
    material: StandardMaterial,
}

impl App for GltfViewer {
    fn init(world: &mut World, _renderer: &mut Renderer) -> Self {
        let model = MeshNode::open_gltf("examples/assets/suzanne.glb").unwrap();
        let model = world.add(model);

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
            model,
            fps_counter: FpsCounter::new(),
            material: Default::default(),
        }
    }

    fn event(&mut self, _world: &mut World, event: &Event<()>) {
        self.camera_controller.event(event);
    }

    fn render(&mut self, world: &mut World, _renderer: &mut Renderer, ctx: &egui::Context) {
        let camera = world.camera_mut(self.camera);
        camera.view = self.camera_controller.view();

        egui::Window::new("Material").show(ctx, |ui| {
            egui::Grid::new("grid").show(ui, |ui| {
                ui.label("Thickness");
                ui.add(egui::Slider::new(&mut self.material.thickness, 0.0..=1.0));
                ui.end_row();

                ui.add(egui::Checkbox::new(
                    &mut self.material.subsurface,
                    "Subsurface",
                ));
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

        for primitive in world.node_mut::<MeshNode>(self.model).primitives.iter_mut() {
            primitive.material = self.material.clone();
        }

        egui::Window::new("Information").show(ctx, |ui| {
            self.fps_counter.update();
            ui.label(format!("FPS: {:.2}", self.fps_counter.get_fps()));
        });
    }
}

fn main() {
    util::framework::<GltfViewer>();
}
