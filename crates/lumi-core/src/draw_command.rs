use std::ops::Range;

use wgpu::RenderPass;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum DrawCommand {
    Indexed {
        indices: Range<u32>,
        base_vertex: i32,
        instances: Range<u32>,
    },
    Vertex {
        vertices: Range<u32>,
        instances: Range<u32>,
    },
}

impl DrawCommand {
    #[inline]
    pub fn draw(&self, render_pass: &mut RenderPass) {
        match self {
            DrawCommand::Indexed {
                indices,
                base_vertex,
                instances,
            } => {
                render_pass.draw_indexed(indices.clone(), *base_vertex, instances.clone());
            }
            DrawCommand::Vertex {
                vertices,
                instances,
            } => {
                render_pass.draw(vertices.clone(), instances.clone());
            }
        }
    }
}
