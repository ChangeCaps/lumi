mod util;

use std::path::PathBuf;

use egui_file::FileDialog;
use lumi::prelude::*;
use util::{App, CameraController, FpsCounter};
use winit::event::Event;

struct GltfViewer {
    camera: Entity,
    camera_controller: CameraController,
    model: Entity,
    fps_counter: FpsCounter,
    material: StandardMaterial,
    file: PathBuf,
    file_dialog: Option<FileDialog>,
}

impl App for GltfViewer {
    fn init(world: &mut World, _renderer: &mut Renderer) -> Self {
        let model = Primitives::open_gltf("examples/assets/suzanne.glb").unwrap();

        world.insert_resource(Environment::open("env.hdr").unwrap());

        let model = world.spawn().insert(model).entity();

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

        Self {
            camera,
            camera_controller: CameraController::default(),
            model,
            fps_counter: FpsCounter::new(),
            material: Default::default(),
            file: PathBuf::from("examples/assets/suzanne.glb"),
            file_dialog: None,
        }
    }

    fn event(&mut self, _world: &mut World, event: &Event<()>) {
        self.camera_controller.event(event);
    }

    fn render(&mut self, world: &mut World, _renderer: &mut Renderer, ctx: &egui::Context) {
        let mut camera = world.entity_mut(self.camera);
        let mut camera = camera.get_mut::<GlobalTransform>().unwrap();
        camera.translation = self.camera_controller.translation();
        camera.rotation_scale = self.camera_controller.rotation();

        util::material_window(ctx, &mut self.material);

        egui::Window::new("File").show(ctx, |ui| {
            if ui.button("Open").clicked() {
                let mut dialog = FileDialog::open_file(Some(self.file.clone()));
                dialog.open();
                self.file_dialog = Some(dialog);
            }

            if let Some(dialog) = &mut self.file_dialog {
                if dialog.show(ctx).selected() {
                    if let Some(path) = dialog.path() {
                        self.file = path.clone();

                        if let Ok(model) = Primitives::open_gltf(path) {
                            let mut entity = world.entity_mut(self.model);
                            entity.insert(model);
                        } else {
                            println!("Failed to open gltf file");
                        }
                    }
                }
            }
        });

        let mut model = world.entity_mut(self.model);
        for _primitive in model.get_mut::<Primitives>().unwrap().iter_mut() {
            //primitive.material = self.material.clone();
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
