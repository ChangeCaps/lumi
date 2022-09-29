use std::{collections::HashMap, num::NonZeroU32};

use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BindingResource, BindingType, BlendState, BufferBindingType, BufferUsages, ColorTargetState,
    ColorWrites, CompareFunction, DepthStencilState, Device, Extent3d, FragmentState,
    MultisampleState, PipelineLayoutDescriptor, Queue, RenderPass, RenderPipeline,
    RenderPipelineDescriptor, ShaderStages, StorageTextureAccess, TextureDescriptor,
    TextureDimension, TextureFormat, TextureSampleType, TextureUsages, TextureViewDescriptor,
    TextureViewDimension, VertexState,
};

use crate::{
    bind::Bind,
    binding::{Bindings, BindingsLayout},
    camera::RawCamera,
    id::{CameraId, EnvironmentId},
    prelude::{ImageData, RenderTarget, ShaderRef, World},
    shader::ShaderProcessor,
    SharedDevice, SharedTexture, SharedTextureView,
};

pub struct Environment {
    image: ImageData,
    id: EnvironmentId,
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            image: ImageData::with_format(
                4096,
                2048,
                include_bytes!("default_env").to_vec(),
                TextureFormat::Rgba16Uint,
            ),
            id: EnvironmentId::new(),
        }
    }
}

impl Environment {
    pub fn new(image: ImageData) -> Self {
        Self {
            image,
            id: EnvironmentId::new(),
        }
    }

    pub fn image(&self) -> &ImageData {
        &self.image
    }

    pub fn id(&self) -> EnvironmentId {
        self.id
    }
}

pub struct PreparedEnvironment {
    diffuse_texture: SharedTexture,
    diffuse_view: SharedTextureView,
    specular_texture: SharedTexture,
    specular_view: SharedTextureView,
    integrated_brdf: SharedTextureView,
    id: EnvironmentId,
}

impl PreparedEnvironment {
    pub fn new(
        device: &Device,
        queue: &Queue,
        environemnt: &Environment,
        integrated_brdf: SharedTextureView,
    ) -> Self {
        let size = environemnt.image().size().height * 3 / 4;

        let environemnt_view = environemnt.image().create_view(device, queue);

        let specular_mips = 5;
        let specular_texture = device.create_shared_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: 6,
            },
            mip_level_count: specular_mips,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba16Float,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::STORAGE_BINDING
                | TextureUsages::COPY_DST,
        });

        let diffuse_texture = device.create_shared_texture(&TextureDescriptor {
            label: Some("Environment Diffuse Texture"),
            size: Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: 6,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba16Float,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::STORAGE_BINDING
                | TextureUsages::COPY_DST,
        });
        let diffuse_view = diffuse_texture.create_view(&Default::default());

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Uint,
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: TextureFormat::Rgba16Float,
                        view_dimension: TextureViewDimension::D2Array,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Environment Pipeline"),
            layout: Some(&pipeline_layout),
            module: &device.create_shader_module(wgpu::include_wgsl!("environment.wgsl")),
            entry_point: "specular",
        });

        let mut bind_groups = Vec::new();

        for i in 0..specular_mips {
            let specular_view = specular_texture.create_view(&TextureViewDescriptor {
                base_mip_level: i,
                mip_level_count: NonZeroU32::new(1),
                ..Default::default()
            });

            let buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: bytemuck::bytes_of(&(i as f32 / (specular_mips - 1) as f32)),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            });

            let bind_group = device.create_bind_group(&BindGroupDescriptor {
                label: None,
                layout: &bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&environemnt_view),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::TextureView(&specular_view),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: buffer.as_entire_binding(),
                    },
                ],
            });

            bind_groups.push(bind_group);
        }

        let mut encoder = device.create_command_encoder(&Default::default());

        let mut compute_pass = encoder.begin_compute_pass(&Default::default());

        compute_pass.set_pipeline(&pipeline);

        for i in 0..specular_mips {
            compute_pass.set_bind_group(0, &bind_groups[i as usize], &[]);
            compute_pass.dispatch_workgroups(size / 16 + 16, size / 16 + 16, 6);
        }

        drop(compute_pass);

        queue.submit(std::iter::once(encoder.finish()));

        let specular_view = specular_texture.create_view(&TextureViewDescriptor {
            dimension: Some(TextureViewDimension::Cube),
            ..Default::default()
        });
        let diffuse_view = diffuse_texture.create_view(&TextureViewDescriptor {
            dimension: Some(TextureViewDimension::Cube),
            ..Default::default()
        });

        Self {
            diffuse_texture,
            diffuse_view,
            specular_texture,
            specular_view,
            integrated_brdf,
            id: environemnt.id(),
        }
    }

    pub fn bindings(&self) -> EnvironmentBindings {
        EnvironmentBindings {
            diffuse_texture: &self.diffuse_view,
            specular_texture: &self.specular_view,
            integrated_brdf: &self.integrated_brdf,
        }
    }
}

