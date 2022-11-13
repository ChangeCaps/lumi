use lumi::prelude::*;
use lumi::LumiPlugin;

use shiv::prelude::*;
use shiv_app::App;
use shiv_transform::TransformPlugin;

fn main() {
    App::new()
        .add_plugin(LumiPlugin)
        .add_plugin(TransformPlugin)
        .add_startup_system(setup)
        .add_system(rotate_system)
        .run();
}

#[derive(Component)]
struct Rotate;

fn setup(mut commands: Commands) {
    commands.spawn().insert(PerspectiveCameraBundle {
        transform: Transform::from_xyz(0.0, 0.0, 10.0),
        ..Default::default()
    });

    commands.spawn().insert(DirectionalLightBundle {
        light: DirectionalLight {
            direction: Vec3::new(-1.0, -1.0, -1.0),
            ..Default::default()
        },
        ..Default::default()
    });

    commands.insert_resource(Environment::open("sky.hdr").unwrap());

    let mesh = shape::cube(1.0, 1.0, 1.0);

    commands
        .spawn()
        .insert(MaterialBundle {
            mesh: mesh.clone(),
            material: StandardMaterial::default(),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..Default::default()
        })
        .insert(Rotate)
        .with_children(|parent| {
            parent
                .spawn()
                .insert(MaterialBundle {
                    mesh: mesh.clone(),
                    material: StandardMaterial::default(),
                    transform: Transform::from_xyz(0.0, 2.0, 0.0),
                    ..Default::default()
                })
                .insert(Rotate);
        });
}

fn rotate_system(mut query: Query<&mut Transform, With<Rotate>>) {
    for mut transform in query.iter_mut() {
        transform.rotation *= Quat::from_rotation_z(0.1);
    }
}
