use std::num::NonZeroU32;

use glam::Vec3;
use wgpu::{
    BlendComponent, BlendFactor, BlendOperation, BlendState, Color, ColorTargetState, ColorWrites,
    CommandEncoder, Extent3d, FragmentState, LoadOp, Operations, RenderPass,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    TextureFormat, TextureUsages, VertexState,
};

use crate::{
    Bind, Bindings, BindingsLayout, Shader, ShaderProcessor, ShaderRef, SharedDevice, SharedQueue,
    SharedTexture, SharedTextureView,
};

struct MipChain {
    texture: SharedTexture,
    views: Vec<SharedTextureView>,
    bindings: Vec<Bindings>,
}

impl MipChain {
    pub fn new(device: &SharedDevice, layout: &BindingsLayout, width: u32, height: u32) -> Self {
        let mip_level_count = Self::mip_levels_for_size(width, height);
        let texture = device.create_shared_texture(&wgpu::TextureDescriptor {
            label: Some("Lumi MipChain texture"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TextureFormat::Rgba16Float,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT,
        });

        let mut views = Vec::with_capacity(mip_level_count as usize);

        for i in 0..mip_level_count {
            views.push(texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some("Lumi MipChain view"),
                base_mip_level: i,
                mip_level_count: NonZeroU32::new(1),
                ..Default::default()
            }));
        }

        let bindings = (0..mip_level_count)
            .map(|_| layout.create_bindings(device))
            .collect();

        Self {
            texture,
            views,
            bindings,
        }
    }

    pub fn mip_levels_for_size(width: u32, height: u32) -> u32 {
        let min_dimension = u32::min(width, height);

        let mut mip_levels = 1;
        while min_dimension >> mip_levels > 8 {
            mip_levels += 1;
        }

        mip_levels
    }

    pub fn begin_render_pass<'a>(
        &'a self,
        encoder: &'a mut CommandEncoder,
        mip_level: u32,
    ) -> RenderPass<'a> {
        encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Lumi MipChain render pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &self.views[mip_level as usize],
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color::TRANSPARENT),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        })
    }

    pub fn width(&self) -> u32 {
        self.texture.size().width
    }

    pub fn height(&self) -> u32 {
        self.texture.size().height
    }

    pub fn mip_levels(&self) -> u32 {
        self.texture.mip_level_count()
    }

    pub fn filter_scale(&self) -> f32 {
        let min_dimension = u32::min(self.width(), self.height());
        (min_dimension >> self.mip_levels()) as f32 / 4.0
    }
}

#[derive(Bind)]
pub struct DownsampleBindings<'a> {
    #[uniform]
    pub scale: f32,
    #[uniform]
    pub threshold: f32,
    #[uniform]
    pub curve: Vec3,
    #[texture]
    #[sampler(name = "source_sampler")]
    pub source_texture: &'a SharedTextureView,
}

#[derive(Bind)]
pub struct UpsampleBindings<'a> {
    #[uniform]
    pub scale: f32,
    #[texture]
    #[sampler(name = "source_sampler")]
    pub source_texture: &'a SharedTextureView,
    #[texture]
    pub source_mip_texture: &'a SharedTextureView,
}

pub struct Bloom {
    down: MipChain,
    up: MipChain,
    down_layout: BindingsLayout,
    up_layout: BindingsLayout,
    downsample_pipeline: RenderPipeline,
    upsample_pipeline: RenderPipeline,
}

