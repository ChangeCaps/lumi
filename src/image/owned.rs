use std::num::NonZeroU32;

use wgpu::{
    Extent3d, FilterMode, SamplerDescriptor, TextureDimension, TextureFormat, TextureUsages,
};

use crate::{
    bind::{DefaultSampler, DefaultTexture, SamplerBinding, SharedBindingResource, TextureBinding},
    SharedDevice, SharedQueue, SharedSampler, SharedTexture, SharedTextureView,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ImageData {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
    pub format: TextureFormat,
    pub filter: bool,
}

impl Default for ImageData {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            data: Vec::new(),
            format: TextureFormat::Rgba8UnormSrgb,
            filter: true,
        }
    }
}

impl ImageData {
    pub fn new(width: u32, height: u32, data: Vec<u8>) -> Self {
        Self::with_format(width, height, data, TextureFormat::Rgba8UnormSrgb)
    }

    pub fn with_format(width: u32, height: u32, data: Vec<u8>, format: TextureFormat) -> Self {
        Self {
            width,
            height,
            data,
            format,
            filter: true,
        }
    }

    pub fn load_from_file(path: &str) -> Result<Self, image::ImageError> {
        let image = image::open(path)?;
        let width = image.width();
        let height = image.height();
        let data = image.into_rgba8().into_raw();
        Ok(Self::new(width, height, data))
    }

    pub fn size(&self) -> Extent3d {
        Extent3d {
            width: self.width,
            height: self.height,
            depth_or_array_layers: 1,
        }
    }

    pub fn write_texture(&self, queue: &SharedQueue, texture: &SharedTexture) {
        if self.data.is_empty() {
            return;
        }

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: texture.texture(),
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &self.data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(4 * self.width),
                rows_per_image: None,
            },
            self.size(),
        );
    }

    pub fn create_texture(&self, device: &SharedDevice, queue: &SharedQueue) -> SharedTexture {
        let desc = wgpu::TextureDescriptor {
            label: None,
            size: self.size(),
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: self.format,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::RENDER_ATTACHMENT
                | TextureUsages::COPY_DST,
        };

        if self.data.is_empty() {
            device.create_shared_texture(&desc)
        } else {
            device.create_shared_texture_with_data(queue, &desc, &self.data)
        }
    }

    pub fn create_view(&self, device: &SharedDevice, queue: &SharedQueue) -> SharedTextureView {
        self.create_texture(device, queue)
            .create_view(&Default::default())
    }

    pub fn create_sampler(&self, device: &SharedDevice) -> SharedSampler {
        let filter_mode = if self.filter {
            FilterMode::Linear
        } else {
            FilterMode::Nearest
        };

        device.create_shared_sampler(&SamplerDescriptor {
            mag_filter: filter_mode,
            min_filter: filter_mode,
            ..Default::default()
        })
    }
}

impl TextureBinding for ImageData {
    type State = Option<SharedTextureView>;

    fn binding(
        &self,
        device: &SharedDevice,
        queue: &SharedQueue,
        state: &mut Self::State,
    ) -> SharedBindingResource {
        if let Some(texture) = state {
            if texture.size() != self.size() {
                let view = self
                    .create_texture(device, queue)
                    .create_view(&Default::default());
                *texture = view;
            }

            SharedBindingResource::TextureView(texture.clone())
        } else {
            let texture = self.create_texture(device, queue);
            let view = texture.create_view(&Default::default());

            *state = Some(view.clone());

            SharedBindingResource::TextureView(view)
        }
    }
}

impl SamplerBinding for ImageData {
    type State = Option<SharedSampler>;

    fn binding(
        &self,
        device: &SharedDevice,
        _queue: &SharedQueue,
        state: &mut Self::State,
    ) -> SharedBindingResource {
        if let Some(sampler) = state {
            if sampler.mag_filter() == FilterMode::Nearest && self.filter {
                *sampler = self.create_sampler(device);
            }

            SharedBindingResource::Sampler(sampler.clone())
        } else {
            let sampler = self.create_sampler(device);
            *state = Some(sampler.clone());

            SharedBindingResource::Sampler(sampler)
        }
    }
}

impl DefaultTexture for ImageData {
    fn default_texture(device: &SharedDevice, queue: &SharedQueue) -> SharedTextureView {
        Self::new(1, 1, vec![255; 4]).create_view(device, queue)
    }
}

impl DefaultSampler for ImageData {
    fn default_sampler(device: &SharedDevice, _queue: &SharedQueue) -> SharedSampler {
        device.create_shared_sampler(&Default::default())
    }
}