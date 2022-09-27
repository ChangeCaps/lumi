@group(1) @binding(0)
var<uniform> base_color: vec4<f32>;

@group(1) @binding(1)
var<uniform> metallic: f32;

@group(1) @binding(2)
var<uniform> roughness: f32;

@group(1) @binding(3)
var<uniform> reflectance: f32;

@group(1) @binding(4)
var<uniform> emissive: vec3<f32>;

@group(1) @binding(5)
var base_color_texture: texture_2d<f32>;

@group(1) @binding(6)
var base_color_sampler: sampler;

@group(1) @binding(7)
var metallic_roughness_texture: texture_2d<f32>;

@group(1) @binding(8)
var metallic_roughness_sampler: sampler;

@group(1) @binding(9)
var normal_map: texture_2d<f32>;

@group(1) @binding(10)
var normal_map_sampler: sampler;

@group(1) @binding(11)
var emissive_map: texture_2d<f32>;

@group(1) @binding(12)
var emissive_map_sampler: sampler;
