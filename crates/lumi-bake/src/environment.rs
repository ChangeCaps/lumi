use std::{
    fs,
    io::{self, prelude::*},
    num::NonZeroU32,
    path::Path,
};

use lumi_core::{
    util::DeviceExt, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, BufferBindingType, BufferDescriptor,
    BufferInitDescriptor, BufferUsages, CommandEncoderDescriptor, ComputePipelineDescriptor,
    Device, Extent3d, ImageCopyBuffer, ImageCopyTexture, ImageDataLayout, Maintain, MapMode,
    Origin3d, PipelineLayoutDescriptor, Queue, ShaderStages, SharedDevice, SharedTexture,
    SharedTextureView, StorageTextureAccess, Texture, TextureAspect, TextureDescriptor,
    TextureDimension, TextureFormat, TextureSampleType, TextureUsages, TextureViewDescriptor,
    TextureViewDimension,
};
use lumi_util::bytemuck;
use tracing_log::log;

pub struct EnvironmentData {
    pub irradiance_size: u32,
    pub indirect_size: u32,
    pub indirect_mip_levels: u32,
    pub sky_size: u32,
    pub irradiance_data: Vec<u8>,
    pub indirect_data: Vec<u8>,
    pub sky_data: Vec<u8>,
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
        let indirect_size = read!(source, u32);
        let indirect_mip_levels = read!(source, u32);
        let sky_size = read!(source, u32);

        let irradiance_data_size =
            BakedEnvironment::texture_size(irradiance_size, irradiance_size, 1) * 6;
        let mut irradiance_data = vec![0u8; irradiance_data_size];
        source.read_exact(&mut irradiance_data)?;

        let indirect_data_size =
            BakedEnvironment::texture_size(indirect_size, indirect_size, indirect_mip_levels) * 6;
        let mut indirect_data = vec![0u8; indirect_data_size];
        source.read_exact(&mut indirect_data)?;

        let sky_data_size = BakedEnvironment::texture_size(sky_size, sky_size, 1) * 6;
        let mut sky_data = vec![0u8; sky_data_size];
        source.read_exact(&mut sky_data)?;

        Ok(Self {
            irradiance_size,
            indirect_size,
            indirect_mip_levels,
            sky_size,
            irradiance_data,
            indirect_data,
            sky_data,
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
        write!(dest, self.indirect_size);
        write!(dest, self.indirect_mip_levels);
        write!(dest, self.sky_size);

        dest.write_all(&self.irradiance_data)?;
        dest.write_all(&self.indirect_data)?;
        dest.write_all(&self.sky_data)?;

        Ok(())
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        self.save(&mut bytes).unwrap();
        bytes
    }
}

fn align(value: u32) -> u32 {
    lumi_core::util::align_to(value, 32)
}

pub struct BakedEnvironment {
    pub irradiance_size: u32,
    pub indirect_size: u32,
    pub indirect_mip_levels: u32,
    pub sky_size: u32,
    pub irradiance: SharedTexture,
    pub irradiance_view: SharedTextureView,
    pub indirect: SharedTexture,
    pub indirect_view: SharedTextureView,
    pub sky: SharedTexture,
    pub sky_view: SharedTextureView,
}

impl BakedEnvironment {
    pub fn create_irradiance(
        device: &Device,
        queue: &Queue,
        environment_data: &EnvironmentData,
    ) -> SharedTexture {
        device.create_shared_texture_with_data(
            queue,
            &TextureDescriptor {
                label: Some("Irradiance"),
                size: Extent3d {
                    width: environment_data.irradiance_size,
                    height: environment_data.irradiance_size,
                    depth_or_array_layers: 6,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba16Float,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            },
            &environment_data.irradiance_data,
        )
    }

    pub fn create_indirect(
        device: &Device,
        queue: &Queue,
        environment_data: &EnvironmentData,
    ) -> SharedTexture {
        device.create_shared_texture_with_data(
            queue,
            &TextureDescriptor {
                label: Some("Indirect"),
                size: Extent3d {
                    width: environment_data.indirect_size,
                    height: environment_data.indirect_size,
                    depth_or_array_layers: 6,
                },
                mip_level_count: environment_data.indirect_mip_levels,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba16Float,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            },
            &environment_data.indirect_data,
        )
    }

    pub fn create_sky(
        device: &Device,
        queue: &Queue,
        environment_data: &EnvironmentData,
    ) -> SharedTexture {
        device.create_shared_texture_with_data(
            queue,
            &TextureDescriptor {
                label: Some("Sky"),
                size: Extent3d {
                    width: environment_data.sky_size,
                    height: environment_data.sky_size,
                    depth_or_array_layers: 6,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba16Float,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            },
            &environment_data.sky_data,
        )
    }

    pub fn read_texture(
        device: &Device,
        queue: &Queue,
        texture: &Texture,
        width: u32,
        height: u32,
        mip_levels: u32,
    ) -> Vec<u8> {
        let mut padded_size = 0;
        for mip_level in 0..mip_levels {
            let padded_width = align(width >> mip_level);
            let mip_height = height >> mip_level;
            padded_size += Self::texture_size(padded_width, mip_height, 1) * 6;
        }

        let buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Read Texture"),
            size: padded_size as u64,
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Read Texture"),
        });

        let mut offset = 0;
        for layer in 0..6 {
            for mip in 0..mip_levels {
                let mip_width = width >> mip;
                let padded_width = align(mip_width);
                let mip_height = height >> mip;
                let padded_size = Self::texture_size(padded_width, mip_height, 1);

                let bytes_per_row = padded_width * EnvironmentData::PIXEL_SIZE as u32;

                encoder.copy_texture_to_buffer(
                    ImageCopyTexture {
                        texture,
                        mip_level: mip,
                        origin: Origin3d {
                            x: 0,
                            y: 0,
                            z: layer,
                        },
                        aspect: TextureAspect::All,
                    },
                    ImageCopyBuffer {
                        buffer: &buffer,
                        layout: ImageDataLayout {
                            offset,
                            bytes_per_row: NonZeroU32::new(bytes_per_row),
                            rows_per_image: NonZeroU32::new(mip_height),
                        },
                    },
                    Extent3d {
                        width: mip_width,
                        height: mip_height,
                        depth_or_array_layers: 1,
                    },
                );

                offset += padded_size as u64;
            }
        }

        queue.submit(std::iter::once(encoder.finish()));
        device.poll(Maintain::Wait);

        buffer.slice(..).map_async(MapMode::Read, |_| {});
        device.poll(Maintain::Wait);

        let mapped = buffer.slice(..).get_mapped_range();

        let size = Self::texture_size(width, height, mip_levels) * 6;
        let mut data = Vec::with_capacity(size);

        let mut offset = 0;

        for _ in 0..6 {
            for mip in 0..mip_levels {
                let mip_width = width >> mip;
                let padded_width = align(mip_width);
                let mip_height = height >> mip;
                let row_size = mip_width * EnvironmentData::PIXEL_SIZE as u32;
                let padded_row_size = padded_width * EnvironmentData::PIXEL_SIZE as u32;

                for _ in 0..mip_height {
                    let row = &mapped[offset..offset + row_size as usize];
                    data.extend_from_slice(row);

                    offset += padded_row_size as usize;
                }
            }
        }

        data
    }

