use std::{num::NonZeroU32, path::Path};

use wgpu::{Extent3d, SamplerDescriptor, TextureDimension, TextureFormat, TextureUsages};

use crate::{
    bind::{DefaultSampler, DefaultTexture, SamplerBinding, SharedBindingResource, TextureBinding},
    bind_key::BindKey,
    Device, Queue, SharedDevice, SharedSampler, SharedTexture, SharedTextureView,
};

#[derive(Clone, Debug, PartialEq)]
pub struct ImageData {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
    pub format: TextureFormat,
    pub sampler: SamplerDescriptor<'static>,
}

impl Default for ImageData {
    #[inline]
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            data: Vec::new(),
            format: TextureFormat::Rgba8UnormSrgb,
            sampler: SamplerDescriptor::default(),
        }
    }
}

impl ImageData {
    #[inline]
    pub fn new(width: u32, height: u32, data: Vec<u8>) -> Self {
        Self::with_format(width, height, data, TextureFormat::Rgba8UnormSrgb)
    }

    #[inline]
    pub fn with_format(width: u32, height: u32, data: Vec<u8>, format: TextureFormat) -> Self {
        Self {
            width,
            height,
            data,
            format,
            ..Default::default()
        }
    }

    #[inline]
    pub fn open_srgb(path: impl AsRef<Path>) -> Result<Self, image::ImageError> {
        let image = image::open(path)?;
        let width = image.width();
        let height = image.height();
        let data = image.into_rgba8().into_raw();
        Ok(Self::new(width, height, data))
    }

    #[inline]
    pub fn open_hdr(path: impl AsRef<Path>) -> Result<Self, image::ImageError> {
        let image = image::open(path)?;
        let width = image.width();
        let height = image.height();
        let data = image
            .into_rgba16()
            .into_raw()
            .into_iter()
            .map(bytemuck::cast::<_, [u8; 2]>)
            .flatten()
            .collect();

        Ok(Self::with_format(
            width,
            height,
            data,
            TextureFormat::Rgba16Uint,
        ))
    }

    #[inline]
    pub fn size(&self) -> Extent3d {
        Extent3d {
            width: self.width,
            height: self.height,
            depth_or_array_layers: 1,
        }
    }

    #[inline]
    pub fn write_texture(&self, queue: &Queue, texture: &SharedTexture) {
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

    #[inline]
    pub fn create_texture(&self, device: &Device, queue: &Queue) -> SharedTexture {
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

    #[inline]
    pub fn create_view(&self, device: &Device, queue: &Queue) -> SharedTextureView {
        self.create_texture(device, queue)
            .create_view(&Default::default())
    }

    #[inline]
    pub fn create_sampler(&self, device: &Device) -> SharedSampler {
        device.create_shared_sampler(&self.sampler)
    }
}

impl TextureBinding for ImageData {
    type State = Option<SharedTextureView>;

    #[inline]
    fn bind_key(&self) -> BindKey {
        BindKey::from_hash(&self.data)
    }

    #[inline]
    fn binding(
        &self,
        device: &Device,
        queue: &Queue,
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

    #[inline]
    fn bind_key(&self) -> BindKey {
        BindKey::from_hash(&self.data)
    }

    #[inline]
    fn binding(
        &self,
        device: &Device,
        _queue: &Queue,
        state: &mut Self::State,
    ) -> SharedBindingResource {
        if let Some(sampler) = state {
            if sampler.descriptor() != self.sampler {
                *sampler = device.create_shared_sampler(&self.sampler);
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
    fn default_texture(device: &Device, queue: &Queue) -> SharedTextureView {
        Self::new(1, 1, vec![255; 4]).create_view(device, queue)
    }
}

impl DefaultSampler for ImageData {
    fn default_sampler(device: &Device, _queue: &Queue) -> SharedSampler {
        device.create_shared_sampler(&Default::default())
    }
}
