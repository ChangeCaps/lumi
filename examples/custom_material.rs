mod util;

use lumi::prelude::*;
use lumi::{bind, bind_key, binding};

#[derive(ShaderType)]
struct Foo {
    bar: f32,
}

#[derive(Bind)]
struct CustomMaterial;

impl Material for CustomMaterial {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::new("examples/assets/custom.wgsl")
    }
}

fn main() {}
