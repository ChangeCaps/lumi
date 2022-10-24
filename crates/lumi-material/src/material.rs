use std::borrow::Cow;

use lumi_bind::Bind;
use lumi_core::VertexFormat;
use lumi_mesh::Mesh;
use lumi_shader::{DefaultShader, Shader, ShaderDefs, ShaderDefsHash, ShaderRef};

#[derive(Clone, Debug)]
pub struct MeshVertexLayout {
    pub attribute: Cow<'static, str>,
    pub format: VertexFormat,
    pub location: u32,
}

#[derive(Debug)]
pub struct MaterialPipeline {
    pub vertex_shader: Shader,
    pub fragment_shader: Shader,
    pub vertices: Vec<MeshVertexLayout>,
}

impl MaterialPipeline {
    #[inline]
    pub fn rebind(&mut self) {
        self.vertex_shader
            .rebind_with(&mut self.fragment_shader)
            .unwrap();
    }
}

pub trait Material: Bind + Send + Sync + 'static {
    #[inline(always)]
    fn vertex_shader() -> ShaderRef {
        ShaderRef::Default(DefaultShader::Vertex)
    }

    #[inline(always)]
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Default(DefaultShader::Fragment)
    }

    #[inline(always)]
    fn shader_defs(&self) -> ShaderDefs {
        ShaderDefs::default()
    }

    #[inline(always)]
    fn shader_defs_hash(&self) -> ShaderDefsHash {
        self.shader_defs().hash()
    }

    #[inline(always)]
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
            MeshVertexLayout {
                attribute: Mesh::TANGENT.into(),
                format: VertexFormat::Float32x4,
                location: 2,
            },
            MeshVertexLayout {
                attribute: Mesh::UV_0.into(),
                format: VertexFormat::Float32x2,
                location: 3,
            },
        ];
    }

    #[inline(always)]
    fn is_translucent(&self) -> bool {
        false
    }

    #[inline(always)]
    fn use_ssr(&self) -> bool {
        false
    }
}
