use lumi_bind::Bind;
use lumi_core::VertexFormat;
use lumi_mesh::Mesh;
use lumi_shader::ShaderRef;
use lumi_util::math::Vec3;
use shiv::world::Component;

use crate::{Material, MaterialPipeline, MeshVertexLayout};

#[derive(Component, Clone, Copy, Debug, PartialEq, Bind)]
pub struct UnlitMaterial {
    #[uniform]
    pub color: Vec3,
}

impl Default for UnlitMaterial {
    fn default() -> Self {
        Self {
            color: Vec3::new(1.0, 1.0, 1.0),
        }
    }
}

impl UnlitMaterial {
    #[inline]
    pub const fn new(color: Vec3) -> Self {
        Self { color }
    }
}

impl Material for UnlitMaterial {
    #[inline]
    fn vertex_shader() -> ShaderRef {
        ShaderRef::module("lumi/unlit.wgsl")
    }

    #[inline]
    fn fragment_shader() -> ShaderRef {
        ShaderRef::module("lumi/unlit.wgsl")
    }

    #[inline]
    fn specialize(pipeline: &mut MaterialPipeline) {
        pipeline.vertices = vec![
            MeshVertexLayout {
                attribute: Mesh::POSITION.into(),
                format: VertexFormat::Float32x3,
                location: 0,
            },
            MeshVertexLayout {
                attribute: Mesh::NORMAL.into(),
                format: VertexFormat::Float32x3,
                location: 1,
            },
        ];
    }
}
