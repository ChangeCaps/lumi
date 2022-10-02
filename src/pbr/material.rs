use glam::{Vec3, Vec4};

use crate::{
    bind::Bind,
    image::{Image, NormalMap},
    material::Material,
    prelude::ShaderRef,
};

#[derive(Clone, Debug, Bind)]
pub struct StandardMaterial {
    #[texture]
    #[sampler(name = "base_color_sampler")]
    pub base_color_texture: Option<Image>,
    #[texture]
    #[sampler(name = "metallic_roughness_sampler")]
    pub metallic_roughness_texture: Option<Image>,
    #[texture]
    #[sampler(name = "normal_map_sampler")]
    pub normal_map: Option<NormalMap>,
    #[texture]
    #[sampler(name = "clearcoat_normal_map_sampler")]
    pub clearcoat_normal_map: Option<NormalMap>,
    #[texture]
    #[sampler(name = "emissive_map_sampler")]
    pub emissive_map: Option<Image>,
    #[uniform]
    pub base_color: Vec4,
    #[uniform]
    pub alpha_cutoff: f32,
    #[uniform]
    pub metallic: f32,
    #[uniform]
    pub roughness: f32,
    #[uniform]
    pub clearcoat: f32,
    #[uniform]
    pub clearcoat_roughness: f32,
    #[uniform]
    pub reflectance: f32,
    #[uniform]
    pub emissive: Vec3,
}

impl Default for StandardMaterial {
    fn default() -> Self {
        Self {
            base_color_texture: None,
            metallic_roughness_texture: None,
            normal_map: None,
            clearcoat_normal_map: None,
            emissive_map: None,
            base_color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            alpha_cutoff: 0.01,
            metallic: 0.01,
            roughness: 0.089,
            clearcoat: 0.0,
            clearcoat_roughness: 0.0,
            reflectance: 0.5,
            emissive: Vec3::ZERO,
        }
    }
}

impl Material for StandardMaterial {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::module("lumi/standard_frag.wgsl")
    }
}
