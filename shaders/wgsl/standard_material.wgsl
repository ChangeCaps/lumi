struct StandardMaterial {	
	base_color: vec4<f32>,
	alpha_cutoff: f32,
	metallic: f32,
	roughness: f32,
	clearcoat: f32,
	clearcoat_roughness: f32,
	reflectance: f32,
	emissive: vec3<f32>,
	emissive_factor: f32,
	emissive_exposure_compensation: f32,	
	thickness: f32,
	subsurface_power: f32,
	subsurface_color: vec3<f32>,
	transmission: f32,
	ior: f32,
	absorption: vec3<f32>,
}

@group(1) @binding(0)
var<uniform> standard_material: StandardMaterial;

@group(1) @binding(0)
var base_color_texture: texture_2d<f32>;

@group(1) @binding(0)
var base_color_sampler: sampler;

@group(1) @binding(0)
var metallic_roughness_texture: texture_2d<f32>;

@group(1) @binding(0)
var metallic_roughness_sampler: sampler;

@group(1) @binding(0)
var normal_map: texture_2d<f32>;

@group(1) @binding(0)
var normal_map_sampler: sampler;

@group(1) @binding(0)
var clearcoat_normal_map: texture_2d<f32>;

@group(1) @binding(0)
var clearcoat_normal_map_sampler: sampler;

@group(1) @binding(0)
var emissive_map: texture_2d<f32>;

@group(1) @binding(0)
var emissive_map_sampler: sampler;
