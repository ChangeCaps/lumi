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
    pub hdr_msaa: Option<SharedTexture>,
    pub hdr_msaa_view: Option<SharedTextureView>,
    pub offscreen_hdr: SharedTexture,
    pub offscreen_hdr_view: SharedTextureView,
    pub depth: SharedTexture,
    pub depth_view: SharedTextureView,
}

impl FrameBuffer {
    pub fn new(device: &Device, width: u32, height: u32, sample_count: u32) -> Self {
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

        let (hdr_msaa, hdr_msaa_view) = if sample_count > 1 {
            let hdr_msaa = device.create_shared_texture(&TextureDescriptor {
                label: Some("Lumi HDR MSAA Target"),
                size: Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba16Float,
                usage: TextureUsages::RENDER_ATTACHMENT
                    | TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_SRC,
            });

            let hdr_msaa_view = hdr_msaa.create_view(&Default::default());

            (Some(hdr_msaa), Some(hdr_msaa_view))
        } else {
            (None, None)
        };

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
            sample_count,
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
            hdr_msaa,
            hdr_msaa_view,
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

    pub fn sample_count(&self) -> u32 {
        self.hdr_msaa.as_ref().map_or(1, |t| t.sample_count())
    }

    pub fn size(&self) -> Extent3d {
        self.hdr.size()
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.width() as f32 / self.height() as f32
    }

    pub fn resize(&mut self, device: &Device, width: u32, height: u32, sample_count: u32) {
        if self.width() != width || self.height() != height || self.sample_count() != sample_count {
            *self = Self::new(device, width, height, sample_count);
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

        let (view, resolve_target) = if let Some(msaa) = &self.hdr_msaa_view {
            (msaa, Some(self.hdr_view.view()))
        } else {
            (&self.hdr_view, None)
        };

        encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Lumi HDR Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view,
                resolve_target,
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
