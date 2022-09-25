use std::{any::TypeId, collections::HashMap};

use glam::{Mat4, Vec3};

use crate::{AsLight, Camera, CameraId, DynMaterial, Light, LightId, Material, Mesh, NodeId};

pub struct Node {
    pub material: Box<dyn DynMaterial>,
    pub mesh: Mesh,
    pub transform: Mat4,
}

impl Node {
    pub fn new<T>(material: T, mesh: Mesh) -> Self
    where
        T: Material + 'static,
    {
        Self {
            material: Box::new(material),
            mesh,
            transform: Mat4::IDENTITY,
        }
    }

    pub fn material<T>(&self) -> Option<&T>
    where
        T: Material + 'static,
    {
        if self.material.type_id() == TypeId::of::<T>() {
            Some(unsafe { &*(self.material.as_ref() as *const dyn DynMaterial as *const T) })
        } else {
            None
        }
    }

    pub fn material_mut<T>(&mut self) -> Option<&mut T>
    where
        T: Material + 'static,
    {
        if self.material.type_id() == TypeId::of::<T>() {
            Some(unsafe { &mut *(self.material.as_mut() as *mut dyn DynMaterial as *mut T) })
        } else {
            None
        }
    }

    pub fn into_material<T>(self) -> Option<T>
    where
        T: Material + 'static,
    {
        if self.material.type_id() == TypeId::of::<T>() {
            Some(unsafe { *Box::from_raw(Box::into_raw(self.material) as *mut T) })
        } else {
            None
        }
    }

    pub fn with_transform(mut self, transform: Mat4) -> Self {
        self.transform = transform;
        self
    }

    pub fn with_position(mut self, position: Vec3) -> Self {
        self.transform = Mat4::from_translation(position);
        self
    }
}

pub trait Resource: Send + Sync + 'static {}

#[derive(Default)]
pub struct Resources {
    resources: HashMap<TypeId, Box<dyn Resource>>,
}

impl Resources {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert<T: Resource>(&mut self, resource: T) {
        self.resources.insert(TypeId::of::<T>(), Box::new(resource));
    }

    pub fn remove<T: Resource>(&mut self) -> Option<T> {
        let resource = self.resources.remove(&TypeId::of::<T>())?;

        Some(unsafe { *Box::from_raw(Box::into_raw(resource) as *mut _) })
    }

    pub fn get<T: Resource>(&self) -> Option<&T> {
        let resource = self.resources.get(&TypeId::of::<T>())?;

        Some(unsafe { &*(resource.as_ref() as *const dyn Resource as *const T) })
    }

    pub fn get_mut<T: Resource>(&mut self) -> Option<&mut T> {
        let resource = self.resources.get_mut(&TypeId::of::<T>())?;

        Some(unsafe { &mut *(resource.as_mut() as *mut dyn Resource as *mut T) })
    }
}

pub struct World {
    resources: Resources,
    nodes: HashMap<NodeId, Node>,
    lights: HashMap<LightId, Light>,
    cameras: HashMap<CameraId, Camera>,
}

impl World {
    pub fn new() -> Self {
        Self {
            resources: Resources::new(),
            nodes: HashMap::new(),
            lights: HashMap::new(),
            cameras: HashMap::new(),
        }
    }

    pub fn iter_nodes(&self) -> impl Iterator<Item = (NodeId, &Node)> {
        self.nodes.iter().map(|(id, node)| (*id, node))
    }

    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.values()
    }

    pub fn iter_lights(&self) -> impl Iterator<Item = (LightId, &Light)> {
        self.lights.iter().map(|(id, light)| (*id, light))
    }

    pub fn lights(&self) -> impl Iterator<Item = &Light> {
        self.lights.values()
    }

    pub fn iter_cameras(&self) -> impl Iterator<Item = (CameraId, &Camera)> {
        self.cameras.iter().map(|(id, camera)| (*id, camera))
    }

    pub fn cameras(&self) -> impl Iterator<Item = &Camera> {
        self.cameras.values()
    }
}

impl World {
    pub fn resources(&self) -> &Resources {
        &self.resources
    }

    pub fn resources_mut(&mut self) -> &mut Resources {
        &mut self.resources
    }

    pub fn add_resource<T: Resource>(&mut self, resource: T) {
        self.resources.insert(resource);
    }

    #[track_caller]
    pub fn resource<T: Resource>(&self) -> &T {
        self.resources.get().expect("resource not found")
    }

    #[track_caller]
    pub fn resource_mut<T: Resource>(&mut self) -> &mut T {
        self.resources.get_mut().expect("resource not found")
    }

    pub fn remove_resource<T: Resource>(&mut self) -> Option<T> {
        self.resources.remove()
    }
}

impl World {
    pub fn add_node(&mut self, node: Node) -> NodeId {
        let entity = NodeId::new();
        self.nodes.insert(entity, node);
        entity
    }

    #[track_caller]
    pub fn node(&self, id: NodeId) -> &Node {
        self.nodes.get(&id).expect("Node not found")
    }

    #[track_caller]
    pub fn node_mut(&mut self, id: NodeId) -> &mut Node {
        self.nodes.get_mut(&id).expect("Node not found")
    }

    #[track_caller]
    pub fn material<T>(&self, id: NodeId) -> &T
    where
        T: Material + 'static,
    {
        self.node(id).material().expect("Material type mismatch")
    }

    #[track_caller]
    pub fn material_mut<T>(&mut self, id: NodeId) -> &mut T
    where
        T: Material + 'static,
    {
        self.node_mut(id)
            .material_mut()
            .expect("Material type mismatch")
    }

    #[track_caller]
    pub fn transform(&self, id: NodeId) -> Mat4 {
        self.node(id).transform
    }

    #[track_caller]
    pub fn transform_mut(&mut self, id: NodeId) -> &mut Mat4 {
        &mut self.node_mut(id).transform
    }

    #[track_caller]
    pub fn mesh(&self, id: NodeId) -> &Mesh {
        &self.node(id).mesh
    }

    #[track_caller]
    pub fn mesh_mut(&mut self, id: NodeId) -> &mut Mesh {
        &mut self.node_mut(id).mesh
    }

    pub fn remove_node(&mut self, id: NodeId) -> Option<Node> {
        self.nodes.remove(&id)
    }
}

impl World {
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
}
