mod util;

use lumi::prelude::*;
use winit::event::Event;

fn main() {
    let mut world = World::new();

    let mesh = MeshNode::open_gltf("examples/assets/susanne.glb").unwrap();

    let node = world.add(mesh);

    world.ambient_mut().intensity = 35_000.0;

    world.add_light(DirectionalLight {
        color: Vec3::new(1.0, 1.0, 1.0),
        direction: Vec3::new(1.0, -1.0, 1.0),
        ..Default::default()
    });

    world.add_camera(Camera {
        view: Mat4::from_translation(Vec3::new(0.0, 0.0, 4.0)),
        aperture: 16.0,
        shutter_speed: 1.0 / 125.0,
        ..Default::default()
    });

    util::framework(world, move |event, renderer, world, ctx| match event {
        Event::RedrawRequested(_) => {
            renderer.settings_mut().render_sky = false;
            renderer.settings_mut().sample_count = 4;

            //let point_light = world.light_mut::<PointLight>(point_light);

            //point_light.position =
            //    Mat4::from_rotation_y(0.01).transform_point3(point_light.position);

            egui::Window::new("World").show(ctx, |ui| {
                egui::Grid::new("world").show(ui, |ui| {
                    ui.label("Ambient");
                    ui.add(egui::DragValue::new(&mut world.ambient_mut().intensity));
                    ui.end_row();
                });
            });

            let node = world.node_mut::<MeshNode>(node);

            egui::Window::new("Material").show(ctx, |ui| {
                egui::Grid::new("grid").show(ui, |ui| {
                    let material = &mut node.primitives[0].material;

                    ui.label("Thickness");
                    ui.add(egui::Slider::new(&mut material.thickness, 0.0..=1.0));
                    ui.end_row();

                    ui.label("Subsurface Power");
                    ui.add(egui::Slider::new(
                        &mut material.subsurface_power,
                        0.0..=10.0,
                    ));
                    ui.end_row();

                    ui.label("Subsurface Color");
                    egui::color_picker::color_edit_button_rgb(
                        ui,
                        bytemuck::cast_mut(&mut material.subsurface_color),
                    );
                    ui.end_row();

                    ui.label("Base Color");
                    egui::color_picker::color_edit_button_rgba(
                        ui,
                        bytemuck::cast_mut(&mut material.base_color),
                        egui::color_picker::Alpha::Opaque,
                    );
                    ui.end_row();

                    ui.label("Metallic");
                    ui.add(egui::Slider::new(&mut material.metallic, 0.0..=1.0));
                    ui.end_row();

                    ui.label("Roughness");
                    ui.add(egui::Slider::new(&mut material.roughness, 0.0..=1.0));
                    ui.end_row();

                    ui.label("Clearcoat");
                    ui.add(egui::Slider::new(&mut material.clearcoat, 0.0..=1.0));
                    ui.end_row();

                    ui.label("Clearcoat Roughness");
                    ui.add(egui::Slider::new(
                        &mut material.clearcoat_roughness,
                        0.0..=1.0,
                    ));
                    ui.end_row();

                    ui.label("Emissive");
                    egui::color_picker::color_edit_button_rgb(
                        ui,
                        bytemuck::cast_mut(&mut material.emissive),
                    );
                    ui.end_row();

                    ui.label("Transmission");
                    ui.add(egui::Slider::new(&mut material.transmission, 0.0..=1.0));
                    ui.end_row();

                    ui.label("Ior");
                    ui.add(egui::Slider::new(&mut material.ior, 0.0..=10.0));
                    ui.end_row();

                    ui.label("Absorption");
                    egui::color_picker::color_edit_button_rgb(
                        ui,
                        bytemuck::cast_mut(&mut material.absorption),
                    );
                    ui.end_row();
                });
            });
        }
        _ => {}
    });
}
