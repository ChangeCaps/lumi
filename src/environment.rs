#[allow(unused_imports)]
use std::path::Path;

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
    util::HashMap,
    SharedDevice, SharedTexture, SharedTextureView,
};

pub struct PreparedEnvironment {
    sky_view: SharedTextureView,
    diffuse_view: SharedTextureView,
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

        let sky_texture = SharedTexture::new(
            baked_env.sky,
            &TextureDescriptor {
                label: None,
                size: Extent3d {
                    width: baked_env.sky_size,
                    height: baked_env.sky_size,
                    depth_or_array_layers: 6,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba16Float,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            },
        );

        let sky_view = sky_texture.create_view(&TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::Cube),
            ..Default::default()
        });

        Self {
            sky_view,
            diffuse_view,
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
    #[texture(name = "environment_diffuse", dimension = cube)]
    #[sampler(name = "environment_sampler")]
    pub diffuse_texture: &'a SharedTextureView,
    #[texture(name = "environment_specular", dimension = cube)]
    pub specular_texture: &'a SharedTextureView,
    #[texture]
    #[sampler(name = "integrated_brdf_sampler")]
    pub integrated_brdf: &'a SharedTextureView,
}
