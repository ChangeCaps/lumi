mod util;

use lumi::prelude::*;
use winit::event::Event;

fn main() {
    let mut world = World::new();

    let image = Image::load_from_file("examples/assets/texture.png").unwrap();
    let normal = NormalMap::load_from_file("examples/assets/normal.png").unwrap();

    let cube = world.add_node(Node::new(
        PbrMaterial {
            base_color_texture: Some(image),
            normal_map: Some(normal),
            ..Default::default()
        },
        shape::cube(1.0, 1.0, 1.0),
    ));
    world.add_light(DirectionalLight {
        direction: Vec3::new(-1.0, -1.0, -1.0),
        ..Default::default()
    });
    world.add_light(PointLight {
        position: Vec3::new(-1.5, 1.5, 1.5),
        color: Vec3::new(1.0, 0.0, 0.0),
        intensity: 4.0,
        ..Default::default()
    });
    world.add_camera(Camera::default().with_position(Vec3::new(0.0, 0.0, 5.0)));

    util::framework(move |event, renderer, surface| match event {
        Event::RedrawRequested(_) => {
            *world.transform_mut(cube) *= Mat4::from_rotation_y(0.01);

            let target = surface.get_current_texture().unwrap();
            renderer.render(&world, &target.texture);
            target.present();
        }
        _ => (),
    });
}