    pub fn from_data(device: &Device, queue: &Queue, environment_data: &EnvironmentData) -> Self {
        let irradiance = Self::create_irradiance(device, queue, environment_data);
        let indirect = Self::create_indirect(device, queue, environment_data);
        let sky = Self::create_sky(device, queue, environment_data);

        let irradiance_view = irradiance.create_view(&TextureViewDescriptor {
            dimension: Some(TextureViewDimension::Cube),
            ..Default::default()
        });
        let indirect_view = indirect.create_view(&TextureViewDescriptor {
            dimension: Some(TextureViewDimension::Cube),
            ..Default::default()
        });
        let sky_view = sky.create_view(&TextureViewDescriptor {
            dimension: Some(TextureViewDimension::Cube),
            ..Default::default()
        });

        Self {
            irradiance_size: environment_data.irradiance_size,
            indirect_size: environment_data.indirect_size,
            indirect_mip_levels: environment_data.indirect_mip_levels,
            sky_size: environment_data.sky_size,
            irradiance,
            irradiance_view,
            indirect,
            indirect_view,
            sky,
            sky_view,
        }
    }

    pub fn from_bytes(device: &Device, queue: &Queue, bytes: &[u8]) -> io::Result<Self> {
        let environment_data = EnvironmentData::load(bytes)?;
        Ok(Self::from_data(device, queue, &environment_data))
    }

    pub fn to_data(&self, device: &Device, queue: &Queue) -> EnvironmentData {
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
        let sky_data =
            Self::read_texture(device, queue, &self.sky, self.sky_size, self.sky_size, 1);

        assert_eq!(
            irradiance_data.len(),
            Self::texture_size(self.irradiance_size, self.irradiance_size, 1) * 6
        );
        assert_eq!(
            indirect_data.len(),
            Self::texture_size(
                self.indirect_size,
                self.indirect_size,
                self.indirect_mip_levels
            ) * 6
        );
        assert_eq!(
            sky_data.len(),
            Self::texture_size(self.sky_size, self.sky_size, 1) * 6
        );

        EnvironmentData {
            irradiance_size: self.irradiance_size,
            indirect_size: self.indirect_size,
            indirect_mip_levels: self.indirect_mip_levels,
            sky_size: self.sky_size,
            irradiance_data,
            indirect_data,
            sky_data,
        }
    }

