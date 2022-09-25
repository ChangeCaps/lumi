mod util;

use lumi::prelude::*;
use winit::event::Event;

#[derive(Bind)]
struct CustomMaterial;

impl Material for CustomMaterial {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::new("examples/assets/custom.wgsl")
    }
}

fn main() {
    let mut world = World::new();

    world.add_node(Node::new(CustomMaterial, shape::uv_sphere(1.0, 32)));
    world.add_light(DirectionalLight {
        direction: Vec3::new(-1.0, -1.0, 1.0),
        ..Default::default()
    });
    world.add_camera(Camera::default().with_position(Vec3::new(0.0, 0.0, 5.0)));

    util::framework(move |event, renderer, surface, size| match event {
        Event::RedrawRequested(_) => {
            let target = surface.get_current_texture().unwrap();
            let view = target.texture.create_view(&Default::default());
            renderer.render(&world, &view, size.width, size.height);
            target.present();
        }
        _ => (),
    });
}
