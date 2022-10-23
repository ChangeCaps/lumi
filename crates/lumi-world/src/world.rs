use std::any::TypeId;

use lumi_id::{Id, IdMap, IdSet};
use lumi_util::{
    crossbeam::channel::{unbounded, Receiver, Sender},
    HashMap,
};

use crate::{AmbientLight, AsLight, Camera, Light, Node, RegisterFn, Renderable};

pub type WorldId = Id<World>;
pub type NodeId = Id<dyn Node>;
pub type LightId = Id<Light>;
pub type CameraId = Id<Camera>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WorldChange {
    Changed(Id),
    Removed(Id),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WorldChanges {
    pub added: IdSet,
    pub changed: IdSet,
    pub removed: IdSet,
}

impl WorldChanges {
    #[inline]
    pub fn clear(&mut self) {
        self.added.clear();
        self.changed.clear();
        self.removed.clear();
    }

    #[inline]
    pub fn push(&mut self, change: WorldChange) {
        match change {
            WorldChange::Changed(uuid) => {
                self.changed.insert(uuid);
            }
            WorldChange::Removed(uuid) => {
                self.removed.insert(uuid);
            }
        }
    }

    #[inline]
    pub fn added(&self) -> impl Iterator<Item = &Id> {
        self.added.iter()
    }

    #[inline]
    pub fn changed(&self) -> impl Iterator<Item = &Id> {
        self.changed.iter()
    }

    #[inline]
    pub fn removed(&self) -> impl Iterator<Item = &Id> {
        self.removed.iter()
    }

    #[inline]
    pub fn dyn_changed_nodes<'a>(
        &'a self,
        world: &'a World,
    ) -> impl Iterator<Item = (NodeId, &dyn Node)> {
        self.changed()
            .filter_map(move |&id| Some((id.cast(), world.get_dyn_node(id.cast())?)))
    }

    #[inline]
    pub fn changed_nodes<'a, T: Node>(
        &'a self,
        world: &'a World,
    ) -> impl Iterator<Item = (NodeId, &T)> {
        self.changed()
            .filter_map(move |&id| Some((id.cast(), world.get_node(id.cast())?)))
    }
}

pub struct World {
    id: Id<World>,
    change_sender: Sender<WorldChange>,
    change_receiver: Receiver<WorldChange>,
    register_fns: HashMap<TypeId, RegisterFn>,
    nodes: IdMap<dyn Node, Box<dyn Node>>,
    ambient_light: AmbientLight,
    lights: IdMap<Light>,
    cameras: IdMap<Camera>,
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
        let (change_sender, change_receiver) = unbounded();

        Self {
            id: Id::new(),
            change_sender,
            change_receiver,
            register_fns: HashMap::default(),
            nodes: IdMap::default(),
            ambient_light: AmbientLight::default(),
            lights: IdMap::default(),
            cameras: IdMap::default(),
        }
    }

    #[inline]
    pub fn id(&self) -> Id<World> {
        self.id
    }

    #[inline]
    pub fn subscribe_changes(&self) -> Receiver<WorldChange> {
        self.change_receiver.clone()
    }

    #[inline]
    pub fn iter_nodes<T: Node>(&self) -> impl Iterator<Item = (NodeId, &T)> {
        self.nodes
            .iter()
            .filter_map(|(&id, node)| node.downcast_ref().map(|node| (id.cast(), node)))
    }

    #[inline]
    pub fn iter_nodes_mut<T: Node>(&mut self) -> impl Iterator<Item = (NodeId, &mut T)> {
        self.nodes.iter_mut().filter_map(|(&id, node)| {
            let node = node.downcast_mut().map(|node| (id, node));

            if node.is_none() {
                self.change_sender
                    .send(WorldChange::Removed(id.cast()))
                    .unwrap();
            }

            node
        })
    }

    #[inline]
    pub fn node_ids(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.nodes.keys().copied().map(Id::cast)
    }

    #[inline]
    pub fn nodes<T: Node>(&self) -> impl Iterator<Item = &T> {
        self.nodes
            .values()
            .filter_map(|node| node.as_ref().downcast_ref::<T>())
    }

    #[inline]
    pub fn nodes_mut<T: Node>(&mut self) -> impl Iterator<Item = &mut T> {
        self.nodes.iter_mut().filter_map(|(&id, node)| {
            let node = node.as_mut().downcast_mut::<T>();

            if node.is_some() {
                self.change_sender
                    .send(WorldChange::Changed(id.cast()))
                    .unwrap();
            }

            node
        })
    }

    #[inline]
    pub fn register_fns(&self) -> &HashMap<TypeId, RegisterFn> {
        &self.register_fns
    }

    #[inline]
    pub fn iter_lights(&self) -> impl Iterator<Item = (LightId, &Light)> {
        self.lights.iter().map(|(&id, light)| (id, light))
    }

    #[inline]
    pub fn iter_lights_mut(&mut self) -> impl Iterator<Item = (LightId, &mut Light)> {
        self.lights.iter_mut().map(|(&id, light)| {
            self.change_sender
                .send(WorldChange::Changed(id.cast()))
                .unwrap();

            (id, light)
        })
    }

    #[inline]
    pub fn light_ids(&self) -> impl Iterator<Item = LightId> + '_ {
        self.lights.keys().copied().map(Id::cast)
    }

    #[inline]
    pub fn lights(&self) -> impl Iterator<Item = &Light> {
        self.lights.values()
    }

    #[inline]
    pub fn lights_mut(&mut self) -> impl Iterator<Item = &mut Light> {
        self.lights.iter_mut().map(|(&id, light)| {
            self.change_sender
                .send(WorldChange::Changed(id.cast()))
                .unwrap();

            light
        })
    }

    #[inline]
    pub fn camera_ids(&self) -> impl Iterator<Item = CameraId> + '_ {
        self.cameras.keys().copied()
    }

    #[inline]
    pub fn iter_cameras(&self) -> impl Iterator<Item = (CameraId, &Camera)> {
        self.cameras.iter().map(|(&id, camera)| (id, camera))
    }

    #[inline]
    pub fn iter_cameras_mut(&mut self) -> impl Iterator<Item = (CameraId, &mut Camera)> {
        self.cameras.iter_mut().map(|(&id, camera)| {
            self.change_sender
                .send(WorldChange::Changed(id.cast()))
                .unwrap();

            (id, camera)
        })
    }

    #[inline]
    pub fn cameras(&self) -> impl Iterator<Item = &Camera> {
        self.cameras.values()
    }

    #[inline]
    pub fn cameras_mut(&mut self) -> impl Iterator<Item = &mut Camera> {
        self.cameras.iter_mut().map(|(&id, camera)| {
            self.change_sender
                .send(WorldChange::Changed(id.cast()))
                .unwrap();

            camera
        })
    }

    pub fn set_lights(&mut self, lights: IdMap<Light>) {
        self.lights = lights;
    }
}