    pub fn write(&self, device: &Device, queue: &Queue, dest: impl Write) -> io::Result<()> {
        let environment_data = self.to_data(device, queue);
        environment_data.save(dest)
    }

    pub fn save(&self, device: &Device, queue: &Queue, path: impl AsRef<Path>) -> io::Result<()> {
        self.write(device, queue, fs::File::create(path)?)
    }

    pub fn texture_size(width: u32, height: u32, mip_levels: u32) -> usize {
        let mut size = 0;
        for mip_level in 0..mip_levels {
            let mip_width = width >> mip_level;
            let mip_height = height >> mip_level;
            size += (mip_width * mip_height) as usize * EnvironmentData::PIXEL_SIZE;
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

    #[cfg(feature = "image")]
    pub fn open_from_eq(
        device: &Device,
        queue: &Queue,
        path: impl AsRef<Path>,
        indirect_size: u32,
        irradiance_size: u32,
        sky_size: u32,
    ) -> image::ImageResult<Self> {
        let eq = image::open(path)?;
        let eq = eq.to_rgba16();
        let bytes = bytemuck::cast_slice(eq.as_raw());

        Ok(Self::from_eq_bytes(
            device,
            queue,
            bytes,
            eq.width(),
            eq.height(),
            indirect_size,
            irradiance_size,
            sky_size,
        ))
    }

    pub fn from_eq_bytes(
        device: &Device,
        queue: &Queue,
        bytes: &[u8],
        width: u32,
        height: u32,
        indirect_size: u32,
        irradiance_size: u32,
        sky_size: u32,
    ) -> Self {
        let texture = device.create_texture_with_data(
            queue,
            &TextureDescriptor {
                label: Some("Environment"),
                size: Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba16Uint,
                usage: TextureUsages::TEXTURE_BINDING,
            },
            &bytes,
        );

        Self::from_eq(
            device,
            queue,
            &texture,
            indirect_size,
            irradiance_size,
            sky_size,
        )
    }

    pub fn from_eq(
        device: &Device,
        queue: &Queue,
        texture: &Texture,
        indirect_size: u32,
        irradiance_size: u32,
        sky_size: u32,
    ) -> Self {
        let environemnt_view = texture.create_view(&Default::default());

        let irradiance_texture = device.create_shared_texture(&TextureDescriptor {
            label: Some("Environment Diffuse Texture"),
            size: Extent3d {
                width: irradiance_size,
                height: irradiance_size,
                depth_or_array_layers: 6,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba16Float,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::STORAGE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::COPY_SRC,
        });
        let irradiance_view = irradiance_texture.create_view(&Default::default());

        let indirect_mips = 30 - indirect_size.leading_zeros();
        let indirect_texture = device.create_shared_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width: indirect_size,
                height: indirect_size,
                depth_or_array_layers: 6,
            },
            mip_level_count: indirect_mips,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba16Float,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::STORAGE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::COPY_SRC,
        });

        let sky_texture = device.create_shared_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width: sky_size,
                height: sky_size,
                depth_or_array_layers: 6,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba16Float,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::STORAGE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::COPY_SRC,
        });
        let sky_view = sky_texture.create_view(&Default::default());

