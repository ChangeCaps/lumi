mod util;

use lumi::prelude::*;
use util::App;
use winit::event::Event;

struct Unlit;

impl App for Unlit {
    fn init(world: &mut World, renderer: &mut Renderer) -> Self {
        renderer.settings_mut().sample_count = 4;

        world.add(MeshNode::new(
            UnlitMaterial::new(Vec3::new(1.0, 0.0, 0.0)),
            shape::arrow_gizmo(Vec3::X),
            Mat4::IDENTITY,
        ));
        world.add(MeshNode::new(
            UnlitMaterial::new(Vec3::new(0.0, 1.0, 0.0)),
            shape::arrow_gizmo(Vec3::Y),
            Mat4::IDENTITY,
        ));
        world.add(MeshNode::new(
            UnlitMaterial::new(Vec3::new(0.0, 0.0, 1.0)),
            shape::arrow_gizmo(Vec3::Z),
            Mat4::IDENTITY,
        ));

        world.add_camera(Camera {
            view: Mat4::from_translation(Vec3::new(0.0, 0.0, 4.0)),
            ..Default::default()
        });

        Self
    }

    fn event(&mut self, _world: &mut World, _event: &Event<()>) {}

    fn render(&mut self, _world: &mut World, _renderer: &mut Renderer, _ctx: &egui::Context) {}
}

fn main() {
    util::framework::<Unlit>();
}
