use lumi_bounds::{Aabb, BoundingShape, Frustum};
use lumi_core::{
    DrawCommand, IndexFormat, RenderPass, SharedBindGroup, SharedBuffer, SharedRenderPipeline,
};
use lumi_util::{
    math::{Mat4, Vec3A},
    smallvec::SmallVec,
};

#[derive(Clone, Debug)]
pub struct MaterialDraw {
    pub prepass_pipeline: SharedRenderPipeline,
    pub render_pipeline: SharedRenderPipeline,
    pub bind_groups: SmallVec<[SharedBindGroup; 4]>,
    pub vertex_buffers: SmallVec<[(u32, SharedBuffer); 4]>,
    pub index_buffer: Option<SharedBuffer>,
    pub draw_command: DrawCommand,
    pub ssr: bool,
    pub aabb: Option<Aabb>,
    pub transform: Mat4,
}

impl MaterialDraw {
    #[inline]
    pub fn distance(&self, frustum: &Frustum) -> f32 {
        let near = frustum.planes()[5];

        if let Some(aabb) = self.aabb {
            let center = aabb.center();
            let center_world = self.transform.transform_point3a(center.into()).extend(1.0);
            let axes = [
                Vec3A::from(self.transform.x_axis),
                Vec3A::from(self.transform.y_axis),
                Vec3A::from(self.transform.z_axis),
            ];

            let relative_radius = aabb.relative_radius(near.normal(), &axes);

            near.normal_d().dot(center_world) - relative_radius
        } else {
            let mut position = self.transform.w_axis;
            position.w = 1.0;

            near.normal_d().dot(position)
        }
    }

    #[inline]
    pub fn prepass_draw<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
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
    pub fn draw<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        render_pass.set_pipeline(&self.render_pipeline);

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
