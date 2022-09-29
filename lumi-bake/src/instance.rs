use futures_lite::future;
use wgpu::{Device, Queue};

pub struct BakeInstance {
    device: Device,
    queue: Queue,
}

impl BakeInstance {
    pub async fn new_async() -> Result<Self, BakeInstanceError> {
        let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or(BakeInstanceError::NoAdapter)?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Lumi Bake Device"),
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await?;

        Ok(Self { device, queue })
    }

    pub fn new() -> Result<Self, BakeInstanceError> {
        future::block_on(Self::new_async())
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
    CreateDevice(#[from] wgpu::RequestDeviceError),
}
