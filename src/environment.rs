use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BindingResource, BindingType, Device, Extent3d, PipelineLayoutDescriptor, Queue, ShaderStages,
    StorageTextureAccess, TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType,
    TextureUsages, TextureViewDescriptor, TextureViewDimension,
};

use crate::{
    bind::Bind, id::EnvironmentId, prelude::ImageData, SharedDevice, SharedTexture,
    SharedTextureView,
};

pub struct Environment {
    image: ImageData,
    id: EnvironmentId,
}

impl Default for Environment {
    fn default() -> Self {
        let image = image::load_from_memory_with_format(
            include_bytes!("default_env.hdr"),
            image::ImageFormat::Hdr,
        )
        .unwrap()
        .to_rgba16();

        let data = bytemuck::cast_slice(image.as_raw());

        Self {
            image: ImageData::with_format(
                image.width(),
                image.height(),
                data.to_vec(),
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
    id: EnvironmentId,
}

impl PreparedEnvironment {
    pub fn new(device: &Device, queue: &Queue, environemnt: &Environment) -> Self {
        let size = environemnt.image().size().height * 2 / 3;

        let environemnt_view = environemnt.image().create_view(device, queue);

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
            ],
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
                    resource: BindingResource::TextureView(&diffuse_view),
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
            entry_point: "main",
        });

        let mut encoder = device.create_command_encoder(&Default::default());

        let mut compute_pass = encoder.begin_compute_pass(&Default::default());

        compute_pass.set_pipeline(&pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[]);
        compute_pass.dispatch_workgroups(size / 16 + 16, size / 16 + 16, 6);

        drop(compute_pass);

        queue.submit(std::iter::once(encoder.finish()));

        let diffuse_view = diffuse_texture.create_view(&TextureViewDescriptor {
            dimension: Some(TextureViewDimension::Cube),
            ..Default::default()
        });

        Self {
            id: environemnt.id(),
            diffuse_texture,
            diffuse_view,
        }
    }

    pub fn bindings(&self) -> EnvironmentBindings {
        EnvironmentBindings {
            diffuse_texture: &self.diffuse_view,
        }
    }
}

#[derive(Bind)]
pub struct EnvironmentBindings<'a> {
    #[sampler(name = "environment_sampler")]
    #[texture(name = "environment_diffuse", dimension = cube)]
    pub diffuse_texture: &'a SharedTextureView,
}
