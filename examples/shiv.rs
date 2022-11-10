use lumi::prelude::*;
use lumi::LumiPlugin;

use shiv::prelude::*;
use shiv_app::App;
use shiv_app::AppExit;
use shiv_transform::TransformPlugin;
use shiv_window::CloseRequested;

fn main() {
    App::new()
        .add_plugin(LumiPlugin)
        .add_plugin(TransformPlugin)
        .add_startup_system(setup)
        .add_system(exit_system)
        .run();
}

fn setup(mut commands: Commands) {
    commands
        .spawn()
        .insert(Camera::default())
        .insert(Transform::from_xyz(0.0, 0.0, 10.0))
        .insert(GlobalTransform::default());

    commands
        .spawn()
        .insert(shape::cube(1.0, 1.0, 1.0))
        .insert(StandardMaterial::default())
        .insert(Transform::from_xyz(0.0, 0.0, 0.0))
        .insert(GlobalTransform::default());

    commands.insert_resource(Environment::open("env.hdr").unwrap());
}

fn exit_system(close_requested: EventReader<CloseRequested>, mut exit: EventWriter<AppExit>) {
    if !close_requested.is_empty() {
        exit.send(AppExit);
    }
}
