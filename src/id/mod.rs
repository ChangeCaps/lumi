mod map;

pub use map::{IdMap, IdSet};
use uuid::Uuid;

/// # Safety
/// * `T` must be `Send + Sync`
/// * `T` must be able to be safely transmuted into [`Uuid`]
pub unsafe trait Id {
    fn uuid(&self) -> Uuid;
    fn uuid_ref(&self) -> &Uuid;
}

unsafe impl Id for Uuid {
    #[inline(always)]
    fn uuid(&self) -> Uuid {
        *self
    }

    #[inline(always)]
    fn uuid_ref(&self) -> &Uuid {
        self
    }
}

macro_rules! ids {
    ($(pub struct $ident:ident;)*) => {
        $(
            #[repr(transparent)]
            #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
            pub struct $ident {
                uuid: ::uuid::Uuid,
            }

            impl Default for $ident {
                #[inline(always)]
                fn default() -> Self {
                    Self::new()
                }
            }

            impl $ident {
                #[inline(always)]
                pub fn new() -> Self {
                    Self {
                        uuid: ::uuid::Uuid::new_v4(),
                    }
                }

                #[inline(always)]
                pub const fn from_uuid(uuid: ::uuid::Uuid) -> Self {
                    Self { uuid }
                }

                #[inline(always)]
                pub const fn uuid(&self) -> ::uuid::Uuid {
                    self.uuid
                }
            }

            unsafe impl $crate::id::Id for $ident {
                #[inline(always)]
                fn uuid(&self) -> ::uuid::Uuid {
                    self.uuid
                }

                #[inline(always)]
                fn uuid_ref(&self) -> &::uuid::Uuid {
                    &self.uuid
                }
            }
        )*
    };
}

ids! {
    pub struct DeviceId;
    pub struct QueueId;
    pub struct BufferId;
    pub struct TextureId;
    pub struct TextureViewId;
    pub struct SamplerId;
    pub struct BindGroupId;
    pub struct RenderPipelineId;
    pub struct ComputePipelineId;
    pub struct ShaderId;
    pub struct EnvironmentId;
    pub struct MeshId;
    pub struct WorldId;
    pub struct NodeId;
    pub struct LightId;
    pub struct CameraId;
}
