use wgpu::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages};

use crate::{SharedDevice, SharedTexture};

#[derive(Clone, Debug)]
pub struct FrameBuffer {
    pub hdr: SharedTexture,
    pub depth: SharedTexture,
}

impl FrameBuffer {
    pub fn new(device: &SharedDevice, width: u32, height: u32) -> Self {
        let hdr = device.create_shared_texture(&TextureDescriptor {
            label: Some("Lumi HDR Target"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba16Float,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
        });

        let depth = device.create_shared_texture(&TextureDescriptor {
            label: Some("Lumi Depth Target"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: TextureUsages::RENDER_ATTACHMENT,
        });

        Self { hdr, depth }
    }

    pub fn width(&self) -> u32 {
        self.hdr.size().width
    }

    pub fn height(&self) -> u32 {
        self.hdr.size().height
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.width() as f32 / self.height() as f32
    }

    pub fn resize(&mut self, device: &SharedDevice, width: u32, height: u32) {
        if self.width() != width || self.height() != height {
            *self = Self::new(device, width, height);
        }
    }
}
