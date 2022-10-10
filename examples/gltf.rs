mod util;

use lumi::prelude::*;

fn main() {
    let mesh = MeshNode::open_gltf("untitled.gltf").unwrap();

    println!("{:#?}", mesh.primitives[0].mesh);
}
