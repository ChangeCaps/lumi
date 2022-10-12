use encase::ShaderType;
use glam::{Vec3, Vec4};

use crate::{
    bind::Bind,
    image::{Image, NormalMap},
    material::Material,
    prelude::ShaderRef,
    shader::ShaderDefs,
};

#[derive(Clone, Debug, Bind)]
#[uniform(RawStandardMaterial = "standard_material")]
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
    pub base_color: Vec4,
    pub alpha_cutoff: f32,
    pub metallic: f32,
    pub roughness: f32,
    pub clearcoat: f32,
    pub clearcoat_roughness: f32,
    pub reflectance: f32,
    pub emissive: Vec3,
    pub emissive_factor: f32,
    pub emissive_exposure_compensation: f32,
    pub thickness: f32,
    pub subsurface_power: f32,
    pub subsurface_color: Vec3,
    pub transmission: f32,
    pub ior: f32,
    pub absorption: Vec3,
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
            emissive_factor: 8.0,
            emissive_exposure_compensation: 0.0,
            thickness: 1.0,
            subsurface_power: 0.0,
            subsurface_color: Vec3::new(0.0, 0.0, 0.0),
            transmission: 0.0,
            ior: 1.5,
            absorption: Vec3::new(0.0, 0.0, 0.0),
        }
    }
}

#[derive(Clone, Copy, ShaderType)]
pub struct RawStandardMaterial {
    pub base_color: Vec4,
    pub alpha_cutoff: f32,
    pub metallic: f32,
    pub roughness: f32,
    pub clearcoat: f32,
    pub clearcoat_roughness: f32,
    pub reflectance: f32,
    pub emissive: Vec3,
    pub emissive_factor: f32,
    pub emissive_exposure_compensation: f32,
    pub thickness: f32,
    pub subsurface_power: f32,
    pub subsurface_color: Vec3,
    pub transmission: f32,
    pub ior: f32,
    pub absorption: Vec3,
}

impl From<&StandardMaterial> for RawStandardMaterial {
    fn from(material: &StandardMaterial) -> Self {
        Self {
            base_color: material.base_color,
            alpha_cutoff: material.alpha_cutoff,
            metallic: material.metallic,
            roughness: material.roughness,
            clearcoat: material.clearcoat,
            clearcoat_roughness: material.clearcoat_roughness,
            reflectance: material.reflectance,
            emissive: material.emissive,
            emissive_factor: material.emissive_factor,
            emissive_exposure_compensation: material.emissive_exposure_compensation,
            thickness: material.thickness,
            subsurface_power: material.subsurface_power,
            subsurface_color: material.subsurface_color,
            transmission: material.transmission,
            ior: material.ior,
            absorption: material.absorption,
        }
    }
}

impl Material for StandardMaterial {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::module("lumi/standard_frag.wgsl")
    }

    fn shader_defs(&self) -> ShaderDefs {
        let mut shader_defs = ShaderDefs::default();

        if self.base_color_texture.is_some() {
            shader_defs.set("BASE_COLOR_TEXTURE");
        }

        if self.metallic_roughness_texture.is_some() {
            shader_defs.set("METALLIC_ROUGHNESS_TEXTURE");
        }

        if self.emissive_map.is_some() {
            shader_defs.set("EMISSIVE_MAP");
        }

        if self.normal_map.is_some() {
            shader_defs.set("NORMAL_MAP");
        }

        if self.clearcoat_normal_map.is_some() {
            shader_defs.set("CLEARCOAT_NORMAL_MAP");
        }

        if self.clearcoat > 0.0 {
            shader_defs.set("CLEARCOAT");
        }

        if self.transmission > 0.0 {
            shader_defs.set("TRANSMISSION");
        }

        if self.transmission > 0.0 {
            shader_defs.set("THICKNESS");
        }

        shader_defs
    }

    fn is_translucent(&self) -> bool {
        self.base_color.w < 1.0 || self.transmission > 0.0
    }

    fn use_ssr(&self) -> bool {
        self.transmission > 0.0
    }
}
