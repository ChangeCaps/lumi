use std::{any::TypeId, collections::HashMap};

use uuid::Uuid;

use crate::{Bind, Bindings, Material, SharedDevice};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Entity {
    uuid: Uuid,
}

impl Entity {
    pub fn new() -> Self {
        Self {
            uuid: Uuid::new_v4(),
        }
    }

    pub fn uuid(&self) -> Uuid {
        self.uuid
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform {
    matrix: [[f32; 4]; 4],
}

impl Transform {
    pub const IDENTITY: Self = Self {
        matrix: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    };

    pub const fn new(matrix: [[f32; 4]; 4]) -> Self {
        Self { matrix }
    }

    pub const fn matrix(&self) -> [[f32; 4]; 4] {
        self.matrix
    }
}

pub struct Node {
    pub bindings: Bindings,
    pub material: Box<dyn Bind>,
    pub transform: Transform,
}

impl Node {
    pub fn new<T>(device: &SharedDevice, material: T) -> Self
    where
        T: Material + 'static,
    {
        let bindings = Bindings::build().bind::<T>().build(device);

        Self {
            bindings,
            material: Box::new(material),
            transform: Transform::IDENTITY,
        }
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
    entities: HashMap<Entity, Node>,
    device: SharedDevice,
}

impl World {
    pub fn new(device: &SharedDevice) -> Self {
        Self {
            resources: Resources::new(),
            entities: HashMap::new(),
            device: device.clone(),
        }
    }

    pub fn resources(&self) -> &Resources {
        &self.resources
    }

    pub fn resources_mut(&mut self) -> &mut Resources {
        &mut self.resources
    }
}
