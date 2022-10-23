use std::{any::TypeId, path::Path, sync::Arc};

use lumi_id::Id;
use lumi_util::{crossbeam::channel::Sender, once_cell::sync::OnceCell};

use crate::Asset;

pub type HandleTracker = Sender<(TypeId, HandleId)>;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PathId {
    id: u64,
}

impl From<&Path> for PathId {
    #[inline]
    fn from(path: &Path) -> Self {
        let id = lumi_util::hash(path);
        Self { id }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HandleId {
    Id(Id),
    Path(PathId),
}

impl HandleId {
    #[inline]
    pub fn new() -> Self {
        Self::Id(Id::new())
    }
}

impl From<&Path> for HandleId {
    #[inline]
    fn from(path: &Path) -> Self {
        Self::Path(path.into())
    }
}

#[derive(Debug)]
pub(crate) struct Inner<T: Asset> {
    pub(crate) id: HandleId,
    pub(crate) asset: OnceCell<T>,
    pub(crate) tracker: Option<HandleTracker>,
}

#[repr(transparent)]
#[derive(Debug)]
pub struct Handle<T: Asset> {
    pub(crate) inner: Arc<Inner<T>>,
}

impl<T: Asset> Handle<T> {
    #[inline]
    pub(crate) fn from_inner(inner: Arc<Inner<T>>) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn id(&self) -> &HandleId {
        &self.inner.id
    }

    #[inline]
    pub fn is_unique(&self) -> bool {
        Arc::strong_count(&self.inner) == 2
    }

    #[inline]
    pub fn is_tracked(&self) -> bool {
        self.inner.tracker.is_some()
    }

    #[inline]
    pub fn set(&self, asset: T) -> Result<(), T> {
        self.inner.asset.set(asset)
    }

    #[inline]
    pub fn get(&self) -> Option<&T> {
        self.inner.asset.get()
    }

    #[inline]
    pub fn wait(&self) -> &T {
        self.inner.asset.wait()
    }
}

impl<T: Asset> Into<HandleId> for &Handle<T> {
    #[inline]
    fn into(self) -> HandleId {
        self.inner.id
    }
}

impl<T: Asset> Clone for Handle<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: Asset> PartialEq for Handle<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

impl<T: Asset> Eq for Handle<T> {}

impl<T: Asset> Drop for Handle<T> {
    #[inline]
    fn drop(&mut self) {
        if let Some(ref tracker) = self.inner.tracker {
            if self.is_unique() {
                let _ = tracker.send((TypeId::of::<T>(), self.inner.id));
            }
        }
    }
}
