mod util;

use lumi_material::StandardMaterial;
use lumi_renderer::{Camera, DirectionalLight, GlobalTransform};
pub use util::*;

use lumi::prelude::*;

struct Sphere;

impl App for Sphere {
    fn init(world: &mut World, _renderer: &mut Renderer) -> Self {
        let mesh = shape::uv_sphere(1.0, 32);

        world
            .spawn()
            .insert(mesh)
            .insert(StandardMaterial::default());
        world.spawn().insert(DirectionalLight::default());
        world
            .spawn()
            .insert(Camera::default())
            .insert(GlobalTransform::from_xyz(0.0, 0.0, 5.0));

        Self
    }

    fn event(&mut self, _world: &mut World, _event: &winit::event::Event<()>) {}

    fn render(&mut self, _world: &mut World, _renderer: &mut Renderer, _ctx: &egui::Context) {}
}

fn main() {
    framework::<Sphere>();
}
