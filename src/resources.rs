use std::{any::TypeId, collections::HashMap};

use crate::key_map::{Key, KeyMap};

pub trait Resource: Send + Sync + 'static {}
impl<T: Send + Sync + 'static> Resource for T {}

impl dyn Resource {
    pub unsafe fn downcast_ref<T: Resource>(&self) -> &T {
        unsafe { &*(self as *const dyn Resource as *const T) }
    }

    pub unsafe fn downcast_mut<T: Resource>(&mut self) -> &mut T {
        unsafe { &mut *(self as *mut dyn Resource as *mut T) }
    }
}

#[derive(Default)]
pub struct Resources {
    typed: HashMap<TypeId, Box<dyn Resource>>,
    keyed: HashMap<TypeId, KeyMap<Box<dyn Resource>>>,
}

impl Resources {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn contains<T: Resource>(&self) -> bool {
        self.typed.contains_key(&TypeId::of::<T>())
    }

    pub fn contains_key<T: Resource>(&self, key: &dyn Key) -> bool {
        self.keyed
            .get(&TypeId::of::<T>())
            .map(|keyed| keyed.contains(key))
            .unwrap_or(false)
    }

    pub fn insert<T: Resource>(&mut self, resource: T) {
        self.typed.insert(TypeId::of::<T>(), Box::new(resource));
    }

    pub fn insert_key<T: Resource, K: Key>(&mut self, key: K, resource: T) {
        self.keyed
            .entry(TypeId::of::<T>())
            .or_insert_with(KeyMap::new)
            .insert(key, Box::new(resource));
    }

    pub fn remove<T: Resource>(&mut self) -> Option<T> {
        let resource = self.typed.remove(&TypeId::of::<T>())?;

        Some(unsafe { *Box::from_raw(Box::into_raw(resource) as *mut _) })
    }

    pub fn remove_key<T: Resource>(&mut self, key: &dyn Key) -> Option<T> {
        let resource = self.keyed.get_mut(&TypeId::of::<T>())?.remove(key)?;

        Some(unsafe { *Box::from_raw(Box::into_raw(resource) as *mut _) })
    }

    pub fn get<T: Resource>(&self) -> Option<&T> {
        let resource = self.typed.get(&TypeId::of::<T>())?;

        Some(unsafe { resource.downcast_ref() })
    }

    pub fn get_key<T: Resource>(&self, key: &dyn Key) -> Option<&T> {
        let resource = self.keyed.get(&TypeId::of::<T>())?.get(key)?;

        Some(unsafe { resource.downcast_ref() })
    }

    pub fn get_mut<T: Resource>(&mut self) -> Option<&mut T> {
        let resource = self.typed.get_mut(&TypeId::of::<T>())?;

        Some(unsafe { resource.downcast_mut() })
    }

    pub fn get_key_mut<T: Resource>(&mut self, key: &dyn Key) -> Option<&mut T> {
        let resource = self.keyed.get_mut(&TypeId::of::<T>())?.get_mut(key)?;

        Some(unsafe { resource.downcast_mut() })
    }

    pub fn get_or_default<T: Resource + Default>(&mut self) -> &T {
        self.get_mut_or_default()
    }

    pub fn get_key_or_default<T: Resource + Default>(&mut self, key: &dyn Key) -> &T {
        self.get_key_mut_or_default(key)
    }

    pub fn get_mut_or_default<T: Resource + Default>(&mut self) -> &mut T {
        if !self.contains::<T>() {
            self.insert(T::default());
        }

        self.get_mut().unwrap()
    }

    pub fn get_key_mut_or_default<T: Resource + Default>(&mut self, key: &dyn Key) -> &mut T {
        if !self.contains_key::<T>(key) {
            self.keyed
                .entry(TypeId::of::<T>())
                .or_insert_with(KeyMap::new)
                .insert_boxed(key.box_clone(), Box::new(T::default()));
        }

        self.get_key_mut(key).unwrap()
    }
}
