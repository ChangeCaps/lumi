use std::hash::{BuildHasher, Hasher};

use uuid::Uuid;

use hashbrown::{HashMap, HashSet};

use super::Id;

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
        debug_assert_eq!(bytes.len(), 16);

        let low = unsafe { *(bytes.as_ptr() as *const u64) };
        let high = unsafe { *(bytes.as_ptr().add(8) as *const u64) };
        self.write_u64(low);
        self.write_u64(high);
    }

    #[inline(always)]
    fn write_u64(&mut self, i: u64) {
        self.state ^= i;
    }
}
#[repr(transparent)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdSet {
    set: HashSet<Uuid, IdState>,
}

#[repr(transparent)]
#[derive(Clone, Debug)]
pub struct Intersection<'a> {
    iter: hashbrown::hash_set::Intersection<'a, Uuid, IdState>,
}

impl Iterator for Intersection<'_> {
    type Item = Uuid;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().copied()
    }
}

impl Default for IdSet {
    #[inline(always)]
    fn default() -> Self {
        Self {
            set: HashSet::default(),
        }
    }
}

impl IdSet {
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
    pub fn insert(&mut self, id: impl Id) -> bool {
        self.set.insert(id.uuid())
    }

    #[inline(always)]
    pub fn remove(&mut self, id: &impl Id) -> bool {
        self.set.remove(id.uuid_ref())
    }

    #[inline(always)]
    pub fn contains(&self, id: &Uuid) -> bool {
        self.set.contains(id)
    }

    #[inline(always)]
    pub fn iter(&self) -> impl Iterator<Item = &Uuid> {
        self.set.iter()
    }

    pub fn intersection<'a>(&'a self, other: &'a Self) -> Intersection<'a> {
        Intersection {
            iter: self.set.intersection(&other.set),
        }
    }
}

#[repr(transparent)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdMap<T> {
    map: HashMap<Uuid, T, IdState>,
}

impl<T> Default for IdMap<T> {
    fn default() -> Self {
        Self {
            map: HashMap::default(),
        }
    }
}

impl<T> IdMap<T> {
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

    #[inline(always)]
    pub fn insert(&mut self, id: impl Id, value: T) -> bool {
        self.map.insert(id.uuid(), value).is_none()
    }

    #[inline(always)]
    pub fn get(&self, id: &impl Id) -> Option<&T> {
        self.map.get(id.uuid_ref())
    }

    #[inline(always)]
    pub fn get_mut(&mut self, id: &impl Id) -> Option<&mut T> {
        self.map.get_mut(id.uuid_ref())
    }

    #[inline(always)]
    pub fn remove(&mut self, id: &impl Id) -> Option<T> {
        self.map.remove(id.uuid_ref())
    }

    #[inline(always)]
    pub fn keys(&self) -> impl Iterator<Item = &Uuid> {
        self.map.keys()
    }

    #[inline(always)]
    pub fn values(&self) -> impl Iterator<Item = &T> {
        self.map.values()
    }

    #[inline(always)]
    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.map.values_mut()
    }

    #[inline(always)]
    pub fn iter(&self) -> impl Iterator<Item = (&Uuid, &T)> {
        self.map.iter()
    }

    #[inline(always)]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&Uuid, &mut T)> {
        self.map.iter_mut()
    }
}
