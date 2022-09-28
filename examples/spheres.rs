mod util;

use lumi::{prelude::*, renderer::RenderTarget};
use winit::event::Event;

fn main() {
    let mut world = World::new();

    let node = world.add(MeshNode::new(
        PbrMaterial {
            ..Default::default()
        },
        shape::cube(1.0, 1.0, 1.0),
        Mat4::IDENTITY,
    ));
    world.add_light(DirectionalLight {
        direction: Vec3::new(-1.0, -1.0, -1.0),
        intensity: 2.0,
        ..Default::default()
    });
    world.add_camera(Camera::default().with_position(Vec3::new(0.0, 0.0, 5.0)));

    util::framework(move |event, renderer, surface, size| match event {
        Event::RedrawRequested(_) => {
            let node = world.node_mut::<MeshNode<PbrMaterial>>(node);
            node.transform *= Mat4::from_rotation_y(0.01);

            let target = surface.get_current_texture().unwrap();
            let view = target.texture.create_view(&Default::default());
            let render_target = RenderTarget {
                view: &view,
                width: size.width,
                height: size.height,
            };
            renderer.render(&world, &render_target);
            target.present();
        }
        _ => (),
    });
}
