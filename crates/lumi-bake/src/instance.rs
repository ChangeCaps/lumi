use lumi_core::{
    Backends, Device, DeviceDescriptor, Features, Instance, Limits, PowerPreference, Queue,
    RequestAdapterOptions, RequestDeviceError,
};
use lumi_util::thiserror;

pub struct BakeInstance {
    device: Device,
    queue: Queue,
}

impl BakeInstance {
    pub async fn new_async() -> Result<Self, BakeInstanceError> {
        let instance = Instance::new(Backends::PRIMARY);

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or(BakeInstanceError::NoAdapter)?;

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: Some("Lumi Bake Device"),
                    features: Features::empty(),
                    limits: Limits::default(),
                },
                None,
            )
            .await?;

        Ok(Self { device, queue })
    }

    pub fn new() -> Result<Self, BakeInstanceError> {
        lumi_task::block_on(Self::new_async())
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn queue(&self) -> &Queue {
        &self.queue
    }
}

#[derive(thiserror::Error, Debug)]
pub enum BakeInstanceError {
    #[error("Failed to create adapter")]
    NoAdapter,
    #[error("Failed to create device")]
    CreateDevice(#[from] RequestDeviceError),
}
