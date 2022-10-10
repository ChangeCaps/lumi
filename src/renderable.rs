use crate::{resources::Resources, world::Node, Device, Queue};

pub type RegisterFn = fn(&Device, &Queue, &mut Resources);

#[allow(unused_variables)]
pub trait Renderable: Node {
    fn register(device: &Device, queue: &Queue, resources: &mut Resources) {}
}