        let indirect_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
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
                    BindGroupLayoutEntry {
                        binding: 3,
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

        let irradiance_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
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

        let sky_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
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

        let irradiance_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&irradiance_bind_group_layout],
            push_constant_ranges: &[],
        });

        let indirect_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&indirect_bind_group_layout],
            push_constant_ranges: &[],
        });

        let sky_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&sky_bind_group_layout],
            push_constant_ranges: &[],
        });

        let irradiance_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Environment Irradiance Pipeline"),
            layout: Some(&irradiance_pipeline_layout),
            module: &device.create_shader_module(lumi_core::include_wgsl!("environment.wgsl")),
            entry_point: "irradiance",
        });

        let indirect_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Environment Indirect Pipeline"),
            layout: Some(&indirect_pipeline_layout),
            module: &device.create_shader_module(lumi_core::include_wgsl!("environment.wgsl")),
            entry_point: "indirect",
        });

        let sky_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Environment Sky Pipeline"),
            layout: Some(&sky_pipeline_layout),
            module: &device.create_shader_module(lumi_core::include_wgsl!("environment.wgsl")),
            entry_point: "eq_to_cube",
        });

        let mut irradiance_bind_groups = Vec::new();

        for i in 0..6u32 {
            let side_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: &i.to_le_bytes(),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            });

            let bind_group = device.create_bind_group(&BindGroupDescriptor {
                label: None,
                layout: &irradiance_bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&environemnt_view),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::TextureView(&irradiance_view),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: side_buffer.as_entire_binding(),
                    },
                ],
            });

            irradiance_bind_groups.push(bind_group);
        }

        let mut indirect_bind_groups = Vec::new();

        for i in 0..indirect_mips {
            let indirect_view = indirect_texture.create_view(&TextureViewDescriptor {
                base_mip_level: i,
                mip_level_count: NonZeroU32::new(1),
                ..Default::default()
            });

            let roughness_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: bytemuck::bytes_of(&(i as f32 / (indirect_mips - 1) as f32)),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            });

            for j in 0..6u32 {
                let side_buffer = device.create_buffer_init(&BufferInitDescriptor {
                    label: None,
                    contents: &j.to_le_bytes(),
                    usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                });

                let bind_group = device.create_bind_group(&BindGroupDescriptor {
                    label: None,
                    layout: &indirect_bind_group_layout,
                    entries: &[
                        BindGroupEntry {
                            binding: 0,
                            resource: BindingResource::TextureView(&environemnt_view),
                        },
                        BindGroupEntry {
                            binding: 1,
                            resource: BindingResource::TextureView(&indirect_view),
                        },
                        BindGroupEntry {
                            binding: 2,
                            resource: side_buffer.as_entire_binding(),
                        },
                        BindGroupEntry {
                            binding: 3,
                            resource: roughness_buffer.as_entire_binding(),
                        },
                    ],
                });

                indirect_bind_groups.push(bind_group);
            }
        }

        let sky_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &sky_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&environemnt_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&sky_view),
                },
            ],
        });

        /* ---------- irradiance ---------- */

        for (i, bind_group) in irradiance_bind_groups.iter().enumerate() {
            let mut encoder = device.create_command_encoder(&Default::default());
            let mut compute_pass = encoder.begin_compute_pass(&Default::default());
            compute_pass.set_pipeline(&irradiance_pipeline);

            compute_pass.set_bind_group(0, bind_group, &[]);
            compute_pass.dispatch_workgroups(
                irradiance_size / Self::WORKGROUP_SIZE + Self::WORKGROUP_SIZE,
                irradiance_size / Self::WORKGROUP_SIZE + Self::WORKGROUP_SIZE,
                1,
            );

            log::trace!(
                "Baked irradiance: {}/{}",
                i + 1,
                irradiance_bind_groups.len()
            );

            drop(compute_pass);
            queue.submit(std::iter::once(encoder.finish()));
            device.poll(Maintain::Wait);
        }

        /* ---------- indirect ---------- */

        for (i, bind_group) in indirect_bind_groups.iter().enumerate() {
            let mut encoder = device.create_command_encoder(&Default::default());
            let mut compute_pass = encoder.begin_compute_pass(&Default::default());
            compute_pass.set_pipeline(&indirect_pipeline);

            compute_pass.set_bind_group(0, bind_group, &[]);
            compute_pass.dispatch_workgroups(
                indirect_size / Self::WORKGROUP_SIZE + Self::WORKGROUP_SIZE,
                indirect_size / Self::WORKGROUP_SIZE + Self::WORKGROUP_SIZE,
                1,
            );

            log::trace!("Baked indirect: {}/{}", i + 1, indirect_bind_groups.len());

            drop(compute_pass);
            queue.submit(std::iter::once(encoder.finish()));
            device.poll(Maintain::Wait);
        }

        /* ---------- sky ---------- */

        let mut encoder = device.create_command_encoder(&Default::default());
        let mut compute_pass = encoder.begin_compute_pass(&Default::default());
        compute_pass.set_pipeline(&sky_pipeline);

        compute_pass.set_bind_group(0, &sky_bind_group, &[]);
        compute_pass.dispatch_workgroups(
            sky_size / Self::WORKGROUP_SIZE + Self::WORKGROUP_SIZE,
            sky_size / Self::WORKGROUP_SIZE + Self::WORKGROUP_SIZE,
            6,
        );

        drop(compute_pass);

        queue.submit(std::iter::once(encoder.finish()));
        device.poll(Maintain::Wait);

        log::trace!("Baked sky: {}/{}", 1, 1);

        let indirect_view = indirect_texture.create_view(&TextureViewDescriptor {
            dimension: Some(TextureViewDimension::Cube),
            ..Default::default()
        });
        let irradiance_view = irradiance_texture.create_view(&TextureViewDescriptor {
            dimension: Some(TextureViewDimension::Cube),
            ..Default::default()
        });
        let sky_view = sky_texture.create_view(&TextureViewDescriptor {
            dimension: Some(TextureViewDimension::Cube),
            ..Default::default()
        });

        Self {
            irradiance_size,
            indirect_size,
            indirect_mip_levels: indirect_mips,
            sky_size,
            irradiance: irradiance_texture,
            irradiance_view,
            indirect: indirect_texture,
            indirect_view,
            sky: sky_texture,
            sky_view,
        }
    }
}
