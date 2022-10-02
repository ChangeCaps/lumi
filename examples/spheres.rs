mod util;

use lumi::prelude::*;
use winit::event::Event;

fn main() {
    let mut world = World::new();

    let image = Image::load_from_file("examples/assets/normal.png").unwrap();
    let mesh = shape::uv_sphere(0.5, 32);

    for i in 0..10 {
        let f = i as f32 / 9.0;

        world.add(MeshNode::new(
            StandardMaterial {
                normal_map: Some(NormalMap::new(image.clone())),
                base_color: Vec4::new(0.0, 0.0, 1.0, 1.0),
                metallic: 1.0,
                roughness: 0.8,
                clearcoat: f,
                ..Default::default()
            },
            mesh.clone(),
            Mat4::from_translation(Vec3::new(f * 12.0 - 6.0, 0.0, 0.0)),
        ));
    }

    world.add_camera(Camera::default().with_position(Vec3::new(0.0, 0.0, 12.0)));

    util::framework(world, move |event, renderer, _world, _ctx| match event {
        Event::RedrawRequested(_) => {
            renderer.settings_mut().render_sky = false;
            renderer.settings_mut().sample_count = 4;
        }
        _ => {}
    });
}
