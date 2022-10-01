#include <lumi/pbr_light.wgsl>

@group(0) @binding(4)
var environment_diffuse: texture_cube<f32>;

@group(0) @binding(5)
var environment_specular: texture_cube<f32>;

@group(0) @binding(6)
var integrated_brdf: texture_2d<f32>;

@group(0) @binding(7)
var environment_sampler: sampler;

fn env_specular(perceptual_roughness: f32, ndotv: f32, r: vec3<f32>, f0: vec3<f32>, f90: f32) -> vec3<f32> {
	let levels = textureNumLevels(environment_specular);
	let lod = perceptual_roughness * f32(levels - 1);
	let env_prefiltered = textureSampleLevel(environment_specular, environment_sampler, r, lod);
	let env_brdf = textureSample(integrated_brdf, environment_sampler, vec2<f32>(ndotv, min(perceptual_roughness, 0.99)));
	let specular_color = f0 * env_brdf.x + f90 * env_brdf.y;
	return env_prefiltered.rgb * specular_color;
}

fn env(
	diffuse_color: vec3<f32>,
	metallic: f32, 
	perceptual_roughness: f32, 
	ndotv: f32, 
	normal: vec3<f32>, 
	reflect: vec3<f32>, 
	f0: vec3<f32>,
	f90: f32
) -> vec3<f32> {	
	// diffuse
	let irradiance = textureSample(environment_diffuse, environment_sampler, normal).rgb;
	let diffuse = irradiance * diffuse_color;

	// specular
	let specular = env_specular(perceptual_roughness, ndotv, reflect, f0, f90);

	return diffuse + specular;
}
