@group(1) @binding(0)
var<uniform> thickness: f32;

@group(1) @binding(0)
var<uniform> subsurface_power: f32;

@group(1) @binding(0)
var<uniform> subsurface_color: vec3<f32>;

@group(1) @binding(0)
var<uniform> base_color: vec4<f32>;

@group(1) @binding(0)
var<uniform> alpha_cutoff: f32;

@group(1) @binding(0)
var<uniform> metallic: f32;

@group(1) @binding(0)
var<uniform> roughness: f32;

@group(1) @binding(0)
var<uniform> reflectance: f32;

@group(1) @binding(0)
var<uniform> clearcoat: f32;

@group(1) @binding(0)
var<uniform> clearcoat_roughness: f32;

@group(1) @binding(0)
var<uniform> emissive: vec3<f32>;

@group(1) @binding(0)
var<uniform> transmission: f32;

@group(1) @binding(0)
var<uniform> ior: f32;

@group(1) @binding(0)
var<uniform> absorption: vec3<f32>;

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
