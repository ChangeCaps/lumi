use std::{fs, io::Read, path::Path};

use glam::Mat4;

use crate::{
    image::{Image, ImageData},
    material::Primitive,
    mesh::Mesh,
    pbr::StandardMaterial,
    prelude::{MeshNode, NormalMap},
};

impl MeshNode {
    pub fn open_gltf(path: impl AsRef<Path>) -> gltf::Result<Self> {
        Self::load_gltf(fs::File::open(path)?)
    }

    pub fn load_gltf(mut read: impl Read) -> gltf::Result<Self> {
        let mut data = Vec::new();
        read.read_to_end(&mut data)?;
        let (document, buffers, images) = gltf::import_slice(&data)?;
        let data = GltfData::new(&document, buffers, images);

        let mut this = Self::default();

        for node in document.nodes() {
            if let Some(mesh) = node.mesh() {
                let mut primitives = data.meshes[mesh.index()].clone();

                let transform: Mat4 = Mat4::from_cols_array_2d(&node.transform().matrix());

                if transform != Mat4::IDENTITY {
                    for primitive in &mut primitives {
                        primitive.mesh.transform(transform);
                    }
                }

                this.primitives.extend(primitives);
            }
        }

        Ok(this)
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
        mut image_data: Vec<gltf::image::Data>,
    ) -> Self {
        let mut images = Vec::<Image>::with_capacity(document.images().len());
        for image in document.images() {
            let data = image_data.remove(image.index());
            let image_data = ImageData::new(data.width, data.height, data.pixels);
            let image = Image::new(image_data);
            images.push(image);
        }

        let mut materials = Vec::with_capacity(document.materials().len());
        for material in document.materials() {
            let pbr = material.pbr_metallic_roughness();

            let base_color_texture = if let Some(image) = pbr.base_color_texture() {
                Some(images[image.texture().source().index()].clone())
            } else {
                None
            };

            let metallic_roughness_texture = if let Some(image) = pbr.metallic_roughness_texture() {
                Some(images[image.texture().source().index()].clone())
            } else {
                None
            };

            let normal_map = if let Some(image) = material.normal_texture() {
                Some(NormalMap::new(
                    images[image.texture().source().index()].clone(),
                ))
            } else {
                None
            };

            let emissive_map = if let Some(image) = material.emissive_texture() {
                Some(images[image.texture().source().index()].clone())
            } else {
                None
            };

            let material = StandardMaterial {
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

            materials.push(material);
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
                mesh.insert_attribute(
                    Mesh::UV_0,
                    reader
                        .read_tex_coords(0)
                        .unwrap()
                        .into_f32()
                        .collect::<Vec<_>>(),
                );

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
