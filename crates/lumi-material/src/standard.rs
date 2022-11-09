use lumi_bind::Bind;
use lumi_core::Image;
use lumi_macro::ShaderType;
use lumi_shader::{ShaderDefs, ShaderRef};
use lumi_util::math::{Vec3, Vec4};
use shiv::{storage::DenseStorage, world::Component};

use crate::Material;

#[derive(Clone, Debug, PartialEq, Bind)]
#[uniform(RawStandardMaterial = "standard_material")]
pub struct StandardMaterial<T = Image> {
    #[texture]
    #[sampler(name = "base_color_sampler")]
    pub base_color_texture: Option<T>,
    #[texture]
    #[sampler(name = "metallic_roughness_sampler")]
    pub metallic_roughness_texture: Option<T>,
    #[texture]
    #[sampler(name = "normal_map_sampler")]
    pub normal_map: Option<T>,
    #[texture]
    #[sampler(name = "clearcoat_normal_map_sampler")]
    pub clearcoat_normal_map: Option<T>,
    #[texture]
    #[sampler(name = "emissive_map_sampler")]
    pub emissive_map: Option<T>,
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
    pub subsurface: bool,
    pub subsurface_power: f32,
    pub subsurface_color: Vec3,
    pub transmission: f32,
    pub ior: f32,
    pub absorption: Vec3,
}

impl Default for StandardMaterial {
    #[inline]
    fn default() -> Self {
        Self {
            base_color_texture: None,
            metallic_roughness_texture: None,
            normal_map: None,
            clearcoat_normal_map: None,
            emissive_map: None,
            base_color: Vec4::ONE,
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
            subsurface: false,
            subsurface_power: 0.0,
            subsurface_color: Vec3::ONE,
            transmission: 0.0,
            ior: 1.5,
            absorption: Vec3::ZERO,
        }
    }
}

impl<T: Send + Sync + 'static> Component for StandardMaterial<T> {
    type Storage = DenseStorage;
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
    #[inline]
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
    #[inline]
    fn fragment_shader() -> ShaderRef {
        ShaderRef::module("lumi/standard_frag.wgsl")
    }

    #[inline]
    fn shader_defs(&self) -> ShaderDefs {
        let mut shader_defs = ShaderDefs::default();

        if self.base_color_texture.is_some() {
            shader_defs.push("BASE_COLOR_TEXTURE");
        }

        if self.metallic_roughness_texture.is_some() {
            shader_defs.push("METALLIC_ROUGHNESS_TEXTURE");
        }

        if self.emissive_map.is_some() {
            shader_defs.push("EMISSIVE_MAP");
        }

        if self.normal_map.is_some() {
            shader_defs.push("NORMAL_MAP");
        }

        if self.clearcoat_normal_map.is_some() {
            shader_defs.push("CLEARCOAT_NORMAL_MAP");
        }

        if self.clearcoat > 0.0 {
            shader_defs.push("CLEARCOAT");
        }

        if self.subsurface {
            shader_defs.push("SUBSURFACE");
        }

        if self.transmission > 0.0 {
            shader_defs.push("TRANSMISSION");
        }

        if self.transmission > 0.0 || self.subsurface {
            shader_defs.push("THICKNESS");
        }

        shader_defs
    }

    #[inline]
    fn is_translucent(&self) -> bool {
        self.base_color.w < 1.0 || self.transmission > 0.0
    }
}
