mod map;

use std::{hash::Hash, marker::PhantomData};

use lumi_util::RandomState;
pub use map::{IdMap, IdSet};
use uuid::Uuid;

#[repr(transparent)]
pub struct Id<T: ?Sized = ()> {
    uuid: Uuid,
    _marker: PhantomData<fn() -> T>,
}

impl<T: ?Sized> Id<T> {
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            uuid: Uuid::new_v4(),
            _marker: PhantomData,
        }
    }

    #[inline(always)]
    pub const fn from_uuid(uuid: Uuid) -> Self {
        Self {
            uuid,
            _marker: PhantomData,
        }
    }

    #[inline(always)]
    pub fn from_hash<H: Hash>(hash: H) -> Self {
        let low = RandomState::with_seed(420);
        let high = RandomState::with_seed(69);

        let low = low.hash_one(&hash);
        let high = high.hash_one(&hash);

        Self::from_uuid(Uuid::from_u64_pair(high, low))
    }

    #[inline(always)]
    pub const fn uuid(&self) -> Uuid {
        self.uuid
    }

    #[inline(always)]
    pub const fn uuid_ref(&self) -> &Uuid {
        &self.uuid
    }

    #[inline(always)]
    pub const fn cast<U: ?Sized>(self) -> Id<U> {
        Id {
            uuid: self.uuid,
            _marker: PhantomData,
        }
    }

    #[inline(always)]
    pub const fn cast_ref<U: ?Sized>(&self) -> &Id<U> {
        unsafe { &*(self as *const Id<T> as *const Id<U>) }
    }

    #[inline(always)]
    pub fn cast_mut<U: ?Sized>(&mut self) -> &mut Id<U> {
        unsafe { &mut *(self as *mut Id<T> as *mut Id<U>) }
    }
}

impl<T: ?Sized> Default for Id<T> {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ?Sized> Clone for Id<T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self {
            uuid: self.uuid,
            _marker: PhantomData,
        }
    }
}

impl<T: ?Sized> Copy for Id<T> {}

impl<T: ?Sized> std::fmt::Debug for Id<T> {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple(&format!("Id<{}>", std::any::type_name::<T>()))
            .field(&self.uuid)
            .finish()
    }
}

impl<T: ?Sized> std::fmt::Display for Id<T> {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.uuid.fmt(f)
    }
}

impl<T: ?Sized> PartialEq for Id<T> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

impl<T: ?Sized> Eq for Id<T> {}

impl<T: ?Sized> PartialOrd for Id<T> {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.uuid.partial_cmp(&other.uuid)
    }
}

impl<T: ?Sized> Ord for Id<T> {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.uuid.cmp(&other.uuid)
    }
}

impl<T: ?Sized> Hash for Id<T> {
    #[inline(always)]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uuid.hash(state);
    }
}

impl<T: ?Sized, U: ?Sized> AsRef<Id<U>> for Id<T> {
    #[inline(always)]
    fn as_ref(&self) -> &Id<U> {
        self.cast_ref()
    }
}
