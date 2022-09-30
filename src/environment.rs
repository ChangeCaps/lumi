use std::collections::HashMap;

use lumi_bake::{BakedEnvironment, EnvironmentData};
use wgpu::{
    BlendState, ColorTargetState, ColorWrites, CompareFunction, DepthStencilState, Device,
    Extent3d, FragmentState, MultisampleState, PipelineLayout, PipelineLayoutDescriptor, Queue,
    RenderPass, RenderPipeline, RenderPipelineDescriptor, ShaderModule, TextureDescriptor,
    TextureDimension, TextureFormat, TextureUsages, TextureViewDescriptor, VertexState,
};

use crate::{
    bind::Bind,
    binding::{Bindings, BindingsLayout},
    camera::RawCamera,
    id::{CameraId, EnvironmentId},
    prelude::{ImageData, RenderTarget, ShaderRef, World},
    shader::{DefaultShader, ShaderProcessor},
    SharedDevice, SharedTexture, SharedTextureView,
};

pub enum EnvironmentKind {
    Baked(EnvironmentData),
    RealTime(ImageData),
}

pub struct Environment {
    kind: EnvironmentKind,
    id: EnvironmentId,
}

impl Default for Environment {
    fn default() -> Self {
        let data = EnvironmentData::from_bytes(&include_bytes!("default_env.bake")[..]).unwrap();

        Self {
            kind: EnvironmentKind::Baked(data),
            id: EnvironmentId::new(),
        }
    }
}

impl Environment {
    pub fn new(image: ImageData) -> Self {
        Self {
            kind: EnvironmentKind::RealTime(image),
            id: EnvironmentId::new(),
        }
    }

    pub fn bake(&self, device: &Device, queue: &Queue) -> BakedEnvironment {
        match &self.kind {
            EnvironmentKind::Baked(data) => BakedEnvironment::from_data(device, queue, data),
            EnvironmentKind::RealTime(image) => BakedEnvironment::from_eq_bytes(
                device,
                queue,
                &image.data,
                image.width,
                image.height,
            ),
        }
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
        let baked_env = environemnt.bake(device, queue);

        let diffuse_texture = SharedTexture::new(
            baked_env.irradiance,
            &TextureDescriptor {
                label: None,
                size: Extent3d {
                    width: baked_env.irradiance_size,
                    height: baked_env.irradiance_size,
                    depth_or_array_layers: 6,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba16Float,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            },
        );

        let diffuse_view = diffuse_texture.create_view(&TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::Cube),
            ..Default::default()
        });

        let specular_texture = SharedTexture::new(
            baked_env.indirect,
            &TextureDescriptor {
                label: None,
                size: Extent3d {
                    width: baked_env.indirect_size,
                    height: baked_env.indirect_size,
                    depth_or_array_layers: 6,
                },
                mip_level_count: baked_env.indirect_mip_levels,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba16Float,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            },
        );

        let specular_view = specular_texture.create_view(&TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::Cube),
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

    pub fn prepare(&mut self, device: &Device, queue: &Queue, environemnt: &Environment) {
        if self.id != environemnt.id() {
            *self = Self::new(device, queue, environemnt, self.integrated_brdf.clone());
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
    pub vertex: ShaderModule,
    pub fragment: ShaderModule,
    pub bindings_layout: BindingsLayout,
    pub bindings: HashMap<CameraId, Bindings>,
    pub pipeline_layout: PipelineLayout,
    pub pipeline: RenderPipeline,
    pub sample_count: u32,
    pub integrated_brdf: SharedTextureView,
}

impl Sky {
    fn create_pipeline(
        device: &Device,
        pipeline_layout: &PipelineLayout,
        vertex: &ShaderModule,
        fragment: &ShaderModule,
        sample_count: u32,
    ) -> RenderPipeline {
        device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Sky Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: vertex,
                entry_point: "vertex",
                buffers: &[],
            },
            fragment: Some(FragmentState {
                module: fragment,
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
                count: sample_count,
                ..Default::default()
            },
            multiview: Default::default(),
        })
    }

    pub fn new(
        device: &Device,
        queue: &Queue,
        shader_processor: &mut ShaderProcessor,
        sample_count: u32,
    ) -> Self {
        let mut vertex = shader_processor
            .process(ShaderRef::module("lumi/fullscreen_vert.wgsl"))
            .unwrap();
        let mut fragment = shader_processor
            .process(ShaderRef::Default(DefaultShader::Sky))
            .unwrap();
        vertex.rebind(&mut fragment).unwrap();
        vertex.compile(device).unwrap();
        fragment.compile(device).unwrap();

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

        let vertex = vertex.create_shader_module(device);
        let fragment = fragment.create_shader_module(device);
        let pipeline =
            Self::create_pipeline(device, &pipeline_layout, &vertex, &fragment, sample_count);

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
                usage: TextureUsages::TEXTURE_BINDING,
            },
            &include_bytes!("integrated_brdf")[..],
        );

        Self {
            vertex,
            fragment,
            bindings_layout,
            bindings: HashMap::new(),
            pipeline_layout,
            pipeline,
            sample_count,
            integrated_brdf: integrated_brdf.create_view(&Default::default()),
        }
    }

    pub fn prepare(
        &mut self,
        device: &Device,
        queue: &Queue,
        world: &World,
        sample_count: u32,
        target: &RenderTarget,
        environment: &PreparedEnvironment,
    ) {
        if self.sample_count != sample_count {
            self.sample_count = sample_count;
            self.pipeline = Self::create_pipeline(
                device,
                &self.pipeline_layout,
                &self.vertex,
                &self.fragment,
                sample_count,
            );
        }

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
