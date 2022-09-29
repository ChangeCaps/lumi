use std::{
    io::{self, prelude::*},
    num::NonZeroU32,
};

use wgpu::util::DeviceExt;

pub struct EnvironmentData {
    pub irradiance_size: u32,
    pub irradiance_data: Vec<u8>,
    pub indirect_size: u32,
    pub indirect_mip_levels: u32,
    pub indirect_data: Vec<u8>,
}

impl EnvironmentData {
    const PIXEL_SIZE: usize = 8;

    pub fn load<T: Read>(mut source: T) -> io::Result<Self> {
        macro_rules! read {
            ($source:expr, $type:ty) => {{
                let mut buf = [0; std::mem::size_of::<$type>()];
                $source.read_exact(&mut buf)?;
                <$type>::from_le_bytes(buf)
            }};
        }

        let irradiance_size = read!(source, u32);
        let irradiance_data_size =
            (irradiance_size * irradiance_size) as usize * Self::PIXEL_SIZE * 6;
        let mut irradiance_data = vec![0u8; irradiance_data_size];
        source.read_exact(&mut irradiance_data)?;

        let indirect_size = read!(source, u32);
        let indirect_mip_levels = read!(source, u32);
        let indirect_data_size =
            BakedEnvironment::texture_size(indirect_size, indirect_size, indirect_mip_levels);

        let mut indirect_data = vec![0u8; indirect_data_size];
        source.read_exact(&mut indirect_data)?;

        Ok(Self {
            irradiance_size,
            irradiance_data,
            indirect_size,
            indirect_mip_levels,
            indirect_data,
        })
    }

    pub fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        Self::load(bytes)
    }

    pub fn save<T: Write>(&self, mut dest: T) -> io::Result<()> {
        macro_rules! write {
            ($dest:expr, $value:expr) => {{
                let buf = $value.to_le_bytes();
                $dest.write_all(&buf)?;
            }};
        }

        write!(dest, self.irradiance_size);
        dest.write_all(&self.irradiance_data)?;

        write!(dest, self.indirect_size);
        write!(dest, self.indirect_mip_levels);
        dest.write_all(&self.indirect_data)?;

        Ok(())
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        self.save(&mut bytes).unwrap();
        bytes
    }
}

pub struct BakedEnvironment {
    pub irradiance_size: u32,
    pub irradiance: wgpu::Texture,
    pub irradiance_view: wgpu::TextureView,
    pub indirect_size: u32,
    pub indirect_mip_levels: u32,
    pub indirect: wgpu::Texture,
    pub indirect_view: wgpu::TextureView,
}

