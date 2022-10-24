use std::path::Path;

use lumi_bake::{BakedEnvironment, EnvironmentData};
use lumi_core::{Device, ImageData, ImageError, Queue};
use lumi_id::Id;

pub enum EnvironmentSource {
    Baked(EnvironmentData),
    RealTime(ImageData),
}

pub type EnvironmentId = Id<Environment>;

pub struct Environment {
    kind: EnvironmentSource,
    id: EnvironmentId,
}

impl Default for Environment {
    fn default() -> Self {
        let data = EnvironmentData::from_bytes(&include_bytes!("default_env.bake")[..]).unwrap();

        Self {
            kind: EnvironmentSource::Baked(data),
            id: EnvironmentId::new(),
        }
    }
}

impl Environment {
    #[inline]
    pub fn new(image: ImageData) -> Self {
        Self {
            kind: EnvironmentSource::RealTime(image),
            id: EnvironmentId::new(),
        }
    }

    #[inline]
    pub fn open(path: impl AsRef<Path>) -> Result<Self, ImageError> {
        let image = ImageData::open_hdr(path)?;

        Ok(Self::new(image))
    }

    #[inline]
    pub fn bake(&self, device: &Device, queue: &Queue) -> BakedEnvironment {
        match &self.kind {
            EnvironmentSource::Baked(data) => BakedEnvironment::from_data(device, queue, data),
            EnvironmentSource::RealTime(image) => BakedEnvironment::from_eq_bytes(
                device,
                queue,
                &image.data,
                image.width,
                image.height,
                256,
                128,
                2048,
            ),
        }
    }

    #[inline]
    pub fn id(&self) -> EnvironmentId {
        self.id
    }
}
