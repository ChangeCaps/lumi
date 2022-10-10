mod node;
mod transform;

use std::any::TypeId;

pub use node::*;
pub use transform::*;

use crate::{
    camera::Camera,
    environment::Environment,
    id::{CameraId, LightId, NodeId, WorldId},
    light::{AmbientLight, AsLight, Light},
    renderable::{RegisterFn, Renderable},
    util::HashMap,
};

#[derive(Default)]
pub struct World {
    id: WorldId,
    environment: Environment,
    register_fns: HashMap<TypeId, RegisterFn>,
    nodes: HashMap<NodeId, Box<dyn Node>>,
    ambient_light: AmbientLight,
    lights: HashMap<LightId, Light>,
    cameras: HashMap<CameraId, Camera>,
}

impl World {
    pub fn new() -> Self {
        Self {
            id: WorldId::new(),
            environment: Environment::default(),
            register_fns: HashMap::default(),
            nodes: HashMap::default(),
            ambient_light: AmbientLight::default(),
            lights: HashMap::default(),
            cameras: HashMap::default(),
        }
    }

    pub fn id(&self) -> WorldId {
        self.id
    }

    pub fn environment(&self) -> &Environment {
        &self.environment
    }

    pub fn environment_mut(&mut self) -> &mut Environment {
        &mut self.environment
    }

    pub fn iter_nodes<T: Node>(&self) -> impl Iterator<Item = (NodeId, &T)> {
        self.nodes
            .iter()
            .filter_map(|(id, node)| node.downcast_ref().map(|node| (*id, node)))
    }

    pub fn iter_nodes_mut<T: Node>(&mut self) -> impl Iterator<Item = (NodeId, &mut T)> {
        self.nodes
            .iter_mut()
            .filter_map(|(id, node)| node.downcast_mut().map(|node| (*id, node)))
    }

    pub fn nodes<T: Node>(&self) -> impl Iterator<Item = &T> {
        self.nodes
            .values()
            .filter_map(|node| node.as_ref().downcast_ref::<T>())
    }

    pub fn nodes_mut<T: Node>(&mut self) -> impl Iterator<Item = &mut T> {
        self.nodes
            .values_mut()
            .filter_map(|node| node.as_mut().downcast_mut::<T>())
    }

    pub fn register_fns(&self) -> &HashMap<TypeId, RegisterFn> {
        &self.register_fns
    }

    pub fn iter_lights(&self) -> impl Iterator<Item = (LightId, &Light)> {
        self.lights.iter().map(|(id, light)| (*id, light))
    }

    pub fn iter_lights_mut(&mut self) -> impl Iterator<Item = (LightId, &mut Light)> {
        self.lights.iter_mut().map(|(id, light)| (*id, light))
    }

    pub fn lights(&self) -> impl Iterator<Item = &Light> {
        self.lights.values()
    }

    pub fn lights_mut(&mut self) -> impl Iterator<Item = &mut Light> {
        self.lights.values_mut()
    }

    pub fn iter_cameras(&self) -> impl Iterator<Item = (CameraId, &Camera)> {
        self.cameras.iter().map(|(id, camera)| (*id, camera))
    }

    pub fn iter_cameras_mut(&mut self) -> impl Iterator<Item = (CameraId, &mut Camera)> {
        self.cameras.iter_mut().map(|(id, camera)| (*id, camera))
    }

    pub fn cameras(&self) -> impl Iterator<Item = &Camera> {
        self.cameras.values()
    }

    pub fn cameras_mut(&mut self) -> impl Iterator<Item = &mut Camera> {
        self.cameras.values_mut()
    }
}

impl World {
    pub fn add<T: Node + Renderable>(&mut self, node: T) -> NodeId {
        let id = NodeId::new();
        self.register_fns.insert(TypeId::of::<T>(), T::register);
        self.nodes.insert(id, Box::new(node));
        id
    }

    pub fn remove<T: Node>(&mut self, id: NodeId) -> Option<T> {
        let node = self.nodes.remove(&id)?;

        if node.as_ref().type_id() == TypeId::of::<T>() {
            unsafe { Some(*Box::from_raw(Box::into_raw(node) as *mut T)) }
        } else {
            self.nodes.insert(id, node);
            None
        }
    }

    #[track_caller]
    pub fn node<T: Node>(&self, id: NodeId) -> &T {
        let node = self.nodes.get(&id).expect("Node not found");
        node.as_ref().downcast_ref().expect("Node type mismatch")
    }

    #[track_caller]
    pub fn node_mut<T: Node>(&mut self, id: NodeId) -> &mut T {
        let node = self.nodes.get_mut(&id).expect("Node not found");
        node.as_mut().downcast_mut().expect("Node type mismatch")
    }
}

impl World {
    pub fn ambient(&self) -> &AmbientLight {
        &self.ambient_light
    }

    pub fn ambient_mut(&mut self) -> &mut AmbientLight {
        &mut self.ambient_light
    }

    pub fn add_light(&mut self, light: impl Into<Light>) -> LightId {
        let entity = LightId::new();
        self.lights.insert(entity, light.into());
        entity
    }

    #[track_caller]
    pub fn light<T: AsLight>(&self, id: LightId) -> &T {
        T::as_light(self.lights.get(&id).expect("Light not found")).expect("light type mismatch")
    }

    #[track_caller]
    pub fn light_mut<T: AsLight>(&mut self, id: LightId) -> &mut T {
        T::as_light_mut(self.lights.get_mut(&id).expect("Light not found"))
            .expect("light type mismatch")
    }

    pub fn remove_light<T: AsLight>(&mut self, id: LightId) -> Option<T> {
        T::from_light(self.lights.remove(&id)?)
    }
}

impl World {
    pub fn add_camera(&mut self, camera: Camera) -> CameraId {
        let entity = CameraId::new();
        self.cameras.insert(entity, camera);
        entity
    }

    #[track_caller]
    pub fn camera(&self, id: CameraId) -> &Camera {
        self.cameras.get(&id).expect("Camera not found")
    }

    #[track_caller]
    pub fn camera_mut(&mut self, id: CameraId) -> &mut Camera {
        self.cameras.get_mut(&id).expect("Camera not found")
    }

    pub fn remove_camera(&mut self, entity: CameraId) -> Option<Camera> {
        self.cameras.remove(&entity)
    }

    pub fn camera_ids(&self) -> Vec<CameraId> {
        self.cameras.keys().copied().collect()
    }
}
