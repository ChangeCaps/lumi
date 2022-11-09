use std::{
    cmp::Ordering,
    ops::{Deref, DerefMut},
};

use deref_derive::{Deref, DerefMut};
use lumi_bounds::{Aabb, BoundingShape, Frustum};
use lumi_core::{
    CommandEncoder, DrawCommand, IndexFormat, RenderPass, SharedBindGroup, SharedBuffer,
    SharedRenderPipeline,
};
use lumi_util::{
    math::{Mat4, Vec3A, Vec4Swizzles},
    smallvec::SmallVec,
};

use shiv::{
    query::Query,
    system::{Res, ResMut},
};

use crate::{Camera, PreparedTransform, View};

#[derive(Clone, Debug)]
pub struct Draw {
    pub prepass_pipeline: SharedRenderPipeline,
    pub resolve_pipeline: SharedRenderPipeline,
    pub bind_groups: SmallVec<[SharedBindGroup; 4]>,
    pub vertex_buffers: SmallVec<[(u32, SharedBuffer); 8]>,
    pub index_buffer: Option<SharedBuffer>,
    pub draw_command: DrawCommand,
    pub aabb: Option<Aabb>,
    pub transform: Mat4,
}

impl Draw {
    #[inline]
    pub fn distance(&self, frustum: &Frustum) -> f32 {
        let far_plane = frustum.planes()[5];

        if let Some(aabb) = self.aabb {
            let center = aabb.center();
            let aabb_center_world = self.transform.transform_point3a(center.into());
            let axes = [
                Vec3A::from(self.transform.x_axis),
                Vec3A::from(self.transform.y_axis),
                Vec3A::from(self.transform.z_axis),
            ];

            let relative_radius = aabb.relative_radius(far_plane.normal(), &axes);

            far_plane.distance(aabb_center_world) - relative_radius
        } else {
            let position = self.transform.w_axis.xyz();
            let distance = far_plane.distance(position.into());

            distance
        }
    }

    #[inline]
    pub fn draw_prepass<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        render_pass.set_pipeline(&self.prepass_pipeline);

        for (index, bind_group) in self.bind_groups.iter().enumerate() {
            render_pass.set_bind_group(index as u32, bind_group, &[]);
        }

        for (index, buffer) in self.vertex_buffers.iter() {
            render_pass.set_vertex_buffer(*index, buffer.slice(..));
        }

        if let Some(buffer) = &self.index_buffer {
            render_pass.set_index_buffer(buffer.slice(..), IndexFormat::Uint32);
        }

        self.draw_command.draw(render_pass);
    }

    #[inline]
    pub fn draw_resolve<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        render_pass.set_pipeline(&self.resolve_pipeline);

        for (index, bind_group) in self.bind_groups.iter().enumerate() {
            render_pass.set_bind_group(index as u32, bind_group, &[]);
        }

        for (index, buffer) in self.vertex_buffers.iter() {
            render_pass.set_vertex_buffer(*index, buffer.slice(..));
        }

        if let Some(buffer) = &self.index_buffer {
            render_pass.set_index_buffer(buffer.slice(..), IndexFormat::Uint32);
        }

        self.draw_command.draw(render_pass);
    }
}

#[derive(Clone, Debug, Default, Deref, DerefMut)]
pub struct OpaqueDraws {
    pub draws: Vec<Draw>,
}

#[derive(Clone, Debug, Default, Deref, DerefMut)]
pub struct TransparentDraws {
    pub draws: Vec<Draw>,
}

pub fn clear_draws_system(
    mut opaque_draws: ResMut<OpaqueDraws>,
    mut transparent_draws: ResMut<TransparentDraws>,
) {
    opaque_draws.clear();
    transparent_draws.clear();
}

#[derive(Clone, Debug)]
pub struct DrawKey {
    pub distance: f32,
    pub is_transparent: bool,
    pub index: usize,
}

impl DrawKey {
    #[inline]
    pub fn get_draw<'a>(
        &self,
        opaque_draws: &'a OpaqueDraws,
        transparent_draws: &'a TransparentDraws,
    ) -> &'a Draw {
        if self.is_transparent {
            &transparent_draws[self.index]
        } else {
            &opaque_draws[self.index]
        }
    }

    #[inline]
    pub fn draw_resolve<'a>(
        &self,
        opaque_draws: &'a OpaqueDraws,
        transparent_draws: &'a TransparentDraws,
        render_pass: &mut RenderPass<'a>,
    ) {
        let draw = self.get_draw(opaque_draws, transparent_draws);
        draw.draw_resolve(render_pass);
    }
}