impl World {
    #[inline]
    pub fn add<T: Node + Renderable>(&mut self, node: T) -> NodeId {
        let id = Id::new();
        self.register_fns.insert(TypeId::of::<T>(), T::register);
        self.nodes.insert(id, Box::new(node));
        id
    }

    #[inline]
    pub fn remove<T: Node>(&mut self, id: NodeId) -> Option<T> {
        let node = self.nodes.remove(&id)?;

        if node.as_ref().type_id() == TypeId::of::<T>() {
            self.change_sender
                .send(WorldChange::Removed(id.cast()))
                .unwrap();

            unsafe { Some(*Box::from_raw(Box::into_raw(node) as *mut T)) }
        } else {
            self.nodes.insert(id, node);
            None
        }
    }

    #[inline]
    pub fn get_dyn_node(&self, id: NodeId) -> Option<&dyn Node> {
        self.nodes.get(&id).map(|node| node.as_ref())
    }

    #[inline]
    pub fn get_dyn_node_mut(&mut self, id: NodeId) -> Option<&mut dyn Node> {
        self.nodes.get_mut(&id).map(|node| {
            self.change_sender
                .send(WorldChange::Changed(id.cast()))
                .unwrap();

            node.as_mut()
        })
    }

    #[inline]
    #[track_caller]
    pub fn dyn_node(&self, id: NodeId) -> &dyn Node {
        self.get_dyn_node(id)
            .unwrap_or_else(|| panic!("Node with id {} does not exist in world", id))
    }

    #[inline]
    #[track_caller]
    pub fn dyn_node_mut(&mut self, id: NodeId) -> &mut dyn Node {
        self.get_dyn_node_mut(id)
            .unwrap_or_else(|| panic!("Node with id {} does not exist in world", id))
    }

    #[inline]
    pub fn get_node<T: Node>(&self, id: NodeId) -> Option<&T> {
        self.get_dyn_node(id)?.downcast_ref()
    }

    #[inline]
    pub fn get_node_mut<T: Node>(&mut self, id: NodeId) -> Option<&mut T> {
        let node = self.nodes.get_mut(&id)?.downcast_mut()?;

        self.change_sender
            .send(WorldChange::Changed(id.cast()))
            .unwrap();

        Some(node)
    }

    #[inline]
    #[track_caller]
    pub fn node<T: Node>(&self, id: NodeId) -> &T {
        self.dyn_node(id)
            .downcast_ref()
            .expect("Node type mismatch")
    }

    #[inline]
    #[track_caller]
    pub fn node_mut<T: Node>(&mut self, id: NodeId) -> &mut T {
        self.dyn_node_mut(id)
            .downcast_mut()
            .expect("Node type mismatch")
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
        let entity = Id::new();
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
        let light = T::as_light_mut(self.lights.get_mut(&id).expect("Light not found"))
            .expect("light type mismatch");

        self.change_sender
            .send(WorldChange::Changed(id.cast()))
            .unwrap();

        light
    }

    #[inline]
    pub fn remove_light<T: AsLight>(&mut self, id: LightId) -> Option<T> {
        if let Some(light) = T::from_light(self.lights.remove(&id)?) {
            self.change_sender
                .send(WorldChange::Removed(id.cast()))
                .unwrap();

            Some(light)
        } else {
            None
        }
    }
}

impl World {
    #[inline]
    pub fn add_camera(&mut self, camera: Camera) -> CameraId {
        let entity = Id::new();
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
        let camera = self.cameras.get_mut(&id).expect("Camera not found");

        self.change_sender
            .send(WorldChange::Changed(id.cast()))
            .unwrap();

        camera
    }

    #[inline]
    pub fn remove_camera(&mut self, entity: CameraId) -> Option<Camera> {
        let camera = self.cameras.remove(&entity);

        if camera.is_some() {
            self.change_sender
                .send(WorldChange::Removed(entity.cast()))
                .unwrap();
        }

        camera
    }
}