impl BakedEnvironment {
    pub fn create_irradiance(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        environment_data: &EnvironmentData,
    ) -> wgpu::Texture {
        device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some("Irradiance"),
                size: wgpu::Extent3d {
                    width: environment_data.irradiance_size,
                    height: environment_data.irradiance_size,
                    depth_or_array_layers: 6,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba32Float,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            },
            &environment_data.irradiance_data,
        )
    }

    pub fn create_indirect(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        environment_data: &EnvironmentData,
    ) -> wgpu::Texture {
        device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some("Indirect"),
                size: wgpu::Extent3d {
                    width: environment_data.indirect_size,
                    height: environment_data.indirect_size,
                    depth_or_array_layers: 6,
                },
                mip_level_count: environment_data.indirect_mip_levels,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba32Float,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            },
            &environment_data.indirect_data,
        )
    }

    fn aligned_width(width: u32) -> u32 {
        wgpu::util::align_to(
            width,
            wgpu::COPY_BYTES_PER_ROW_ALIGNMENT / EnvironmentData::PIXEL_SIZE as u32,
        )
    }

    pub fn read_texture(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture: &wgpu::Texture,
        width: u32,
        height: u32,
        mip_levels: u32,
    ) -> Vec<u8> {
        let size = Self::texture_size(width, height, mip_levels);
        let mut padded_size = 0;
        for mip_level in 0..mip_levels {
            let mip_width = Self::aligned_width(width >> mip_level);
            let mip_height = height >> mip_level;
            padded_size += Self::texture_size(mip_width, mip_height, 1);
        }

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Read Texture"),
            size: padded_size as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Read Texture"),
        });

        let mut offset = 0;
        for mip_level in 0..mip_levels {
            let mip_width = Self::aligned_width(width >> mip_level);
            let mip_height = height >> mip_level;
            let size = Self::texture_size(mip_width, mip_height, 1);

            encoder.copy_texture_to_buffer(
                wgpu::ImageCopyTexture {
                    texture,
                    mip_level,
                    aspect: wgpu::TextureAspect::All,
                    origin: wgpu::Origin3d::ZERO,
                },
                wgpu::ImageCopyBuffer {
                    buffer: &buffer,
                    layout: wgpu::ImageDataLayout {
                        offset,
                        bytes_per_row: NonZeroU32::new(
                            mip_width * EnvironmentData::PIXEL_SIZE as u32,
                        ),
                        rows_per_image: NonZeroU32::new(mip_height),
                    },
                },
                wgpu::Extent3d {
                    width: mip_width,
                    height: mip_height,
                    depth_or_array_layers: 6,
                },
            );

            offset += size as u64;
        }

        queue.submit(std::iter::once(encoder.finish()));

        buffer.slice(..).map_async(wgpu::MapMode::Read, |_| {});
        device.poll(wgpu::Maintain::Wait);

        let mapped = buffer.slice(..).get_mapped_range();

        let mut data = Vec::with_capacity(size);

        let mut offset = 0;
        for mip_level in 0..mip_levels {
            let mip_width = width >> mip_level;
            let padded_width = Self::aligned_width(mip_width);
            let rows = (height >> mip_level) * 6;
            let row_size = padded_width * EnvironmentData::PIXEL_SIZE as u32;
            let padded_row_size = padded_width * EnvironmentData::PIXEL_SIZE as u32;

            for _ in 0..rows {
                data.extend_from_slice(
                    &mapped[offset as usize..offset as usize + row_size as usize],
                );
                offset += padded_row_size;
            }
        }

        data
    }

    pub fn from_data(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        environment_data: &EnvironmentData,
    ) -> Self {
        let irradiance = Self::create_irradiance(device, queue, environment_data);
        let indirect = Self::create_indirect(device, queue, environment_data);

        let irradiance_view = irradiance.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::Cube),
            ..Default::default()
        });
        let indirect_view = indirect.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::Cube),
            ..Default::default()
        });

        Self {
            irradiance_size: environment_data.irradiance_size,
            irradiance,
            irradiance_view,
            indirect_size: environment_data.indirect_size,
            indirect_mip_levels: environment_data.indirect_mip_levels,
            indirect,
            indirect_view,
        }
    }

    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
    ) -> io::Result<Self> {
        let environment_data = EnvironmentData::load(bytes)?;
        Ok(Self::from_data(device, queue, &environment_data))
    }

    pub fn to_data(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> EnvironmentData {
        let irradiance_data = Self::read_texture(
            device,
            queue,
            &self.irradiance,
            self.irradiance_size,
            self.irradiance_size,
            1,
        );
        let indirect_data = Self::read_texture(
            device,
            queue,
            &self.indirect,
            self.indirect_size,
            self.indirect_size,
            self.indirect_mip_levels,
        );

        EnvironmentData {
            irradiance_size: self.irradiance_size,
            irradiance_data,
            indirect_size: self.indirect_size,
            indirect_mip_levels: self.indirect_mip_levels,
            indirect_data,
        }
    }

    pub fn texture_size(width: u32, height: u32, mip_levels: u32) -> usize {
        let mut size = 0;
        for mip_level in 0..mip_levels {
            let mip_width = width >> mip_level;
            let mip_height = height >> mip_level;
            size += (mip_width * mip_height) as usize * EnvironmentData::PIXEL_SIZE * 6;
        }
        size
    }

    pub fn irradiance_size(&self) -> usize {
        Self::texture_size(self.irradiance_size, self.irradiance_size, 1)
    }

    pub fn indirect_size(&self) -> usize {
        Self::texture_size(
            self.indirect_size,
            self.indirect_size,
            self.indirect_mip_levels,
        )
    }
}

