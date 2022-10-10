macro_rules! ids {
    ($(pub struct $ident:ident;)*) => {
        $(
            #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
            pub struct $ident {
                uuid: ::uuid::Uuid,
            }

            impl Default for $ident {
                fn default() -> Self {
                    Self::new()
                }
            }

            impl $ident {
                pub fn new() -> Self {
                    Self {
                        uuid: ::uuid::Uuid::new_v4(),
                    }
                }

                pub const fn uuid(&self) -> ::uuid::Uuid {
                    self.uuid
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