#[derive(Bind)]
pub struct EnvironmentBindings<'a> {
    #[sampler(name = "environment_sampler")]
    #[texture(name = "environment_diffuse", dimension = cube)]
    pub diffuse_texture: &'a SharedTextureView,
    #[texture(name = "environment_specular", dimension = cube)]
    pub specular_texture: &'a SharedTextureView,
    #[texture]
    pub integrated_brdf: &'a SharedTextureView,
}

#[derive(Bind)]
pub struct SkyBindings {
    #[uniform]
    pub camera: RawCamera,
}

pub struct Sky {
    pub bindings_layout: BindingsLayout,
    pub bindings: HashMap<CameraId, Bindings>,
    pub pipeline: RenderPipeline,
    pub integrated_brdf: SharedTextureView,
}

impl Sky {
    pub fn new(device: &Device, queue: &Queue, shader_processor: &mut ShaderProcessor) -> Self {
        let mut vertex = shader_processor
            .process(ShaderRef::module("lumi/fullscreen_vert.wgsl"))
            .unwrap();
        let mut fragment = shader_processor
            .process(ShaderRef::module("lumi/default_env_frag.wgsl"))
            .unwrap();
        vertex.rebind(&mut fragment).unwrap();

        let bindings_layout = BindingsLayout::new()
            .with_shader(&vertex)
            .with_shader(&fragment)
            .bind::<EnvironmentBindings>()
            .bind::<SkyBindings>();

        let bind_group_layouts = bindings_layout.create_bind_group_layouts(device);
        let bind_group_layouts = bind_group_layouts.iter().collect::<Vec<_>>();

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &bind_group_layouts,
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Sky Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &vertex.shader_module(device),
                entry_point: "vertex",
                buffers: &[],
            },
            fragment: Some(FragmentState {
                module: &fragment.shader_module(device),
                entry_point: "fragment",
                targets: &[Some(ColorTargetState {
                    format: TextureFormat::Rgba16Float,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: Default::default(),
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: CompareFunction::Always,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: MultisampleState {
                count: 4,
                ..Default::default()
            },
            multiview: Default::default(),
        });

        let integrated_brdf = device.create_shared_texture_with_data(
            queue,
            &TextureDescriptor {
                label: Some("Integrated BRDF"),
                size: Extent3d {
                    width: 400,
                    height: 400,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8Unorm,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            },
            &include_bytes!("integrated_brdf")[..],
        );

        Self {
            bindings_layout,
            bindings: HashMap::new(),
            pipeline,
            integrated_brdf: integrated_brdf.create_view(&Default::default()),
        }
    }

    pub fn prepare(
        &mut self,
        device: &Device,
        queue: &Queue,
        world: &World,
        target: &RenderTarget,
        environment: &PreparedEnvironment,
    ) {
        for (camera_id, camera) in world.iter_cameras() {
            let bindings = self
                .bindings
                .entry(camera_id)
                .or_insert_with(|| self.bindings_layout.create_bindings(device));

            let aspect = camera.target.get_aspect(target);

            bindings.bind(
                device,
                queue,
                &SkyBindings {
                    camera: camera.raw_aspect(aspect),
                },
            );
            bindings.bind(device, queue, &environment.bindings());

            bindings.update_bind_groups(device);
        }
    }

    pub fn render<'a>(&'a self, render_pass: &mut RenderPass<'a>, camera_id: CameraId) {
        render_pass.set_pipeline(&self.pipeline);
        self.bindings[&camera_id].apply(render_pass);
        render_pass.draw(0..3, 0..1);
    }
}
