use std::hash::{BuildHasher, Hasher};

use lumi_util::hashbrown::{self, HashMap, HashSet};

use crate::Id;

#[derive(Clone, Debug, Default)]
struct IdState;

impl BuildHasher for IdState {
    type Hasher = IdHasher;

    #[inline(always)]
    fn build_hasher(&self) -> Self::Hasher {
        IdHasher::default()
    }
}

#[repr(transparent)]
#[derive(Default)]
struct IdHasher {
    state: u64,
}

impl Hasher for IdHasher {
    #[inline(always)]
    fn finish(&self) -> u64 {
        self.state
    }

    #[inline(always)]
    fn write(&mut self, bytes: &[u8]) {
        match bytes.len() {
            8 => {
                let low = unsafe { *(bytes.as_ptr() as *const u64) };
                self.write_u64(low);
            }
            16 => {
                let low = unsafe { *(bytes.as_ptr() as *const u64) };
                let high = unsafe { *(bytes.as_ptr().add(8) as *const u64) };
                self.write_u64(low);
                self.write_u64(high);
            }
            _ => unreachable!(),
        }
    }

    #[inline(always)]
    fn write_u64(&mut self, i: u64) {
        self.state ^= i;
    }
}

#[repr(transparent)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdSet<T: ?Sized = ()> {
    set: HashSet<Id<T>, IdState>,
}

#[repr(transparent)]
#[derive(Clone, Debug)]
pub struct Intersection<'a, T: ?Sized> {
    iter: hashbrown::hash_set::Intersection<'a, Id<T>, IdState>,
}

impl<T: ?Sized> Iterator for Intersection<'_, T> {
    type Item = Id<T>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().copied()
    }
}

impl<T: ?Sized> Default for IdSet<T> {
    #[inline(always)]
    fn default() -> Self {
        Self {
            set: HashSet::default(),
        }
    }
}

impl<T: ?Sized> IdSet<T> {
    #[inline(always)]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.set.len()
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.set.is_empty()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.set.clear();
    }

    #[inline(always)]
    pub fn insert(&mut self, id: Id<T>) -> bool {
        self.set.insert(id)
    }

    #[inline(always)]
    pub fn remove(&mut self, id: Id<T>) -> bool {
        self.set.remove(id.as_ref())
    }

    #[inline(always)]
    pub fn contains(&self, id: Id<T>) -> bool {
        self.set.contains(id.as_ref())
    }

    #[inline(always)]
    pub fn iter(&self) -> impl Iterator<Item = &Id<T>> {
        self.set.iter()
    }

    #[inline(always)]
    pub fn intersection<'a>(&'a self, other: &'a Self) -> Intersection<'a, T> {
        Intersection {
            iter: self.set.intersection(&other.set),
        }
    }
}

#[repr(transparent)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdMap<K: ?Sized, V = K> {
    map: HashMap<Id<K>, V, IdState>,
}

impl<K: ?Sized, V> Default for IdMap<K, V> {
    #[inline]
    fn default() -> Self {
        Self {
            map: HashMap::default(),
        }
    }
}

impl<K: ?Sized, V> IdMap<K, V> {
    #[inline(always)]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.map.clear();
    }

    #[inline]
    pub fn contains_id(&self, id: Id<K>) -> bool {
        self.map.contains_key(id.as_ref())
    }

    #[inline(always)]
    pub fn insert(&mut self, id: Id<K>, value: V) -> bool {
        self.map.insert(id, value).is_none()
    }

    #[inline(always)]
    pub fn get(&self, id: Id<K>) -> Option<&V> {
        self.map.get(id.as_ref())
    }

    #[inline(always)]
    pub fn get_mut(&mut self, id: Id<K>) -> Option<&mut V> {
        self.map.get_mut(id.as_ref())
    }

    #[inline(always)]
    pub fn get_or_insert_with(&mut self, id: Id<K>, f: impl FnOnce() -> V) -> &mut V {
        self.map.entry(id).or_insert_with(f)
    }

    #[inline(always)]
    pub fn get_or_default(&mut self, id: Id<K>) -> &mut V
    where
        V: Default,
    {
        self.map.entry(id).or_default()
    }

    #[inline(always)]
    pub fn remove(&mut self, id: Id<K>) -> Option<V> {
        self.map.remove(id.as_ref())
    }

    #[inline(always)]
    pub fn retain(&mut self, f: impl FnMut(&Id<K>, &mut V) -> bool) {
        self.map.retain(f)
    }

    #[inline(always)]
    pub fn keys(&self) -> impl Iterator<Item = &Id<K>> {
        self.map.keys()
    }

    #[inline(always)]
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.map.values()
    }

    #[inline(always)]
    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.map.values_mut()
    }

    #[inline(always)]
    pub fn iter(&self) -> impl Iterator<Item = (&Id<K>, &V)> {
        self.map.iter()
    }

    #[inline(always)]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&Id<K>, &mut V)> {
        self.map.iter_mut()
    }
}

impl<K: ?Sized, V> FromIterator<(Id<K>, V)> for IdMap<K, V> {
    #[inline]
    fn from_iter<T: IntoIterator<Item = (Id<K>, V)>>(iter: T) -> Self {
        Self {
            map: iter.into_iter().collect(),
        }
    }
}
