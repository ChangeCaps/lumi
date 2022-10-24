use std::num::NonZeroU32;

use lumi_bind::{Bind, Bindings, BindingsLayout};
use lumi_core::{
    BlendComponent, BlendFactor, BlendOperation, BlendState, Color, ColorTargetState, ColorWrites,
    CommandEncoder, Device, Extent3d, FragmentState, LoadOp, Operations, PipelineLayoutDescriptor,
    Queue, RenderPass, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, SharedDevice, SharedTexture, SharedTextureView, TextureDescriptor,
    TextureDimension, TextureFormat, TextureUsages, TextureViewDescriptor, VertexState,
};
use lumi_shader::{ShaderProcessor, ShaderRef};
use lumi_util::math::Vec3;

use crate::Renderer;

pub struct MipChain {
    pub texture: SharedTexture,
    pub view: SharedTextureView,
    pub mip_level_count: Option<u32>,
    pub views: Vec<SharedTextureView>,
    pub bindings: Vec<Bindings>,
}

impl MipChain {
    pub fn new(
        device: &Device,
        layout: &BindingsLayout,
        width: u32,
        height: u32,
        mip_level_count: Option<u32>,
    ) -> Self {
        let mips = mip_level_count.unwrap_or(Self::mip_levels_for_size(width, height));
        let texture = device.create_shared_texture(&TextureDescriptor {
            label: Some("Lumi MipChain texture"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: mips,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba16Float,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT,
        });
        let view = texture.create_view(&Default::default());

        let mut views = Vec::with_capacity(mips as usize);

        for i in 0..mips {
            views.push(texture.create_view(&TextureViewDescriptor {
                label: Some("Lumi MipChain view"),
                base_mip_level: i,
                mip_level_count: NonZeroU32::new(1),
                ..Default::default()
            }));
        }

        let bindings = (0..mips).map(|_| layout.create_bindings(device)).collect();

        Self {
            texture,
            view,
            mip_level_count,
            views,
            bindings,
        }
    }

    pub fn mip_levels_for_size(width: u32, height: u32) -> u32 {
        let min_dimension = u32::min(width, height);

        let mut mip_levels = 1;
        while min_dimension >> mip_levels > 4 {
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

    pub fn size(&self) -> Extent3d {
        self.texture.size()
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

    pub fn prepare_downsample_bindings(
        &mut self,
        device: &Device,
        queue: &Queue,
        source: &SharedTextureView,
        scale: f32,
    ) {
        let scale = if self.mip_level_count.is_some() {
            scale
        } else {
            self.filter_scale() * scale
        };

        self.bindings[0].bind(
            device,
            queue,
            &DownsampleBindings {
                source_texture: source,
                scale: scale * 0.1,
                threshold: 0.0,
                curve: Vec3::ZERO,
            },
        );

        self.bindings[0].update_bind_groups(device);

        for mip in 1..self.mip_levels() as usize {
            self.bindings[mip].bind(
                device,
                queue,
                &DownsampleBindings {
                    source_texture: &self.views[mip - 1],
                    scale,
                    threshold: 0.0,
                    curve: Vec3::ZERO,
                },
            );

            self.bindings[mip].update_bind_groups(device);
        }
    }

    pub fn downsample(&self, pipeline: &MipChainPipeline, encoder: &mut CommandEncoder) {
        for mip in 0..self.mip_levels() {
            let mut pass = self.begin_render_pass(encoder, mip);

            pass.set_pipeline(&pipeline.downsample_pipeline);
            self.bindings[mip as usize].apply(&mut pass);
            pass.draw(0..3, 0..1);
        }
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

pub struct MipChainPipeline {
    pub down_layout: BindingsLayout,
    pub up_layout: BindingsLayout,
    pub downsample_pipeline: RenderPipeline,
    pub upsample_pipeline: RenderPipeline,
}

impl MipChainPipeline {
    pub fn init(renderer: &mut Renderer, device: &Device) {
        if renderer.resources.contains::<Self>() {
            return;
        }

        let shader_processor = renderer.resources.get_mut::<ShaderProcessor>().unwrap();
        let pipeline = Self::new(device, shader_processor);
        renderer.resources.insert(pipeline);
    }

    pub fn new(device: &Device, shader_processor: &mut ShaderProcessor) -> Self {
        let mut vertex = shader_processor
            .process(
                ShaderRef::module("lumi/fullscreen_vert.wgsl"),
                &Default::default(),
            )
            .unwrap();
        let mut fragment = shader_processor
            .process(
                ShaderRef::module("lumi/bloom_frag.wgsl"),
                &Default::default(),
            )
            .unwrap();
        vertex.rebind_with(&mut fragment).unwrap();

        let down_layout = BindingsLayout::new()
            .with_shader(&vertex)
            .with_shader(&fragment)
            .bind::<DownsampleBindings>();

        let up_layout = BindingsLayout::new()
            .with_shader(&vertex)
            .with_shader(&fragment)
            .bind::<UpsampleBindings>();

        let bind_group_layouts = down_layout.create_bind_group_layouts(device);
        let bind_group_layouts = bind_group_layouts.iter().collect::<Vec<_>>();
        let downsample_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Lumi Bloom downsample pipeline layout"),
            bind_group_layouts: &bind_group_layouts,
            push_constant_ranges: &[],
        });

        let bind_group_layouts = up_layout.create_bind_group_layouts(device);
        let bind_group_layouts = bind_group_layouts.iter().collect::<Vec<_>>();
        let upsample_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
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
            down_layout,
            up_layout,
            downsample_pipeline,
            upsample_pipeline,
        }
    }
}
