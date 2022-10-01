mod util;

use lumi::prelude::*;
use winit::event::Event;

fn main() {
    let mut world = World::new();

    let delorean = MeshNode::open_gltf("examples/assets/delorean.glb").unwrap();
    let node = world.add(delorean);
    world.add_camera(Camera::default().with_position(Vec3::new(0.0, 0.0, 5.0)));

    util::framework(world, move |event, renderer, world, _ctx| match event {
        Event::RedrawRequested(_) => {
            renderer.settings_mut().render_sky = false;
            renderer.settings_mut().sample_count = 4;

            world.node_mut::<MeshNode>(node).transform *= Mat4::from_rotation_y(0.01);
        }
        _ => {}
    });
}
