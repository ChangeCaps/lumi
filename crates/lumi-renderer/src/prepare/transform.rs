use lumi_bind::Bind;
use lumi_core::{BufferInitDescriptor, BufferUsages, SharedBuffer, SharedDevice};
use lumi_util::{bytemuck, math::Mat4};

use shiv::{
    query::{Changed, Query, Without},
    system::{Commands, Res},
    world::{Component, Entity},
};
use shiv_transform::GlobalTransform;

use crate::{Extract, RenderDevice, RenderQueue};

#[derive(Component, Debug, Bind)]
pub struct PreparedTransform {
    #[uniform(name = "transform")]
    pub transform_buffer: SharedBuffer,
    pub transform: Mat4,
}

pub fn extract_transform_system(
    mut commands: Commands,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    transform_query: Extract<Query<(Entity, &GlobalTransform), Changed<GlobalTransform>>>,
    no_transform_query: Extract<Query<Entity, Without<GlobalTransform>>>,
    mut prepared_query: Query<&mut PreparedTransform>,
) {
    for (entity, transform) in transform_query.iter() {
        let matrix = transform.compute_matrix();

        if let Some(mut prepared) = prepared_query.get_mut(entity) {
            if prepared.transform != matrix {
                queue.write_buffer(&prepared.transform_buffer, 0, bytemuck::bytes_of(&matrix));

                prepared.transform = matrix;
            }
        } else {
            let buffer = device.create_shared_buffer_init(&BufferInitDescriptor {
                label: Some("Lumi Transform Buffer"),
                contents: bytemuck::bytes_of(&matrix),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            });

            let prepared = PreparedTransform {
                transform_buffer: buffer,
                transform: matrix,
            };

            commands.entity(entity).insert(prepared);
        }
    }

    for entity in no_transform_query.iter() {
        if let Some(mut prepared) = prepared_query.get_mut(entity) {
            if prepared.transform != Mat4::IDENTITY {
                queue.write_buffer(
                    &prepared.transform_buffer,
                    0,
                    bytemuck::bytes_of(&Mat4::IDENTITY),
                );

                prepared.transform = Mat4::IDENTITY;
            }
        } else {
            let buffer = device.create_shared_buffer_init(&BufferInitDescriptor {
                label: Some("Lumi Transform Buffer"),
                contents: bytemuck::bytes_of(&Mat4::IDENTITY),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            });

            let prepared = PreparedTransform {
                transform_buffer: buffer,
                transform: Mat4::IDENTITY,
            };

            commands.entity(entity).insert(prepared);
        }
    }
}
