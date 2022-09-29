mod util;

use lumi::prelude::*;
use winit::event::Event;

fn main() {
    let mut world = World::new();

    let mesh = shape::uv_sphere(0.5, 32);
    for x in -4..=4 {
        for y in -2..=2 {
            world.add(MeshNode::new(
                PbrMaterial {
                    metallic: y as f32 / 9.0 + 0.5,
                    roughness: x as f32 / 5.0 + 0.5,
                    ..Default::default()
                },
                mesh.clone(),
                Mat4::from_translation(Vec3::new(x as f32 * 1.5, y as f32 * 1.5, 0.0)),
            ));
        }
    }
    world.add_light(DirectionalLight {
        direction: Vec3::new(-1.0, -1.0, -1.0),
        intensity: 2.0,
        ..Default::default()
    });
    world.add_camera(Camera::default().with_position(Vec3::new(0.0, 0.0, 10.0)));

    util::framework(world, move |event, renderer, _world, _ctx| match event {
        Event::RedrawRequested(_) => {
            renderer.settings_mut().render_sky = false;
            renderer.settings_mut().sample_count = 4;
        }
        _ => {}
    });
}
