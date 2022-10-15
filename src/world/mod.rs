mod node;
mod transform;

use std::any::TypeId;

use crossbeam::channel::{Receiver, Sender};
pub use node::*;
pub use transform::*;

use crate::{
    camera::Camera,
    environment::Environment,
    id::{CameraId, LightId, NodeId, WorldId},
    light::{AmbientLight, AsLight, Light},
    renderable::{RegisterFn, Renderable},
    util::{HashMap, HashSet},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WorldChange {
    Node(NodeId),
    Camera(CameraId),
    Light(LightId),
    NodeRemoved(NodeId),
    CameraRemoved(CameraId),
    LightRemoved(LightId),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WorldChanges {
    nodes_added: HashSet<NodeId>,
    cameras_added: HashSet<CameraId>,
    lights_added: HashSet<LightId>,
    nodes_changed: HashSet<NodeId>,
    cameras_changed: HashSet<CameraId>,
    lights_changed: HashSet<LightId>,
    nodes_removed: HashSet<NodeId>,
    cameras_removed: HashSet<CameraId>,
    lights_removed: HashSet<LightId>,
}

impl WorldChanges {
    #[inline]
    pub fn clear(&mut self) {
        self.nodes_added.clear();
        self.cameras_added.clear();
        self.lights_added.clear();
        self.nodes_changed.clear();
        self.cameras_changed.clear();
        self.lights_changed.clear();
        self.nodes_removed.clear();
        self.cameras_removed.clear();
        self.lights_removed.clear();
    }

    #[inline]
    pub fn push_added_node(&mut self, id: NodeId) {
        self.nodes_added.insert(id);
    }

    #[inline]
    pub fn push_added_camera(&mut self, id: CameraId) {
        self.cameras_added.insert(id);
    }

    #[inline]
    pub fn push_added_light(&mut self, id: LightId) {
        self.lights_added.insert(id);
    }

    #[inline]
    pub fn push(&mut self, change: WorldChange) {
        match change {
            WorldChange::Node(id) => {
                self.nodes_changed.insert(id);
            }
            WorldChange::Camera(id) => {
                self.cameras_changed.insert(id);
            }
            WorldChange::Light(id) => {
                self.lights_changed.insert(id);
            }
            WorldChange::NodeRemoved(id) => {
                self.nodes_removed.insert(id);
            }
            WorldChange::CameraRemoved(id) => {
                self.cameras_removed.insert(id);
            }
            WorldChange::LightRemoved(id) => {
                self.lights_removed.insert(id);
            }
        }
    }
}

pub struct World {
    id: WorldId,
    environment: Environment,
    change_sender: Sender<WorldChange>,
    change_receiver: Receiver<WorldChange>,
    register_fns: HashMap<TypeId, RegisterFn>,
    nodes: HashMap<NodeId, Box<dyn Node>>,
    ambient_light: AmbientLight,
    lights: HashMap<LightId, Light>,
    cameras: HashMap<CameraId, Camera>,
}

impl Default for World {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    #[inline]
    pub fn new() -> Self {
        let (change_sender, change_receiver) = crossbeam::channel::unbounded();

        Self {
            id: WorldId::new(),
            environment: Environment::default(),
            change_sender,
            change_receiver,
            register_fns: HashMap::default(),
            nodes: HashMap::default(),
            ambient_light: AmbientLight::default(),
            lights: HashMap::default(),
            cameras: HashMap::default(),
        }
    }

    #[inline]
    pub fn id(&self) -> WorldId {
        self.id
    }

    #[inline]
    pub fn subscribe_changes(&self) -> Receiver<WorldChange> {
        self.change_receiver.clone()
    }

    #[inline]
    pub fn environment(&self) -> &Environment {
        &self.environment
    }

    #[inline]
    pub fn environment_mut(&mut self) -> &mut Environment {
        &mut self.environment
    }

    #[inline]
    pub fn iter_nodes<T: Node>(&self) -> impl Iterator<Item = (NodeId, &T)> {
        self.nodes
            .iter()
            .filter_map(|(id, node)| node.downcast_ref().map(|node| (*id, node)))
    }

    #[inline]
    pub fn iter_nodes_mut<T: Node>(&mut self) -> impl Iterator<Item = (NodeId, &mut T)> {
        self.nodes
            .iter_mut()
            .filter_map(|(id, node)| node.downcast_mut().map(|node| (*id, node)))
    }

    #[inline]
    pub fn node_ids(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.nodes.keys().copied()
    }

    #[inline]
    pub fn nodes<T: Node>(&self) -> impl Iterator<Item = &T> {
        self.nodes
            .values()
            .filter_map(|node| node.as_ref().downcast_ref::<T>())
    }

    #[inline]
    pub fn nodes_mut<T: Node>(&mut self) -> impl Iterator<Item = &mut T> {
        self.nodes
            .values_mut()
            .filter_map(|node| node.as_mut().downcast_mut::<T>())
    }

    #[inline]
    pub fn register_fns(&self) -> &HashMap<TypeId, RegisterFn> {
        &self.register_fns
    }

    #[inline]
    pub fn iter_lights(&self) -> impl Iterator<Item = (LightId, &Light)> {
        self.lights.iter().map(|(id, light)| (*id, light))
    }

    #[inline]
    pub fn iter_lights_mut(&mut self) -> impl Iterator<Item = (LightId, &mut Light)> {
        self.lights.iter_mut().map(|(id, light)| (*id, light))
    }

    #[inline]
    pub fn light_ids(&self) -> impl Iterator<Item = LightId> + '_ {
        self.lights.keys().copied()
    }

    #[inline]
    pub fn lights(&self) -> impl Iterator<Item = &Light> {
        self.lights.values()
    }

    #[inline]
    pub fn lights_mut(&mut self) -> impl Iterator<Item = &mut Light> {
        self.lights.values_mut()
    }

    #[inline]
    pub fn camera_ids(&self) -> impl Iterator<Item = CameraId> + '_ {
        self.cameras.keys().copied()
    }

    #[inline]
    pub fn iter_cameras(&self) -> impl Iterator<Item = (CameraId, &Camera)> {
        self.cameras.iter().map(|(id, camera)| (*id, camera))
    }

    #[inline]
    pub fn iter_cameras_mut(&mut self) -> impl Iterator<Item = (CameraId, &mut Camera)> {
        self.cameras.iter_mut().map(|(id, camera)| (*id, camera))
    }

    #[inline]
    pub fn cameras(&self) -> impl Iterator<Item = &Camera> {
        self.cameras.values()
    }

    #[inline]
    pub fn cameras_mut(&mut self) -> impl Iterator<Item = &mut Camera> {
        self.cameras.values_mut()
    }
}

