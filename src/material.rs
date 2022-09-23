use crate::{Bind, ShaderRef};

pub trait Material: Bind {
    fn vertex_shader() -> ShaderRef {
        ShaderRef::Default
    }

    fn fragment_shader() -> ShaderRef {
        ShaderRef::Default
    }
}
