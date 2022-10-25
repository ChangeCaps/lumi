use lumi_bind::Bind;
use lumi_core::{Resources, UniformBuffer};
use lumi_util::math::Mat4;
use lumi_world::{ExtractOne, Node, World};

use crate::{PhaseContext, RenderPhase};

#[derive(Debug, Default, Bind)]
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
        T: Node + ExtractOne<Mat4>,
    {
        Self {
            prepare: Self::prepare_default::<T>,
        }
    }

    fn prepare_default<T>(context: &PhaseContext, world: &World, resources: &mut Resources)
    where
        T: Node + ExtractOne<Mat4>,
    {
        for (id, node) in world.iter_nodes::<T>() {
            if let Some(&transform) = node.extract_one() {
                let prepared_transform =
                    resources.get_id_or_default::<PreparedTransform>(id.cast());
                prepared_transform.transform.set(transform);
            }
        }

        for id in context.changes.removed() {
            resources.remove_id::<PreparedTransform>(id);
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
