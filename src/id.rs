macro_rules! ids {
    ($(pub struct $ident:ident;)*) => {
        $(
            #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
            pub struct $ident {
                uuid: ::uuid::Uuid,
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
    pub struct ShaderId;
    pub struct PipelineLayoutId;
    pub struct RenderPipelineId;
    pub struct ComputePipelineId;
}
