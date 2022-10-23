#[cfg(feature = "ahash")]
pub use ahash;
#[cfg(feature = "async-trait")]
pub use async_trait::async_trait;
#[cfg(feature = "bytemuck")]
pub use bytemuck;
#[cfg(feature = "crossbeam")]
pub use crossbeam;
#[cfg(feature = "dashmap")]
pub use dashmap;
#[cfg(feature = "hashbrown")]
pub use hashbrown;
#[cfg(feature = "once_cell")]
pub use once_cell;
#[cfg(feature = "smallvec")]
pub use smallvec;
#[cfg(feature = "thiserror")]
pub use thiserror;
#[cfg(feature = "wgpu-types")]
pub use wgpu_types;

use std::{
    future::Future,
    hash::Hash,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

#[cfg(feature = "ahash")]
pub type RandomState = ahash::RandomState;

#[cfg(feature = "hashbrown")]
pub type HashMap<K, V, S = RandomState> = hashbrown::HashMap<K, V, S>;
#[cfg(feature = "hashbrown")]
pub type HashSet<T, S = RandomState> = hashbrown::HashSet<T, S>;
#[cfg(feature = "dashmap")]
pub type DashMap<K, V, S = RandomState> = dashmap::DashMap<K, V, S>;

pub type BoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[cfg(feature = "math")]
pub mod math {
    pub use glam::{swizzles::*, *};
}

#[inline]
#[cfg(feature = "ahash")]
pub fn hash<T: Hash>(hash: T) -> u64 {
    let state = RandomState::with_seed(42069);
    state.hash_one(hash)
}

#[repr(transparent)]
#[derive(Debug, Default)]
pub struct AtomicMarker {
    marker: AtomicBool,
}

impl Clone for AtomicMarker {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            marker: AtomicBool::new(self.is_marked()),
        }
    }
}

impl AtomicMarker {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn mark(&self) {
        self.marker.store(true, Ordering::Release);
    }

    #[inline]
    pub fn unmark(&self) {
        self.marker.store(false, Ordering::Release);
    }

    #[inline]
    pub fn is_marked(&self) -> bool {
        self.marker.load(Ordering::Acquire)
    }

    #[inline]
    pub fn take(&self) -> bool {
        self.marker.swap(false, Ordering::AcqRel)
    }
}

#[repr(transparent)]
#[derive(Default, Debug, PartialEq, Eq, Hash)]
pub struct SharedState<T> {
    state: Arc<T>,
}

impl<T> SharedState<T> {
    #[inline]
    pub fn new(state: T) -> Self {
        Self {
            state: Arc::new(state),
        }
    }

    #[inline]
    pub fn get(&self) -> &T {
        &self.state
    }

    #[inline]
    pub fn get_mut(&mut self) -> &mut T
    where
        T: Clone,
    {
        Arc::make_mut(&mut self.state)
    }

    #[inline]
    pub fn is_unique(&self) -> bool {
        Arc::strong_count(&self.state) == 1
    }

    #[inline]
    pub fn is_shared(&self) -> bool {
        Arc::strong_count(&self.state) > 1
    }

    #[inline]
    pub fn into_inner(self) -> T
    where
        T: Clone,
    {
        Arc::try_unwrap(self.state).unwrap_or_else(|state| state.as_ref().clone())
    }
}

impl<T> Clone for SharedState<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
        }
    }
}

impl<T> AsRef<T> for SharedState<T> {
    #[inline]
    fn as_ref(&self) -> &T {
        &self.state
    }
}

impl<T> std::ops::Deref for SharedState<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<T> std::ops::DerefMut for SharedState<T>
where
    T: Clone,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
    }
}
