@group(0) @binding(4)
var environment_diffuse: texture_cube<f32>;

@group(0) @binding(5)
var environment_specular: texture_cube<f32>;

@group(0) @binding(6)
var integrated_brdf: texture_2d<f32>;

@group(0) @binding(7)
var environment_sampler: sampler;

fn environment(roughness: f32, ndotv: f32, reflect: vec3<f32>, f0: vec3<f32>) -> vec3<f32> {
	let levels = textureNumLevels(environment_specular);
	let lod = roughness * f32(levels - 1);
	let env_prefiltered = textureSampleLevel(environment_specular, environment_sampler, reflect, lod);
	let env_brdf = textureSample(integrated_brdf, environment_sampler, vec2<f32>(ndotv, roughness));
	return env_prefiltered.rgb * (f0 * env_brdf.x + env_brdf.y);
}
