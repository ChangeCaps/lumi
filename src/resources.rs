use std::any::TypeId;

use crate::{
    key_map::{Key, KeyMap},
    util::HashMap,
};

pub trait Resource: Send + Sync + 'static {}
impl<T: Send + Sync + 'static> Resource for T {}

impl dyn Resource {
    #[inline]
    pub unsafe fn downcast_ref<T: Resource>(&self) -> &T {
        unsafe { &*(self as *const dyn Resource as *const T) }
    }

    #[inline]
    pub unsafe fn downcast_mut<T: Resource>(&mut self) -> &mut T {
        unsafe { &mut *(self as *mut dyn Resource as *mut T) }
    }
}

#[derive(Default)]
pub struct Resources {
    typed: HashMap<TypeId, Box<dyn Resource>>,
}

impl Resources {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn contains<T: Resource>(&self) -> bool {
        self.typed.contains_key(&TypeId::of::<T>())
    }

    #[inline]
    pub fn get_keyed<T: Resource>(&self) -> Option<&KeyMap<T>> {
        self.get()
    }

    #[inline]
    pub fn get_keyed_mut<T: Resource>(&mut self) -> &mut KeyMap<T> {
        self.get_mut_or_default()
    }

    #[inline]
    pub fn contains_key<T: Resource>(&self, key: &dyn Key) -> bool {
        self.get_keyed::<T>()
            .map(|keyed| keyed.contains(key))
            .unwrap_or(false)
    }

    #[inline]
    pub fn insert<T: Resource>(&mut self, resource: T) {
        self.typed.insert(TypeId::of::<T>(), Box::new(resource));
    }

    #[inline]
    pub fn insert_key<T: Resource, K: Key>(&mut self, key: K, resource: T) {
        self.get_keyed_mut().insert(key, resource);
    }

    #[inline]
    pub fn remove<T: Resource>(&mut self) -> Option<T> {
        let resource = self.typed.remove(&TypeId::of::<T>())?;

        Some(unsafe { *Box::from_raw(Box::into_raw(resource) as *mut _) })
    }

    #[inline]
    pub fn remove_key<T: Resource>(&mut self, key: &dyn Key) -> Option<T> {
        self.get_keyed_mut().remove(key)
    }

    #[inline]
    pub fn get<T: Resource>(&self) -> Option<&T> {
        let resource = self.typed.get(&TypeId::of::<T>())?;

        Some(unsafe { resource.downcast_ref() })
    }

    #[inline]
    pub fn get_key<T: Resource>(&self, key: &dyn Key) -> Option<&T> {
        self.get_keyed()?.get(key)
    }

    #[inline]
    pub fn get_mut<T: Resource>(&mut self) -> Option<&mut T> {
        let resource = self.typed.get_mut(&TypeId::of::<T>())?;

        Some(unsafe { resource.downcast_mut() })
    }

    #[inline]
    pub fn get_key_mut<T: Resource>(&mut self, key: &dyn Key) -> Option<&mut T> {
        self.get_keyed_mut().get_mut(key)
    }

    #[inline]
    pub fn get_or_default<T: Resource + Default>(&mut self) -> &T {
        self.get_mut_or_default()
    }

    #[inline]
    pub fn get_key_or_default<T: Resource + Default>(&mut self, key: &dyn Key) -> &T {
        self.get_key_mut_or_default(key)
    }

    #[inline]
    pub fn get_mut_or_insert_with<T: Resource>(&mut self, f: impl FnOnce() -> T) -> &mut T {
        if !self.contains::<T>() {
            self.insert(f());
        }

        self.get_mut().unwrap()
    }

    #[inline]
    pub fn get_mut_or_default<T: Resource + Default>(&mut self) -> &mut T {
        self.get_mut_or_insert_with(T::default)
    }

    #[inline]
    pub fn get_key_mut_or_insert_with<T: Resource>(
        &mut self,
        key: &dyn Key,
        f: impl FnOnce() -> T,
    ) -> &mut T {
        if !self.contains_key::<T>(key) {
            self.get_keyed_mut().insert_boxed(key.box_clone(), f());
        }

        self.get_key_mut(key).unwrap()
    }

    #[inline]
    pub fn get_key_mut_or_default<T: Resource + Default>(&mut self, key: &dyn Key) -> &mut T {
        self.get_key_mut_or_insert_with(key, T::default)
    }

    #[inline]
    pub fn remove_keyed<T: Resource>(&mut self) -> KeyMap<T> {
        self.remove().unwrap_or_default()
    }

    #[inline]
    pub fn iter<T: Resource>(&self) -> impl Iterator<Item = &T> {
        self.get_keyed()
            .map(|keyed| keyed.values())
            .into_iter()
            .flatten()
    }

    #[inline]
    pub fn iter_mut<T: Resource>(&mut self) -> impl Iterator<Item = &mut T> {
        self.get_keyed_mut().values_mut()
    }
}
