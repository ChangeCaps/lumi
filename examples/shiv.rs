use lumi::prelude::*;
use lumi::LumiPlugin;

use lumi_material::MaterialBundle;
use shiv::prelude::*;
use shiv_app::App;
use shiv_app::AppExit;
use shiv_transform::TransformPlugin;
use shiv_window::CloseRequested;

#[derive(Component, Clone, Copy, Debug, Default)]
struct Velocity {
    velocity: Vec3,
}

fn main() {
    App::new()
        .add_plugin(LumiPlugin)
        .add_plugin(TransformPlugin)
        .add_startup_system(setup)
        .add_system(exit_system)
        .add_system(gravity_system.label(ExampleSystem::Gravity))
        .add_system(
            velocity_system
                .label(ExampleSystem::Velocity)
                .after(ExampleSystem::Gravity),
        )
        .run();
}

fn setup(mut commands: Commands) {
    commands
        .spawn()
        .insert(Camera::default())
        .insert(Transform::from_xyz(0.0, 0.0, 20.0))
        .insert(GlobalTransform::default());

    for x in -5..=5 {
        for y in -5..=5 {
            for z in -5..=5 {
                commands
                    .spawn()
                    .insert(MaterialBundle {
                        mesh: shape::cube(0.5, 0.5, 0.5),
                        material: StandardMaterial::default(),
                        transform: Transform::from_xyz(x as f32, y as f32, z as f32),
                        ..Default::default()
                    })
                    .insert(Velocity::default());
            }
        }
    }
}

#[derive(SystemLabel)]
enum ExampleSystem {
    Gravity,
    Velocity,
}

fn gravity_system(mut query: Query<(&mut Velocity, &GlobalTransform)>) {
    for (mut velocity, transform) in query.iter_mut() {
        velocity.velocity -= transform.translation.normalize() * 0.01;
    }
}

fn velocity_system(mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation += velocity.velocity * 0.01;
    }
}

fn exit_system(close_requested: EventReader<CloseRequested>, mut exit: EventWriter<AppExit>) {
    if !close_requested.is_empty() {
        exit.send(AppExit);
    }
}
