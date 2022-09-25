use std::{
    any::Any,
    collections::{hash_map::Entry, HashMap},
    fmt::Debug,
    hash::{Hash, Hasher},
};

pub trait Key: Any + Debug {
    fn hash(&self, hasher: &mut dyn Hasher);
    fn eq(&self, other: &dyn Key) -> bool;
    fn box_clone(&self) -> Box<dyn Key>;
}

impl<T> Key for T
where
    T: Clone + Debug + Any + Eq + Hash,
{
    fn hash(&self, mut hasher: &mut dyn Hasher) {
        self.hash(&mut hasher);
    }

    fn eq(&self, other: &dyn Key) -> bool {
        if self.type_id() != other.type_id() {
            return false;
        }

        // SAFETY: We just checked that the types are the same.
        let other = unsafe { &*(other as *const dyn Key as *const T) };

        self.eq(other)
    }

    fn box_clone(&self) -> Box<dyn Key> {
        Box::new(self.clone())
    }
}

impl PartialEq for dyn Key {
    fn eq(&self, other: &Self) -> bool {
        self.eq(other)
    }
}

impl Eq for dyn Key {}

impl Hash for dyn Key {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash(state);
    }
}

impl Clone for Box<dyn Key> {
    fn clone(&self) -> Self {
        self.box_clone()
    }
}

#[derive(Clone, Debug)]
pub struct KeyMap<T> {
    map: HashMap<Box<dyn Key>, T>,
}

impl<T> Default for KeyMap<T> {
    fn default() -> Self {
        Self {
            map: HashMap::default(),
        }
    }
}

impl<T> KeyMap<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert<K: Key>(&mut self, key: K, value: T) -> Option<T> {
        self.map.insert(Box::new(key), value)
    }

    pub fn remove(&mut self, key: &dyn Key) -> Option<T> {
        self.map.remove(key)
    }

    pub fn entry<K: Key>(&mut self, key: K) -> Entry<'_, Box<dyn Key>, T> {
        self.map.entry(Box::new(key))
    }

    pub fn get(&self, key: &dyn Key) -> Option<&T> {
        self.map.get(key)
    }

    pub fn get_mut(&mut self, key: &dyn Key) -> Option<&mut T> {
        self.map.get_mut(key)
    }
}