impl BakedEnvironment {
    const WORKGROUP_SIZE: u32 = 16;

    pub fn from_eq(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture: &wgpu::Texture,
        height: u32,
    ) -> Self {
        let specular_size = height * 3 / 4;
        let diffuse_size = height / 4;

        let environemnt_view = texture.create_view(&Default::default());

        let specular_mips = 5;
        let specular_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: specular_size,
                height: specular_size,
                depth_or_array_layers: 6,
            },
            mip_level_count: specular_mips,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
        });

        let diffuse_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Environment Diffuse Texture"),
            size: wgpu::Extent3d {
                width: diffuse_size,
                height: diffuse_size,
                depth_or_array_layers: 6,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
        });
        let diffuse_view = diffuse_texture.create_view(&Default::default());

        let specular_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Uint,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: wgpu::TextureFormat::Rgba16Float,
                            view_dimension: wgpu::TextureViewDimension::D2Array,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let diffuse_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Uint,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: wgpu::TextureFormat::Rgba16Float,
                            view_dimension: wgpu::TextureViewDimension::D2Array,
                        },
                        count: None,
                    },
                ],
            });

        let specular_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&specular_bind_group_layout],
                push_constant_ranges: &[],
            });

        let diffuse_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&diffuse_bind_group_layout],
                push_constant_ranges: &[],
            });

        let specular_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Environment Specular Pipeline"),
            layout: Some(&specular_pipeline_layout),
            module: &device.create_shader_module(wgpu::include_wgsl!("environment.wgsl")),
            entry_point: "specular",
        });

        let diffuse_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Environment Diffuse Pipeline"),
            layout: Some(&diffuse_pipeline_layout),
            module: &device.create_shader_module(wgpu::include_wgsl!("environment.wgsl")),
            entry_point: "irradiance",
        });

        let mut bind_groups = Vec::new();

        for i in 0..specular_mips {
            let specular_view = specular_texture.create_view(&wgpu::TextureViewDescriptor {
                base_mip_level: i,
                mip_level_count: NonZeroU32::new(1),
                ..Default::default()
            });

            let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::bytes_of(&(i as f32 / (specular_mips - 1) as f32)),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &specular_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&environemnt_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&specular_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: buffer.as_entire_binding(),
                    },
                ],
            });

            bind_groups.push(bind_group);
        }

        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &diffuse_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&environemnt_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&diffuse_view),
                },
            ],
        });

        let mut encoder = device.create_command_encoder(&Default::default());

        let mut compute_pass = encoder.begin_compute_pass(&Default::default());

        compute_pass.set_pipeline(&specular_pipeline);

        for i in 0..specular_mips {
            compute_pass.set_bind_group(0, &bind_groups[i as usize], &[]);
            compute_pass.dispatch_workgroups(
                specular_size / Self::WORKGROUP_SIZE + Self::WORKGROUP_SIZE,
                specular_size / Self::WORKGROUP_SIZE + Self::WORKGROUP_SIZE,
                6,
            );
        }

        compute_pass.set_pipeline(&diffuse_pipeline);
        compute_pass.set_bind_group(0, &diffuse_bind_group, &[]);
        compute_pass.dispatch_workgroups(
            diffuse_size / Self::WORKGROUP_SIZE + Self::WORKGROUP_SIZE,
            diffuse_size / Self::WORKGROUP_SIZE + Self::WORKGROUP_SIZE,
            6,
        );

        drop(compute_pass);

        queue.submit(std::iter::once(encoder.finish()));

        let specular_view = specular_texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::Cube),
            ..Default::default()
        });
        let diffuse_view = diffuse_texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::Cube),
            ..Default::default()
        });

        Self {
            irradiance_size: diffuse_size,
            irradiance: diffuse_texture,
            irradiance_view: diffuse_view,
            indirect_size: specular_size,
            indirect_mip_levels: specular_mips,
            indirect: specular_texture,
            indirect_view: specular_view,
        }
    }
}
