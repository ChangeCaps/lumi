mod util;

use lumi::prelude::*;
use winit::event::Event;

fn main() {
    let mut world = World::new();

    let node = world.add(MeshNode::new(
        PbrMaterial {
            ..Default::default()
        },
        shape::uv_sphere(0.5, 32),
        //shape::cube(1.0, 1.0, 1.0),
        Mat4::IDENTITY,
    ));
    world.add_light(DirectionalLight {
        direction: Vec3::new(-1.0, -1.0, -1.0),
        intensity: 2.0,
        ..Default::default()
    });
    world.add_camera(Camera::default().with_position(Vec3::new(0.0, 0.0, 5.0)));

    util::framework(world, move |event, _renderer, world, ctx| match event {
        Event::RedrawRequested(_) => {
            let node = world.node_mut::<MeshNode<PbrMaterial>>(node);
            node.transform *= Mat4::from_rotation_y(0.01);

            let material = &mut node.primitives[0].material;

            egui::Window::new("Material Properties").show(ctx, |ui| {
                egui::Grid::new("material_grid").show(ui, |ui| {
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
                    ui.label("Emissive");
                    ui.color_edit_button_rgb(bytemuck::cast_mut(&mut material.emissive));
                    ui.end_row();
                })
            });
        }
        _ => {}
    });
}
