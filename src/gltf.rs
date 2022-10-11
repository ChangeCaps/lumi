use std::{io::Read, path::Path};

use glam::Mat4;
use wgpu::{FilterMode, TextureFormat};

use crate::{
    image::{Image, ImageData},
    material::{Primitive, StandardMaterial},
    mesh::Mesh,
    prelude::{MeshNode, NormalMap},
};

impl MeshNode {
    pub fn open_gltf(path: impl AsRef<Path>) -> gltf::Result<Self> {
        println!("Loading glTF file: {}", path.as_ref().display());
        let (document, buffers, images) = gltf::import(path)?;
        Ok(Self::gltf_inner(document, buffers, images))
    }

    pub fn load_gltf(mut read: impl Read) -> gltf::Result<Self> {
        let mut data = Vec::new();
        read.read_to_end(&mut data)?;
        let (document, buffers, images) = gltf::import_slice(&data)?;
        Ok(Self::gltf_inner(document, buffers, images))
    }

    fn gltf_inner(
        document: gltf::Document,
        buffers: Vec<gltf::buffer::Data>,
        images: Vec<gltf::image::Data>,
    ) -> Self {
        let data = GltfData::new(&document, buffers, images);

        let mut this = Self::default();
        let mut nodes = document
            .default_scene()
            .unwrap()
            .nodes()
            .map(|node| (Mat4::IDENTITY, node))
            .collect::<Vec<_>>();

        while !nodes.is_empty() {
            for (global, node) in std::mem::take(&mut nodes) {
                if let Some(mesh) = node.mesh() {
                    let mut primitives = data.meshes[mesh.index()].clone();

                    let transform: Mat4 = Mat4::from_cols_array_2d(&node.transform().matrix());
                    let global_transform = global * transform;

                    if transform != Mat4::IDENTITY {
                        for primitive in &mut primitives {
                            primitive.mesh.transform(global_transform);
                        }
                    }

                    this.primitives.extend(primitives);

                    nodes.extend(node.children().map(|node| (global_transform, node)));
                }
            }
        }

        this
    }
}

#[allow(dead_code)]
struct GltfData {
    images: Vec<Image>,
    materials: Vec<StandardMaterial>,
    meshes: Vec<Vec<Primitive>>,
}

