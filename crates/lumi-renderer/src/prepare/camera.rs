use lumi_bind::Bind;
use lumi_core::UniformBuffer;
use shiv::{
    query::{Changed, Query},
    system::{Commands, Res},
    world::{Component, Entity},
};

use crate::{Camera, Extract, PreparedTransform, RawCamera, View};

#[derive(Component, Debug, Bind)]
pub struct PreparedCamera {
    #[uniform]
    pub camera: UniformBuffer<RawCamera>,
}

pub fn extract_camera_system(
    mut commands: Commands,
    extract_query: Extract<Query<(Entity, &Camera), Changed<Camera>>>,
) {
    for (entity, camera) in extract_query.iter() {
        commands.entity(entity).insert(camera.clone());
    }
}

pub fn prepare_camera_system(
    mut commands: Commands,
    view: Res<View>,
    mut query: Query<(
        Entity,
        &Camera,
        &PreparedTransform,
        Option<&mut PreparedCamera>,
    )>,
) {
    if let Some((entity, camera, transform, prepared)) = query.get_mut(view.camera) {
        let view_matrix = transform.transform;
        let raw_camera = camera.raw_with_aspect(view_matrix, view.frame_buffer.aspect_ratio());

        if let Some(mut prepared) = prepared {
            prepared.camera.set(raw_camera);
        } else {
            commands.entity(entity).insert(PreparedCamera {
                camera: UniformBuffer::new(raw_camera),
            });
        }
    }
}