impl World {
    #[inline]
    pub fn add<T: Node + Renderable>(&mut self, node: T) -> NodeId {
        let id = NodeId::new();
        self.register_fns.insert(TypeId::of::<T>(), T::register);
        self.nodes.insert(id, Box::new(node));
        id
    }

    #[inline]
    pub fn remove<T: Node>(&mut self, id: NodeId) -> Option<T> {
        let node = self.nodes.remove(&id)?;

        if node.as_ref().type_id() == TypeId::of::<T>() {
            unsafe { Some(*Box::from_raw(Box::into_raw(node) as *mut T)) }
        } else {
            self.nodes.insert(id, node);
            None
        }
    }

    #[inline]
    #[track_caller]
    pub fn node<T: Node>(&self, id: NodeId) -> &T {
        let node = self.nodes.get(&id).expect("Node not found");
        node.as_ref().downcast_ref().expect("Node type mismatch")
    }

    #[inline]
    #[track_caller]
    pub fn node_mut<T: Node>(&mut self, id: NodeId) -> &mut T {
        let node = self.nodes.get_mut(&id).expect("Node not found");
        node.as_mut().downcast_mut().expect("Node type mismatch")
    }
}

impl World {
    #[inline]
    pub fn ambient(&self) -> &AmbientLight {
        &self.ambient_light
    }

    #[inline]
    pub fn ambient_mut(&mut self) -> &mut AmbientLight {
        &mut self.ambient_light
    }

    #[inline]
    pub fn add_light(&mut self, light: impl Into<Light>) -> LightId {
        let entity = LightId::new();
        self.lights.insert(entity, light.into());
        entity
    }

    #[inline]
    #[track_caller]
    pub fn light<T: AsLight>(&self, id: LightId) -> &T {
        T::as_light(self.lights.get(&id).expect("Light not found")).expect("light type mismatch")
    }

    #[inline]
    #[track_caller]
    pub fn light_mut<T: AsLight>(&mut self, id: LightId) -> &mut T {
        T::as_light_mut(self.lights.get_mut(&id).expect("Light not found"))
            .expect("light type mismatch")
    }

    #[inline]
    pub fn remove_light<T: AsLight>(&mut self, id: LightId) -> Option<T> {
        T::from_light(self.lights.remove(&id)?)
    }
}

impl World {
    #[inline]
    pub fn add_camera(&mut self, camera: Camera) -> CameraId {
        let entity = CameraId::new();
        self.cameras.insert(entity, camera);
        entity
    }

    #[inline]
    #[track_caller]
    pub fn camera(&self, id: CameraId) -> &Camera {
        self.cameras.get(&id).expect("Camera not found")
    }

    #[inline]
    #[track_caller]
    pub fn camera_mut(&mut self, id: CameraId) -> &mut Camera {
        self.cameras.get_mut(&id).expect("Camera not found")
    }

    #[inline]
    pub fn remove_camera(&mut self, entity: CameraId) -> Option<Camera> {
        self.cameras.remove(&entity)
    }
}