impl GltfData {
    fn new(
        document: &gltf::Document,
        buffers: Vec<gltf::buffer::Data>,
        image_data: Vec<gltf::image::Data>,
    ) -> Self {
        let mut images = Vec::<Image>::with_capacity(document.images().len());
        for texture in document.textures() {
            let image = &image_data[texture.source().index()];
            let mut pixels = image.pixels.clone();

            let format = match image.format {
                gltf::image::Format::R8 => TextureFormat::R8Unorm,
                gltf::image::Format::R8G8 => TextureFormat::Rg8Unorm,
                gltf::image::Format::R8G8B8 => {
                    pixels = pixels
                        .chunks(3)
                        .map(|c| [c[0], c[1], c[2], 255])
                        .flatten()
                        .collect();

                    TextureFormat::Rgba8Unorm
                }
                gltf::image::Format::R8G8B8A8 => TextureFormat::Rgba8Unorm,
                gltf::image::Format::B8G8R8 => {
                    pixels = pixels
                        .chunks(3)
                        .map(|c| [c[2], c[1], c[0], 255])
                        .flatten()
                        .collect();

                    TextureFormat::Bgra8Unorm
                }
                gltf::image::Format::B8G8R8A8 => TextureFormat::Bgra8Unorm,
                gltf::image::Format::R16 => TextureFormat::R16Float,
                gltf::image::Format::R16G16 => TextureFormat::Rg16Float,
                gltf::image::Format::R16G16B16 => unimplemented!(),
                gltf::image::Format::R16G16B16A16 => TextureFormat::Rgba16Float,
            };

            let mut image_data = ImageData::with_format(image.width, image.height, pixels, format);

            fn convert_wrap(wrap: gltf::texture::WrappingMode) -> wgpu::AddressMode {
                match wrap {
                    gltf::texture::WrappingMode::ClampToEdge => wgpu::AddressMode::ClampToEdge,
                    gltf::texture::WrappingMode::MirroredRepeat => wgpu::AddressMode::MirrorRepeat,
                    gltf::texture::WrappingMode::Repeat => wgpu::AddressMode::Repeat,
                }
            }

            image_data.sampler.address_mode_u = convert_wrap(texture.sampler().wrap_s());
            image_data.sampler.address_mode_v = convert_wrap(texture.sampler().wrap_t());

            if texture.sampler().mag_filter() == Some(gltf::texture::MagFilter::Nearest) {
                image_data.sampler.mag_filter = FilterMode::Nearest;
            }

            if texture.sampler().min_filter() == Some(gltf::texture::MinFilter::Nearest) {
                image_data.sampler.min_filter = FilterMode::Nearest;
            }

            let texture = Image::new(image_data);
            images.push(texture);
        }

        let mut materials = Vec::with_capacity(document.materials().len());
        for material in document.materials() {
            let pbr = material.pbr_metallic_roughness();

            let base_color_texture = if let Some(image) = pbr.base_color_texture() {
                let mut image = images[image.texture().index()].clone();
                image.format = TextureFormat::Rgba8UnormSrgb;
                Some(image)
            } else {
                None
            };

            let metallic_roughness_texture = if let Some(image) = pbr.metallic_roughness_texture() {
                let mut image = images[image.texture().index()].clone();
                image.format = TextureFormat::Rgba8Unorm;
                Some(image)
            } else {
                None
            };

            let normal_map = if let Some(image) = material.normal_texture() {
                Some(NormalMap::new(images[image.texture().index()].clone()))
            } else {
                None
            };

            let emissive_map = if let Some(image) = material.emissive_texture() {
                Some(images[image.texture().index()].clone())
            } else {
                None
            };

            let mut standard_material = StandardMaterial {
                base_color_texture,
                alpha_cutoff: material.alpha_cutoff().unwrap_or(-1.0),
                metallic_roughness_texture,
                normal_map,
                emissive_map,
                base_color: pbr.base_color_factor().into(),
                metallic: pbr.metallic_factor(),
                roughness: pbr.roughness_factor(),
                emissive: material.emissive_factor().into(),
                ..Default::default()
            };

            if let Some(transmission) = material.transmission() {
                standard_material.transmission = transmission.transmission_factor();
            }

            materials.push(standard_material);
        }

        let mut meshes = Vec::with_capacity(document.meshes().len());
        for mesh in document.meshes() {
            let mut primitives = Vec::with_capacity(mesh.primitives().len());
            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

                let mut mesh = Mesh::new();

                mesh.insert_attribute(
                    Mesh::POSITION,
                    reader.read_positions().unwrap().collect::<Vec<_>>(),
                );

                if let Some(uvs) = reader.read_tex_coords(0) {
                    mesh.insert_attribute(Mesh::UV_0, uvs.into_f32().collect::<Vec<_>>());
                } else {
                    let len = mesh.positions().unwrap().len();
                    mesh.insert_attribute(Mesh::UV_0, vec![[0.0, 0.0]; len]);
                }

                if let Some(normals) = reader.read_normals() {
                    mesh.insert_attribute(Mesh::NORMAL, normals.collect::<Vec<_>>());
                }
                if let Some(tangents) = reader.read_tangents() {
                    mesh.insert_attribute(Mesh::TANGENT, tangents.collect::<Vec<_>>());
                }

                if let Some(indices) = reader.read_indices() {
                    mesh.insert_indices(indices.into_u32().collect::<Vec<_>>());
                }

                let material = if let Some(index) = primitive.material().index() {
                    materials[index].clone()
                } else {
                    Default::default()
                };

                let primitive = Primitive { mesh, material };

                primitives.push(primitive);
            }

            meshes.push(primitives);
        }

        Self {
            images,
            materials,
            meshes,
        }
    }
}
