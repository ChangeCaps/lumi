mod util;

use lumi::bind;
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

    world.add(MeshNode::new(
        CustomMaterial,
        shape::uv_sphere(1.0, 32),
        Mat4::IDENTITY,
    ));
    world.add_light(DirectionalLight {
        direction: Vec3::new(-1.0, -1.0, 1.0),
        ..Default::default()
    });
    world.add_camera(Camera::default().with_position(Vec3::new(0.0, 0.0, 5.0)));

    util::framework(world, move |event, _renderer, _world, _| match event {
        Event::RedrawRequested(_) => {}
        _ => (),
    });
}
