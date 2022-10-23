#[allow(unused_imports)]
use std::path::Path;
use std::{borrow::Cow, num::NonZeroU32};

use wgpu::{
    Extent3d, ImageCopyTexture, ImageDataLayout, Origin3d, TextureAspect, TextureDescriptor,
    TextureDimension, TextureFormat, TextureUsages,
};

use crate::{Device, Queue, SharedDevice, SharedTexture, SharedTextureView};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ImageData {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
    pub format: TextureFormat,
}

impl Default for ImageData {
    #[inline]
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            data: Vec::new(),
            format: TextureFormat::Rgba8UnormSrgb,
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
    #[cfg(feature = "image")]
    pub fn open_srgb(path: impl AsRef<Path>) -> Result<Self, image::ImageError> {
        let image = image::open(path)?;
        let width = image.width();
        let height = image.height();
        let data = image.into_rgba8().into_raw();
        Ok(Self::new(width, height, data))
    }

    #[inline]
    #[cfg(feature = "image")]
    pub fn open_rgb(path: impl AsRef<Path>) -> Result<Self, image::ImageError> {
        let image = image::open(path)?;
        let width = image.width();
        let height = image.height();
        let data = image.into_rgba8().into_raw();
        Ok(Self::with_format(
            width,
            height,
            data,
            TextureFormat::Rgba8Unorm,
        ))
    }

    #[inline]
    #[cfg(feature = "image")]
    pub fn open_hdr(path: impl AsRef<Path>) -> Result<Self, image::ImageError> {
        use lumi_util::bytemuck;

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
            ImageCopyTexture {
                texture: texture.texture(),
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            &self.data,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(4 * self.width),
                rows_per_image: None,
            },
            self.size(),
        );
    }

    #[inline]
    pub fn descriptor(&self) -> TextureDescriptor<'static> {
        TextureDescriptor {
            label: None,
            size: self.size(),
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: self.format,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        }
    }

    #[inline]
    pub fn extent(&self) -> Extent3d {
        Extent3d {
            width: self.width,
            height: self.height,
            depth_or_array_layers: 1,
        }
    }

    #[inline]
    pub fn data_aligned(&self) -> Cow<'_, [u8]> {
        if self.is_aligned() {
            return Cow::Borrowed(&self.data);
        }

        let bytes_per_row = self.bytes_per_row() as usize;
        let bytes_per_row_aligned = self.bytes_per_row_aligned() as usize;
        let mut data = vec![0; bytes_per_row_aligned as usize * self.height as usize];

        for y in 0..self.height as usize {
            let src_offset = y * bytes_per_row;
            let dst_offset = y * bytes_per_row_aligned;

            let src = &self.data[src_offset..src_offset + bytes_per_row];
            let dst = &mut data[dst_offset..dst_offset + bytes_per_row_aligned];

            dst.copy_from_slice(src);
        }

        Cow::Owned(data)
    }

    #[inline]
    pub fn copy_from_slice_aligned(&mut self, data: &[u8]) {
        if self.is_aligned() {
            self.data.copy_from_slice(data);
            return;
        }

        let bytes_per_row = self.bytes_per_row() as usize;
        let bytes_per_row_aligned = self.bytes_per_row_aligned() as usize;

        for y in 0..self.height as usize {
            let src_offset = y * bytes_per_row_aligned;
            let dst_offset = y * bytes_per_row;

            let src = &data[src_offset..src_offset + bytes_per_row_aligned];
            let dst = &mut self.data[dst_offset..dst_offset + bytes_per_row];

            dst.copy_from_slice(src);
        }
    }

    #[inline]
    pub fn bytes_per_row(&self) -> u32 {
        let info = self.format.describe();

        let block_size = info.block_size as u32;
        let block_width = info.block_dimensions.0 as u32;
        let components = info.components as u32;
        let bytes_per_row = self.width / block_width * block_size * components;

        bytes_per_row
    }

    #[inline]
    pub fn is_aligned(&self) -> bool {
        self.bytes_per_row() % wgpu::COPY_BYTES_PER_ROW_ALIGNMENT == 0
    }

    #[inline]
    pub fn bytes_per_row_aligned(&self) -> u32 {
        wgpu::util::align_to(self.bytes_per_row(), wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
    }

    #[inline]
    pub fn aligned_size(&self) -> u64 {
        self.bytes_per_row_aligned() as u64 * self.height as u64
    }

    #[inline]
    pub fn image_data_layout(&self) -> ImageDataLayout {
        ImageDataLayout {
            offset: 0,
            bytes_per_row: NonZeroU32::new(self.bytes_per_row()),
            rows_per_image: None,
        }
    }

    #[inline]
    pub fn image_data_layout_aligned(&self) -> ImageDataLayout {
        ImageDataLayout {
            offset: 0,
            bytes_per_row: NonZeroU32::new(self.bytes_per_row_aligned()),
            rows_per_image: None,
        }
    }

    #[inline]
    pub fn create_texture(&self, device: &Device, queue: &Queue) -> SharedTexture {
        let desc = self.descriptor();

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
}
