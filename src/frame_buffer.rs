use wgpu::{
    Color, CommandEncoder, Extent3d, LoadOp, Operations, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, RenderPassDescriptor, TextureDescriptor, TextureDimension,
    TextureFormat, TextureUsages,
};

use crate::{Device, SharedDevice, SharedTexture, SharedTextureView};

#[derive(Clone, Debug)]
pub struct FrameBuffer {
    pub hdr: SharedTexture,
    pub hdr_view: SharedTextureView,
    pub offscreen_hdr: SharedTexture,
    pub offscreen_hdr_view: SharedTextureView,
    pub depth: SharedTexture,
    pub depth_view: SharedTextureView,
}

impl FrameBuffer {
    pub fn new(device: &Device, width: u32, height: u32) -> Self {
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
            usage: TextureUsages::RENDER_ATTACHMENT
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_SRC,
        });

        let offscreen_hdr = device.create_shared_texture(&TextureDescriptor {
            label: Some("Lumi Offscreen HDR Target"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba16Float,
            usage: TextureUsages::RENDER_ATTACHMENT
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_SRC
                | TextureUsages::COPY_DST,
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

        let hdr_view = hdr.create_view(&Default::default());
        let offscreen_hdr_view = offscreen_hdr.create_view(&Default::default());
        let depth_view = depth.create_view(&Default::default());

        Self {
            hdr,
            hdr_view,
            offscreen_hdr,
            offscreen_hdr_view,
            depth,
            depth_view,
        }
    }

    pub fn width(&self) -> u32 {
        self.hdr.size().width
    }

    pub fn height(&self) -> u32 {
        self.hdr.size().height
    }

    pub fn size(&self) -> Extent3d {
        self.hdr.size()
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.width() as f32 / self.height() as f32
    }

    pub fn resize(&mut self, device: &Device, width: u32, height: u32) {
        if self.width() != width || self.height() != height {
            *self = Self::new(device, width, height);
        }
    }

    pub fn copy_offscreen(&self, encoder: &mut CommandEncoder) {
        encoder.copy_texture_to_texture(
            wgpu::ImageCopyTexture {
                texture: self.hdr.texture(),
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyTexture {
                texture: self.offscreen_hdr.texture(),
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            self.size(),
        );
    }

    pub fn begin_hdr_render_pass<'a>(
        &'a self,
        encoder: &'a mut CommandEncoder,
        load: bool,
    ) -> wgpu::RenderPass<'a> {
        let color_load = if load {
            LoadOp::Load
        } else {
            LoadOp::Clear(Color::TRANSPARENT)
        };

        let depth_load = if load {
            LoadOp::Load
        } else {
            LoadOp::Clear(1.0)
        };

        encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Lumi HDR Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &self.hdr_view,
                resolve_target: None,
                ops: Operations {
                    load: color_load,
                    store: true,
                },
            })],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: &self.depth_view,
                depth_ops: Some(Operations {
                    load: depth_load,
                    store: true,
                }),
                stencil_ops: None,
            }),
        })
    }
}
