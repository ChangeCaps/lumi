use std::ops::{Deref, DerefMut};

use lumi_bind::Bind;
use lumi_core::{Resources, UniformBuffer};
use lumi_util::{math::Mat4, smallvec::SmallVec};
use lumi_world::{Extract, Node, World};

use crate::{PhaseContext, RenderPhase};

#[derive(Debug, Default)]
pub struct PreparedTransforms {
    pub transforms: SmallVec<[PreparedTransform; 1]>,
}

impl Deref for PreparedTransforms {
    type Target = SmallVec<[PreparedTransform; 1]>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.transforms
    }
}

impl DerefMut for PreparedTransforms {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.transforms
    }
}

#[derive(Debug, Bind)]
pub struct PreparedTransform {
    #[uniform]
    pub transform: UniformBuffer<Mat4>,
}

pub struct PrepareTransformFunction {
    prepare: fn(&PhaseContext, &World, &mut Resources),
}

impl PrepareTransformFunction {
    #[inline]
    pub fn new<T>() -> Self
    where
        T: Node + Extract<Mat4>,
    {
        Self {
            prepare: Self::prepare_default::<T>,
        }
    }

    fn prepare_default<T>(context: &PhaseContext, world: &World, resources: &mut Resources)
    where
        T: Node + Extract<Mat4>,
    {
        for (id, node) in context.changes.changed_nodes::<T>(world) {
            let prepared_transforms = resources.get_id_or_default::<PreparedTransforms>(id.cast());

            let mut i = 0;
            node.extract(&mut |&transform| {
                if let Some(prepared) = prepared_transforms.get_mut(i) {
                    prepared.transform.set(transform);
                } else {
                    let prepared = PreparedTransform {
                        transform: UniformBuffer::new(transform),
                    };

                    prepared_transforms.push(prepared);
                }

                i += 1;
            });
        }

        for id in context.changes.removed() {
            resources.remove_id::<PreparedTransforms>(id);
        }
    }

    #[inline]
    pub fn prepare(&self, context: &PhaseContext, world: &World, resources: &mut Resources) {
        (self.prepare)(context, world, resources)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PrepareTransforms;

impl RenderPhase for PrepareTransforms {
    fn prepare(&mut self, context: &PhaseContext, world: &World, resources: &mut Resources) {
        let functions = resources.remove_id_map::<PrepareTransformFunction>();

        for function in functions.values() {
            function.prepare(context, world, resources);
        }

        resources.insert(functions);
    }
}
