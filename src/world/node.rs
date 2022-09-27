use std::{any::TypeId, collections::HashMap};

use wgpu::RenderPass;

use crate::{
    id::NodeId,
    renderable::{DynamicRenderable, RenderContext, Renderable},
    resources::{Resource, Resources},
};

pub unsafe trait Node: Send + Sync + 'static {
    fn type_id(&self) -> TypeId;
}
unsafe impl<T: Send + Sync + 'static> Node for T {
    fn type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }
}

impl dyn Node {
    pub unsafe fn downcast_ref_unchecked<T: Node>(&self) -> &T {
        unsafe { &*(self as *const dyn Node as *const T) }
    }

    pub unsafe fn downcast_mut_unchecked<T: Node>(&mut self) -> &mut T {
        unsafe { &mut *(self as *mut dyn Node as *mut T) }
    }

    pub fn downcast_ref<T: Node>(&self) -> Option<&T> {
        if Node::type_id(self) == TypeId::of::<T>() {
            Some(unsafe { &*(self as *const dyn Node as *const T) })
        } else {
            None
        }
    }

    pub fn downcast_mut<T: Node>(&mut self) -> Option<&mut T> {
        if Node::type_id(self) == TypeId::of::<T>() {
            Some(unsafe { &mut *(self as *mut dyn Node as *mut T) })
        } else {
            None
        }
    }
}

#[derive(Default)]
pub struct NodeStorage {
    dynamics: HashMap<TypeId, DynamicRenderable>,
    nodes: HashMap<NodeId, Box<dyn Node>>,
}

impl NodeStorage {
    pub fn new() -> Self {
        Self {
            dynamics: HashMap::new(),
            nodes: HashMap::new(),
        }
    }

    pub fn register_renderable<T: Node + Renderable>(&mut self) {
        if !self.dynamics.contains_key(&TypeId::of::<T>()) {
            self.dynamics
                .insert(TypeId::of::<T>(), DynamicRenderable::new::<T>());
        }
    }

    pub fn insert<T: Node>(&mut self, node: T) -> NodeId {
        let id = NodeId::new();
        self.nodes.insert(id, Box::new(node));
        id
    }

    pub fn insert_renderable<T: Node + Renderable>(&mut self, node: T) -> NodeId {
        self.register_renderable::<T>();
        self.insert(node)
    }

    pub fn remove<T: Node>(&mut self, id: NodeId) -> Option<T> {
        let node = self.nodes.remove(&id)?;

        if node.as_ref().type_id() == TypeId::of::<T>() {
            Some(*unsafe { Box::from_raw(Box::into_raw(node) as *mut T) })
        } else {
            self.nodes.insert(id, node);
            None
        }
    }

    pub fn get<T: Node>(&self, id: NodeId) -> Option<&T> {
        self.nodes.get(&id)?.downcast_ref()
    }

    pub fn get_mut<T: Node>(&mut self, id: NodeId) -> Option<&mut T> {
        self.nodes.get_mut(&id)?.downcast_mut()
    }

    pub fn get_renderable<'a>(&'a self, id: NodeId) -> Option<RenderableNode<'a>> {
        let node = self.nodes.get(&id)?;
        let renderable = self.dynamics.get(&node.type_id())?;

        Some(RenderableNode {
            node: node.as_ref(),
            renderable,
        })
    }

    pub fn init(&mut self, context: &RenderContext<'_>, resources: &mut Resources) {
        for renderable in self.dynamics.values_mut() {
            renderable.init(context, resources);
        }
    }

    pub fn nodes(&self) -> impl Iterator<Item = &dyn Node> {
        self.nodes.values().map(|node| node.as_ref())
    }

    pub fn iter(&self) -> impl Iterator<Item = (NodeId, &dyn Node)> {
        self.nodes.iter().map(|(id, node)| (*id, node.as_ref()))
    }

    pub fn iter_renderables<'a>(&'a self) -> impl Iterator<Item = (NodeId, RenderableNode<'a>)> {
        self.nodes.iter().filter_map(move |(id, node)| {
            let renderable = self.dynamics.get(&node.as_ref().type_id())?;
            Some((
                *id,
                RenderableNode {
                    node: node.as_ref(),
                    renderable,
                },
            ))
        })
    }

    pub fn dynamic_renderables(&self) -> impl Iterator<Item = &DynamicRenderable> {
        self.dynamics.values()
    }
}

pub struct RenderableNode<'a> {
    node: &'a dyn Node,
    renderable: &'a DynamicRenderable,
}

impl<'a> RenderableNode<'a> {
    pub fn register(&self, context: &RenderContext<'_>, resources: &mut Resources) {
        self.renderable.register(context, resources);
    }

    pub fn init(
        &self,
        context: &RenderContext<'_>,
        resources: &mut Resources,
    ) -> Box<dyn Resource> {
        self.renderable.init(context, resources)
    }

    pub unsafe fn prepare(
        &self,
        context: &RenderContext<'_>,
        resources: &mut Resources,
        state: &mut dyn Resource,
    ) {
        unsafe {
            self.renderable
                .prepare(self.node, context, resources, state)
        };
    }

    pub unsafe fn render(
        &self,
        context: &RenderContext<'_>,
        render_pass: &mut RenderPass<'a>,
        resources: &'a Resources,
        state: &'a dyn Resource,
    ) {
        unsafe {
            self.renderable
                .render(self.node, context, render_pass, resources, state)
        };
    }
}