#[derive(Clone, Debug, Default)]
pub struct DrawKeys {
    pub keys: Vec<DrawKey>,
    pub first_transparent: Option<usize>,
}

impl DrawKeys {
    #[inline]
    pub fn prepare(&mut self) {
        self.keys.sort_by(|a, b| {
            a.distance
                .partial_cmp(&b.distance)
                .unwrap_or(Ordering::Equal)
        });

        self.first_transparent = self.keys.iter().rposition(|key| key.is_transparent);
    }
}

impl Deref for DrawKeys {
    type Target = Vec<DrawKey>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.keys
    }
}

impl DerefMut for DrawKeys {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.keys
    }
}

pub fn draw_system(
    view: Res<View>,
    mut opaque_draws: ResMut<OpaqueDraws>,
    mut transparent_draws: ResMut<TransparentDraws>,
    mut draw_keys: ResMut<DrawKeys>,
    camera_query: Query<(&Camera, &PreparedTransform)>,
) {
    let (camera, camera_transform) = camera_query.get(view.camera).unwrap();
    let frustum =
        camera.camera_frustum(camera_transform.transform, view.frame_buffer.aspect_ratio());

    opaque_draws.retain(|draw| {
        if let Some(aabb) = draw.aabb {
            frustum.intersects_shape(&aabb, draw.transform)
        } else {
            true
        }
    });

    transparent_draws.retain(|draw| {
        if let Some(aabb) = draw.aabb {
            frustum.intersects_shape(&aabb, draw.transform)
        } else {
            true
        }
    });

    draw_keys.clear();

    for (index, draw) in opaque_draws.iter().enumerate() {
        draw_keys.push(DrawKey {
            distance: draw.distance(&frustum.frustum),
            is_transparent: false,
            index,
        });
    }

    for (index, draw) in transparent_draws.iter().enumerate() {
        draw_keys.push(DrawKey {
            distance: draw.distance(&frustum.frustum),
            is_transparent: true,
            index,
        });
    }

    draw_keys.prepare();
}

pub fn render_opaque_system(
    mut encoder: ResMut<CommandEncoder>,
    view: Res<View>,
    opaque_draws: Res<OpaqueDraws>,
    draw_keys: Res<DrawKeys>,
) {
    let mut depth_prepass = view.frame_buffer.begin_depth_prepass(&mut encoder);

    for draw_key in draw_keys.iter() {
        if draw_key.is_transparent {
            continue;
        }

        let draw = &opaque_draws[draw_key.index];
        draw.draw_prepass(&mut depth_prepass);
    }

    drop(depth_prepass);

    let mut opaque_pass = (view.frame_buffer).begin_hdr_opaque_resolve_pass(&mut encoder);

    if let Some(first_transparent) = draw_keys.first_transparent {
        for draw_key in draw_keys
            .iter()
            .take(first_transparent)
            .filter(|key| !key.is_transparent)
        {
            let draw = &opaque_draws[draw_key.index];
            draw.draw_resolve(&mut opaque_pass);
        }
    } else {
        for draw_key in draw_keys.iter() {
            let draw = &opaque_draws[draw_key.index];
            draw.draw_resolve(&mut opaque_pass);
        }
    }
}

pub fn render_transparent_system(
    mut encoder: ResMut<CommandEncoder>,
    view: Res<View>,
    opaque_draws: Res<OpaqueDraws>,
    transparent_draws: Res<TransparentDraws>,
    draw_keys: Res<DrawKeys>,
) {
    if let Some(first_transparent) = draw_keys.first_transparent {
        let mut transparent_pass = view.frame_buffer.begin_hdr_resolve_pass(&mut encoder);

        for (i, draw_key) in draw_keys.iter().enumerate() {
            if i < first_transparent && !draw_key.is_transparent {
                continue;
            }

            draw_key.draw_resolve(&opaque_draws, &transparent_draws, &mut transparent_pass);
        }
    }
}
