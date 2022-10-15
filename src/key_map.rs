use hashbrown::hash_map::Entry;
use std::{
    any::Any,
    fmt::Debug,
    hash::{Hash, Hasher},
};

use crate::util::{HashMap, RandomState};

pub trait Key: Send + Sync + Any + Debug {
    fn hash(&self, hasher: &mut dyn Hasher);
    fn eq(&self, other: &dyn Key) -> bool;
    fn box_clone(&self) -> Box<dyn Key>;
}

impl<T> Key for T
where
    T: Clone + Debug + Send + Sync + Any + Eq + Hash,
{
    #[inline]
    fn hash(&self, mut hasher: &mut dyn Hasher) {
        self.hash(&mut hasher);
    }

    #[inline]
    fn eq(&self, other: &dyn Key) -> bool {
        if self.type_id() != other.type_id() {
            return false;
        }

        // SAFETY: We just checked that the types are the same.
        let other = unsafe { &*(other as *const dyn Key as *const T) };

        self.eq(other)
    }

    #[inline]
    fn box_clone(&self) -> Box<dyn Key> {
        Box::new(self.clone())
    }
}

impl PartialEq for dyn Key {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.eq(other)
    }
}

impl Eq for dyn Key {}

impl Hash for dyn Key {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash(state);
    }
}

impl Clone for Box<dyn Key> {
    #[inline]
    fn clone(&self) -> Self {
        self.box_clone()
    }
}

#[derive(Clone, Debug)]
pub struct KeyMap<T> {
    map: HashMap<Box<dyn Key>, T>,
}

impl<T> Default for KeyMap<T> {
    #[inline]
    fn default() -> Self {
        Self {
            map: HashMap::default(),
        }
    }
}

impl<T> KeyMap<T> {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn contains(&self, key: &dyn Key) -> bool {
        self.map.contains_key(key)
    }

    #[inline]
    pub fn insert<K: Key>(&mut self, key: K, value: T) -> Option<T> {
        self.map.insert(Box::new(key), value)
    }

    #[inline]
    pub fn insert_boxed(&mut self, key: Box<dyn Key>, value: T) -> Option<T> {
        self.map.insert(key, value)
    }

    #[inline]
    pub fn remove(&mut self, key: &dyn Key) -> Option<T> {
        self.map.remove(key)
    }

    #[inline]
    pub fn entry<K: Key>(&mut self, key: K) -> Entry<'_, Box<dyn Key>, T, RandomState> {
        self.map.entry(Box::new(key))
    }

    #[inline(never)]
    pub fn get(&self, key: &dyn Key) -> Option<&T> {
        self.map.get(key)
    }

    #[inline]
    pub fn get_mut(&mut self, key: &dyn Key) -> Option<&mut T> {
        self.map.get_mut(key)
    }

    #[inline]
    pub fn keys(&self) -> impl Iterator<Item = &dyn Key> {
        self.map.keys().map(|key| key.as_ref())
    }

    #[inline]
    pub fn values(&self) -> impl Iterator<Item = &T> {
        self.map.values()
    }

    #[inline]
    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.map.values_mut()
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (&dyn Key, &T)> {
        self.map.iter().map(|(key, value)| (key.as_ref(), value))
    }

    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&dyn Key, &mut T)> {
        self.map
            .iter_mut()
            .map(|(key, value)| (key.as_ref(), value))
    }
}
