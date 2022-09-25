use std::{any::Any, collections::LinkedList};

use crate::{Bind, BindingLayoutEntry, DefaultShader, ShaderRef};

pub trait Material: Bind {
    fn vertex_shader() -> ShaderRef {
        ShaderRef::Default(DefaultShader::Vertex)
    }

    fn fragment_shader() -> ShaderRef {
        ShaderRef::Default(DefaultShader::Fragment)
    }
}

pub trait DynMaterial: Bind + Any {
    fn vertex_shader(&self) -> ShaderRef;

    fn fragment_shader(&self) -> ShaderRef;

    fn entries(&self) -> LinkedList<BindingLayoutEntry>;
}

impl<T> DynMaterial for T
where
    T: Material + Any,
{
    fn vertex_shader(&self) -> ShaderRef {
        T::vertex_shader()
    }

    fn fragment_shader(&self) -> ShaderRef {
        T::fragment_shader()
    }

    fn entries(&self) -> LinkedList<BindingLayoutEntry> {
        Self::entries()
    }
}
