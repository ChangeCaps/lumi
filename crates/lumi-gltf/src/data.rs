use std::path::Path;

use lumi_core::{FilterMode, Image, ImageData, TextureFormat};
use lumi_material::{MeshNode, Primitive, StandardMaterial};
use lumi_mesh::Mesh;
use lumi_util::math::Mat4;

fn wrapping_to_address(mode: gltf::texture::WrappingMode) -> lumi_core::AddressMode {
    match mode {
        gltf::texture::WrappingMode::ClampToEdge => lumi_core::AddressMode::ClampToEdge,
        gltf::texture::WrappingMode::MirroredRepeat => lumi_core::AddressMode::MirrorRepeat,
        gltf::texture::WrappingMode::Repeat => lumi_core::AddressMode::Repeat,
    }
}

pub struct GltfData {
    pub document: gltf::Document,
    pub textures: Vec<Image>,
    pub materials: Vec<StandardMaterial>,
    pub meshes: Vec<MeshNode>,
}

impl GltfData {
    pub fn new(
        document: gltf::Document,
        buffer_data: &[gltf::buffer::Data],
        image_data: &[gltf::image::Data],
    ) -> Self {
        let mut this = Self {
            document,
            textures: Vec::new(),
            materials: Vec::new(),
            meshes: Vec::new(),
        };

        for texture in this.document.textures() {
            let texture = Self::load_texture(texture, image_data);
            this.textures.push(texture);
        }

        for material in this.document.materials() {
            let material = Self::load_material(material, &this.textures);
            this.materials.push(material);
        }

        for mesh in this.document.meshes() {
            let mesh = this.load_mesh(mesh, buffer_data);
            this.meshes.push(mesh);
        }

        this
    }

    pub fn open(path: impl AsRef<Path>) -> Result<Self, gltf::Error> {
        let (document, buffers, images) = gltf::import(path)?;

        Ok(Self::new(document, &buffers, &images))
    }

    pub fn create_mesh_node(&self) -> MeshNode {
        let mut mesh_node = MeshNode::default();

        if let Some(scene) = self.document.default_scene() {
            for node in scene.nodes() {
                self.append_mesh_node(&mut mesh_node, node, Mat4::IDENTITY);
            }
        }

        mesh_node
    }

    fn append_mesh_node(&self, mesh_node: &mut MeshNode, node: gltf::Node, global_transform: Mat4) {
        let transform = global_transform * Mat4::from_cols_array_2d(&node.transform().matrix());

        if let Some(mesh) = node.mesh() {
            let mesh = &self.meshes[mesh.index()];
            let transform = transform * mesh.transform;

            for primitive in mesh.primitives.iter() {
                let mut mesh = primitive.mesh.clone();
                mesh.transform(transform);

                mesh_node.add_primitive(primitive.material.clone(), mesh);
            }
        }

        for child in node.children() {
            self.append_mesh_node(mesh_node, child, transform);
        }
    }

    fn load_texture(texture: gltf::Texture, data: &[gltf::image::Data]) -> Image {
        let image = texture.source();
        let data = &data[image.index()];
        let mut pixels = data.pixels.clone();

        match data.format {
            gltf::image::Format::R8G8B8 => {
                pixels = pixels
                    .chunks_exact(3)
                    .flat_map(|c| [c[0], c[1], c[2], 255])
                    .collect();
            }
            gltf::image::Format::B8G8R8 => {
                pixels = pixels
                    .chunks_exact(3)
                    .flat_map(|c| [c[0], c[1], c[2], 255])
                    .collect();
            }
            gltf::image::Format::R16G16B16 => {
                pixels = pixels
                    .chunks_exact(6)
                    .flat_map(|c| [c[0], c[1], c[2], c[3], c[4], c[5], 0, 255])
                    .collect();
            }
            _ => {}
        }

        let format = match data.format {
            gltf::image::Format::R8 => TextureFormat::R8Unorm,
            gltf::image::Format::R8G8 => TextureFormat::Rg8Unorm,
            gltf::image::Format::R8G8B8 => TextureFormat::Rgba8Unorm,
            gltf::image::Format::R8G8B8A8 => TextureFormat::Rgba8Unorm,
            gltf::image::Format::B8G8R8 => TextureFormat::Bgra8Unorm,
            gltf::image::Format::B8G8R8A8 => TextureFormat::Bgra8Unorm,
            gltf::image::Format::R16 => TextureFormat::R16Float,
            gltf::image::Format::R16G16 => TextureFormat::Rg16Float,
            gltf::image::Format::R16G16B16 => TextureFormat::Rgba16Float,
            gltf::image::Format::R16G16B16A16 => TextureFormat::Rgba16Float,
        };

        let mut image = Image::new(ImageData::with_format(
            data.width,
            data.height,
            pixels,
            format,
        ));

        image.sampler.address_mode_u = wrapping_to_address(texture.sampler().wrap_s());
        image.sampler.address_mode_v = wrapping_to_address(texture.sampler().wrap_t());

        if texture.sampler().mag_filter() == Some(gltf::texture::MagFilter::Nearest) {
            image.sampler.mag_filter = FilterMode::Nearest;
        }
        if texture.sampler().min_filter() == Some(gltf::texture::MinFilter::Nearest) {
            image.sampler.min_filter = FilterMode::Nearest;
        }

        image.into()
    }

    fn load_material(material: gltf::Material, textures: &[Image]) -> StandardMaterial {
        let mut standard = StandardMaterial::default();

        let pbr = material.pbr_metallic_roughness();

        if let Some(base_color) = pbr.base_color_texture() {
            let image = textures[base_color.texture().index()].clone();
            standard.base_color_texture = Some(image);
        }

        if let Some(metallic_roughness) = pbr.metallic_roughness_texture() {
            let image = textures[metallic_roughness.texture().index()].clone();
            standard.metallic_roughness_texture = Some(image);
        }

        if let Some(normal) = material.normal_texture() {
            let image = textures[normal.texture().index()].clone();
            standard.normal_map = Some(image);
        }

        if let Some(emissive) = material.emissive_texture() {
            let image = textures[emissive.texture().index()].clone();
            standard.emissive_map = Some(image);
        }

        if let Some(transmission) = material.transmission() {
            standard.transmission = transmission.transmission_factor();
        }

        standard.base_color = pbr.base_color_factor().into();
        standard.alpha_cutoff = material.alpha_cutoff().unwrap_or(-1.0);
        standard.metallic = pbr.metallic_factor();
        standard.roughness = pbr.roughness_factor();
        standard.emissive = material.emissive_factor().into();

        standard
    }

    fn load_mesh(&self, mesh: gltf::Mesh, data: &[gltf::buffer::Data]) -> MeshNode {
        let mut mesh_node = MeshNode::default();

        for primitive in mesh.primitives() {
            let primitive = self.load_primitive(primitive, data);
            mesh_node.primitives.push(primitive);
        }

        mesh_node
    }

    fn load_primitive(&self, primitive: gltf::Primitive, data: &[gltf::buffer::Data]) -> Primitive {
        let reader = primitive.reader(|buffer| Some(&data[buffer.index()]));

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
            self.materials[index].clone()
        } else {
            Default::default()
        };

        let primitive = Primitive { mesh, material };

        primitive
    }
}
