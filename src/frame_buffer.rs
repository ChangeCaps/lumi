use wgpu::{
    Color, CommandEncoder, Extent3d, LoadOp, Operations, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, RenderPassDescriptor, TextureDescriptor, TextureDimension,
    TextureFormat, TextureUsages,
};

use crate::{SharedDevice, SharedTexture, SharedTextureView};

#[derive(Clone, Debug)]
pub struct FrameBuffer {
    pub hdr: SharedTexture,
    pub hdr_view: SharedTextureView,
    pub hdr_msaa: Option<SharedTexture>,
    pub hdr_msaa_view: Option<SharedTextureView>,
    pub depth: SharedTexture,
    pub depth_view: SharedTextureView,
}

impl FrameBuffer {
    pub fn new(device: &SharedDevice, width: u32, height: u32, sample_count: u32) -> Self {
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

        let hdr_msaa = if sample_count > 1 {
            Some(device.create_shared_texture(&TextureDescriptor {
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
                usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            }))
        } else {
            None
        };

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
        let hdr_msaa_view = hdr_msaa
            .as_ref()
            .map(|t| t.create_view(&Default::default()));
        let depth_view = depth.create_view(&Default::default());

        Self {
            hdr,
            hdr_view,
            hdr_msaa,
            hdr_msaa_view,
            depth,
            depth_view,
        }
    }

    pub fn set_sample_count(&mut self, device: &SharedDevice, sample_count: u32) {
        if self.sample_count() != sample_count {
            self.depth = device.create_shared_texture(&TextureDescriptor {
                label: Some("Lumi Depth Target"),
                size: Extent3d {
                    width: self.width(),
                    height: self.height(),
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count,
                dimension: TextureDimension::D2,
                format: TextureFormat::Depth32Float,
                usage: TextureUsages::RENDER_ATTACHMENT,
            });

            self.depth_view = self.depth.create_view(&Default::default());
        }

        if self.hdr_msaa.is_some() && sample_count == 1 {
            self.hdr_msaa = None;
            self.hdr_msaa_view = None;
        } else if sample_count > 1 && self.sample_count() != sample_count {
            self.hdr_msaa = Some(device.create_shared_texture(&TextureDescriptor {
                label: Some("Lumi HDR MSAA Target"),
                size: self.size(),
                mip_level_count: 1,
                sample_count,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba16Float,
                usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            }));

            self.hdr_msaa_view = self
                .hdr_msaa
                .as_ref()
                .map(|t| t.create_view(&Default::default()));
        }
    }

    pub fn sample_count(&self) -> u32 {
        self.hdr_msaa.as_ref().map_or(1, |t| t.sample_count())
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

    pub fn resize(&mut self, device: &SharedDevice, width: u32, height: u32) {
        if self.width() != width || self.height() != height {
            *self = Self::new(device, width, height, self.sample_count());
        }
    }

    pub fn begin_hdr_render_pass<'a>(
        &'a self,
        encoder: &'a mut CommandEncoder,
    ) -> wgpu::RenderPass<'a> {
        encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Lumi HDR Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &self.hdr_msaa_view.as_ref().unwrap_or(&self.hdr_view),
                resolve_target: self.hdr_msaa_view.as_ref().map(|_| self.hdr_view.view()),
                ops: Operations {
                    load: LoadOp::Clear(Color::TRANSPARENT),
                    store: true,
                },
            })],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: &self.depth_view,
                depth_ops: Some(Operations {
                    load: LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        })
    }
}