impl Bloom {
    pub fn new(
        device: &SharedDevice,
        shader_processor: &mut ShaderProcessor,
        width: u32,
        height: u32,
    ) -> Self {
        let vertex = shader_processor
            .process(ShaderRef::module("lumi/fullscreen_vert.wgsl"))
            .unwrap();
        let fragment = shader_processor
            .process(ShaderRef::module("lumi/bloom_frag.wgsl"))
            .unwrap();

        let mut vertex = Shader::from_wgsl(&vertex).unwrap();
        let mut fragment = Shader::from_wgsl(&fragment).unwrap();
        vertex.rebind(&mut fragment).unwrap();

        let down_layout = BindingsLayout::new()
            .with_shader(&vertex)
            .with_shader(&fragment)
            .bind::<DownsampleBindings>();

        let up_layout = BindingsLayout::new()
            .with_shader(&vertex)
            .with_shader(&fragment)
            .bind::<UpsampleBindings>();

        let down = MipChain::new(device, &down_layout, width, height);
        let up = MipChain::new(device, &up_layout, width, height);

        let bind_group_layouts = down_layout.create_bind_group_layouts(device);
        let bind_group_layouts = bind_group_layouts.iter().collect::<Vec<_>>();
        let downsample_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Lumi Bloom downsample pipeline layout"),
                bind_group_layouts: &bind_group_layouts,
                push_constant_ranges: &[],
            });

        let bind_group_layouts = up_layout.create_bind_group_layouts(device);
        let bind_group_layouts = bind_group_layouts.iter().collect::<Vec<_>>();
        let upsample_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Lumi Bloom upsample pipeline layout"),
                bind_group_layouts: &bind_group_layouts,
                push_constant_ranges: &[],
            });

        let additive_blending = BlendState {
            color: BlendComponent {
                src_factor: BlendFactor::One,
                dst_factor: BlendFactor::One,
                operation: BlendOperation::Add,
            },
            alpha: BlendComponent::REPLACE,
        };

        let downsample_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Lumi Bloom downsample pipeline"),
            layout: Some(&downsample_pipeline_layout),
            vertex: VertexState {
                module: vertex.shader_module(device),
                entry_point: "vertex",
                buffers: &[],
            },
            fragment: Some(FragmentState {
                module: fragment.shader_module(device),
                entry_point: "downsample",
                targets: &[Some(ColorTargetState {
                    format: TextureFormat::Rgba16Float,
                    blend: Some(additive_blending),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: Default::default(),
            depth_stencil: Default::default(),
            multisample: Default::default(),
            multiview: Default::default(),
        });

        let upsample_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Lumi Bloom upsample pipeline"),
            layout: Some(&upsample_pipeline_layout),
            vertex: VertexState {
                module: vertex.shader_module(device),
                entry_point: "vertex",
                buffers: &[],
            },
            fragment: Some(FragmentState {
                module: fragment.shader_module(device),
                entry_point: "upsample",
                targets: &[Some(ColorTargetState {
                    format: TextureFormat::Rgba16Float,
                    blend: Some(additive_blending),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: Default::default(),
            depth_stencil: Default::default(),
            multisample: Default::default(),
            multiview: Default::default(),
        });

        Self {
            down,
            up,
            down_layout,
            up_layout,
            downsample_pipeline,
            upsample_pipeline,
        }
    }

    pub fn resize(&mut self, device: &SharedDevice, width: u32, height: u32) {
        if self.down.width() != width || self.down.height() != height {
            self.down = MipChain::new(device, &self.down_layout, width, height);
            self.up = MipChain::new(device, &self.up_layout, width, height);
        }
    }

    pub fn render(
        &mut self,
        device: &SharedDevice,
        queue: &SharedQueue,
        encoder: &mut CommandEncoder,
        source: &SharedTextureView,
        threshold: f32,
        knee: f32,
        scale: f32,
    ) {
        let scale = self.down.filter_scale() * scale;

        let curve = Vec3::new(threshold - knee, knee * 2.0, 0.25 / knee);

        self.down.bindings[0].bind(
            device,
            queue,
            &DownsampleBindings {
                source_texture: source,
                scale,
                threshold,
                curve,
            },
        );

        self.down.bindings[0].update_bind_groups(device);

        for mip in 1..self.down.mip_levels() as usize {
            self.down.bindings[mip].bind(
                device,
                queue,
                &DownsampleBindings {
                    source_texture: &self.down.views[mip - 1],
                    scale,
                    threshold: 0.0,
                    curve: Vec3::ZERO,
                },
            );

            self.down.bindings[mip].update_bind_groups(device);
        }

        for mip in 0..self.up.mip_levels() as usize {
            let source_mip_texture = if mip == self.up.mip_levels() as usize - 1 {
                &self.down.views[mip]
            } else {
                &self.up.views[mip + 1]
            };

            self.up.bindings[mip].bind(
                device,
                queue,
                &UpsampleBindings {
                    source_texture: &self.down.views[mip],
                    source_mip_texture,
                    scale,
                },
            );

            self.up.bindings[mip].update_bind_groups(device);
        }

        for mip in 0..self.down.mip_levels() {
            let mut pass = self.down.begin_render_pass(encoder, mip);

            pass.set_pipeline(&self.downsample_pipeline);
            self.down.bindings[mip as usize].bind_pass(&mut pass);
            pass.draw(0..3, 0..1);
        }

        for mip in (1..self.up.mip_levels()).rev() {
            let mut pass = self.up.begin_render_pass(encoder, mip);

            pass.set_pipeline(&self.upsample_pipeline);
            self.up.bindings[mip as usize].bind_pass(&mut pass);
            pass.draw(0..3, 0..1);
        }

        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Lumi Bloom upsample pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: source,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        pass.set_pipeline(&self.upsample_pipeline);
        self.up.bindings[0].bind_pass(&mut pass);
        pass.draw(0..3, 0..1);
    }
}
