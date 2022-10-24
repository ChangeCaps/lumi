use lumi_assets::Handle;
use lumi_core::{Device, Queue, Resources};

use crate::Node;

pub type RegisterFn = fn(&Device, &Queue, &mut Resources);

#[allow(unused_variables)]
pub trait Renderable: Node {
    fn register(device: &Device, queue: &Queue, resources: &mut Resources) {}
}

#[allow(unused_variables)]
pub trait HandleRenderable: Renderable {
    fn register_handle(device: &Device, queue: &Queue, resources: &mut Resources) {}
}

impl<T: HandleRenderable> Renderable for Handle<T> {
    fn register(device: &Device, queue: &Queue, resources: &mut Resources) {
        T::register_handle(device, queue, resources);
    }
}
