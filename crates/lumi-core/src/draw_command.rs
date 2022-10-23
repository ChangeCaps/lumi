use std::ops::Range;

use wgpu::RenderPass;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum DrawCommand {
    Indexed {
        indices: Range<u32>,
        base_vertex: i32,
    },
    Vertex {
        vertices: Range<u32>,
    },
}

impl DrawCommand {
    #[inline]
    pub fn draw(&self, render_pass: &mut RenderPass) {
        match self {
            DrawCommand::Indexed {
                indices,
                base_vertex,
            } => {
                render_pass.draw_indexed(indices.clone(), *base_vertex, 0..1);
            }
            DrawCommand::Vertex { vertices } => {
                render_pass.draw(vertices.clone(), 0..1);
            }
        }
    }

    #[inline]
    pub fn draw_instanced(&self, render_pass: &mut RenderPass, instances: Range<u32>) {
        match self {
            DrawCommand::Indexed {
                indices,
                base_vertex,
            } => {
                render_pass.draw_indexed(indices.clone(), *base_vertex, instances);
            }
            DrawCommand::Vertex { vertices } => {
                render_pass.draw(vertices.clone(), instances);
            }
        }
    }
}
