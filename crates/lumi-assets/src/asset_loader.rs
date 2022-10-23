use std::any::TypeId;

use lumi_util::async_trait;

use crate::{Asset, AssetServer, Handle, HandleId};

pub unsafe trait HandleUntyped: Send + Sync {
    fn type_id(&self) -> TypeId;
    fn id(&self) -> &HandleId;
}

unsafe impl<T: Asset> HandleUntyped for Handle<T> {
    #[inline]
    fn type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }

    #[inline]
    fn id(&self) -> &HandleId {
        &self.inner.id
    }
}

impl dyn HandleUntyped {
    #[inline]
    pub fn is<T: Asset>(&self) -> bool {
        self.type_id() == TypeId::of::<T>()
    }

    #[inline]
    pub fn downcast_ref<T: Asset>(&self) -> Option<&Handle<T>> {
        if self.is::<T>() {
            Some(unsafe { &*(self as *const dyn HandleUntyped as *const Handle<T>) })
        } else {
            None
        }
    }

    #[inline]
    pub fn set<T: Asset>(&self, asset: T) -> Result<(), T> {
        if let Some(handle) = self.downcast_ref::<T>() {
            handle.set(asset)
        } else {
            Err(asset)
        }
    }
}

pub struct LoadContext<'a> {
    pub bytes: &'a [u8],
    pub handle: &'a dyn HandleUntyped,
    pub extension: &'a str,
    pub asset_server: &'a AssetServer,
}

#[async_trait]
pub trait AssetLoader: Send + Sync + 'static {
    async fn load(&self, ctx: &LoadContext<'_>) -> Result<(), ()>;

    fn extensions(&self) -> &[&str];
}
