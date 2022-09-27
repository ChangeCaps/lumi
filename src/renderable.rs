use wgpu::RenderPass;

use crate::{
    resources::{Resource, Resources},
    world::Node,
    SharedDevice, SharedQueue,
};

pub struct RenderContext<'a> {
    pub device: &'a SharedDevice,
    pub queue: &'a SharedQueue,
}

#[allow(unused_variables)]
pub trait Renderable: Node {
    type Resource: Resource;
    type State: Resource;

    fn register(context: &RenderContext<'_>, resources: &mut Resources) -> Self::Resource;
    fn init(context: &RenderContext<'_>, resource: &Self::Resource) -> Self::State;
    fn prepare(
        &self,
        context: &RenderContext<'_>,
        resources: &mut Resources,
        resource: &Self::Resource,
        state: &mut Self::State,
    ) {
    }
    fn render<'a>(
        &self,
        context: &RenderContext<'_>,
        render_pass: &mut RenderPass<'a>,
        resources: &'a Resources,
        resource: &'a Self::Resource,
        state: &'a Self::State,
    );
}

pub struct RenderableResource<T: Renderable> {
    pub resource: T::Resource,
}

pub struct DynamicRenderable {
    register: fn(&RenderContext<'_>, &mut Resources),
    init: fn(&RenderContext<'_>, &Resources) -> Box<dyn Resource>,
    prepare: unsafe fn(&dyn Node, &RenderContext<'_>, &mut Resources, &mut dyn Resource),
    render: for<'a> unsafe fn(
        &dyn Node,
        &RenderContext<'_>,
        &mut RenderPass<'a>,
        &'a Resources,
        &'a dyn Resource,
    ),
}

impl DynamicRenderable {
    pub fn new<T: Renderable>() -> Self {
        Self {
            register: |context, resources| {
                if !resources.contains::<RenderableResource<T>>() {
                    let resource = T::register(context, resources);
                    let resource = RenderableResource::<T> { resource };
                    resources.insert(resource);
                }
            },
            init: |context, resources| {
                let resource = resources.get::<RenderableResource<T>>().unwrap();
                Box::new(T::init(context, &resource.resource))
            },
            prepare: |this, context, resources, state| {
                let resource = resources.remove::<RenderableResource<T>>().unwrap();
                T::prepare(
                    unsafe { this.downcast_ref_unchecked() },
                    context,
                    resources,
                    &resource.resource,
                    unsafe { state.downcast_mut() },
                );
                resources.insert(resource);
            },
            render: |this, context, render_pass, resources, state| {
                let resource = resources.get::<RenderableResource<T>>().unwrap();
                T::render(
                    unsafe { this.downcast_ref_unchecked() },
                    context,
                    render_pass,
                    resources,
                    &resource.resource,
                    unsafe { state.downcast_ref() },
                )
            },
        }
    }

    pub fn register(&self, context: &RenderContext<'_>, resources: &mut Resources) {
        (self.register)(context, resources);
    }

    pub fn init(&self, context: &RenderContext<'_>, resources: &Resources) -> Box<dyn Resource> {
        (self.init)(context, resources)
    }

    pub unsafe fn prepare(
        &self,
        this: &dyn Node,
        context: &RenderContext<'_>,
        resources: &mut Resources,
        state: &mut dyn Resource,
    ) {
        unsafe { (self.prepare)(this, context, resources, state) }
    }

    pub unsafe fn render<'a>(
        &self,
        this: &dyn Node,
        context: &RenderContext<'_>,
        render_pass: &mut RenderPass<'a>,
        resources: &'a Resources,
        state: &'a dyn Resource,
    ) {
        unsafe { (self.render)(this, context, render_pass, resources, state) }
    }
}
