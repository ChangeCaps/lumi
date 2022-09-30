#include <lumi/pbr_light.wgsl>

@group(0) @binding(4)
var environment_diffuse: texture_cube<f32>;

@group(0) @binding(5)
var environment_specular: texture_cube<f32>;

@group(0) @binding(6)
var integrated_brdf: texture_2d<f32>;

@group(0) @binding(7)
var environment_sampler: sampler;

fn environment(
	base_color: vec3<f32>,
	metallic: f32, 
	roughness: f32, 
	ndotv: f32, 
	normal: vec3<f32>, 
	reflect: vec3<f32>, 
	f0: vec3<f32>
) -> vec3<f32> {
	let f = f_schlick_roughness3(f0, ndotv, roughness);
	let kd = (1.0 - f) * (1.0 - metallic);
	
	// diffuse
	let irradiance = textureSample(environment_diffuse, environment_sampler, normal).rgb;
	let diffuse = irradiance * base_color;

	// specular
	let levels = textureNumLevels(environment_specular);
	let lod = roughness * f32(levels - 1);
	let env_prefiltered = textureSampleLevel(environment_specular, environment_sampler, reflect, lod);
	let env_brdf = textureSample(integrated_brdf, environment_sampler, vec2<f32>(ndotv, min(roughness, 0.99)));
	let specular = env_prefiltered.rgb * (f * env_brdf.x + env_brdf.y);

	return kd * diffuse + specular;
}
