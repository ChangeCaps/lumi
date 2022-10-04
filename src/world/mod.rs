mod node;
mod transform;

use std::collections::HashMap;

pub use node::*;
pub use transform::*;

use crate::{
    camera::Camera,
    environment::Environment,
    id::{CameraId, LightId, NodeId},
    light::{AmbientLight, AsLight, Light},
    renderable::Renderable,
};

#[derive(Default)]
pub struct World {
    environment: Environment,
    nodes: NodeStorage,
    ambient_light: AmbientLight,
    lights: HashMap<LightId, Light>,
    cameras: HashMap<CameraId, Camera>,
}

impl World {
    pub fn new() -> Self {
        Self {
            environment: Environment::default(),
            nodes: NodeStorage::new(),
            ambient_light: AmbientLight::default(),
            lights: HashMap::new(),
            cameras: HashMap::new(),
        }
    }

    pub fn environment(&self) -> &Environment {
        &self.environment
    }

    pub fn iter_nodes(&self) -> impl Iterator<Item = (NodeId, &dyn Node)> {
        self.nodes.iter()
    }

    pub fn nodes(&self) -> impl Iterator<Item = &dyn Node> {
        self.nodes.nodes()
    }

    pub fn iter_renderables(&self) -> impl Iterator<Item = (NodeId, RenderableNode<'_>)> {
        self.nodes.iter_renderables()
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
    pub fn node_storage(&self) -> &NodeStorage {
        &self.nodes
    }

    pub fn add<T: Node + Renderable>(&mut self, node: T) -> NodeId {
        self.nodes.insert_renderable(node)
    }

    pub fn remove<T: Node>(&mut self, id: NodeId) -> Option<T> {
        self.nodes.remove(id)
    }

    #[track_caller]
    pub fn node<T: Node>(&self, id: NodeId) -> &T {
        self.nodes.get(id).expect("Node not found")
    }

    #[track_caller]
    pub fn node_mut<T: Node>(&mut self, id: NodeId) -> &mut T {
        self.nodes.get_mut(id).expect("Node not found")
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
