#include <lumi/pbr_light.wgsl>
#include <lumi/pbr_types.wgsl>
#include <lumi/pbr_pixel.wgsl>
#include <lumi/camera.wgsl>

@group(0) @binding(4)
var environment_diffuse: texture_cube<f32>;

@group(0) @binding(5)
var environment_specular: texture_cube<f32>;

@group(0) @binding(6)
var environment_sampler: sampler;

fn env_diffuse(diffuse_color: vec3<f32>, n: vec3<f32>) -> vec3<f32> {
	let irradiance = textureSample(environment_diffuse, environment_sampler, n).rgb;
	let diffuse = irradiance * diffuse_color;

	return diffuse;
}

fn env_indirect(perceptual_roughness: f32, r: vec3<f32>) -> vec3<f32> {
	let levels = textureNumLevels(environment_specular);
	let lod = perceptual_roughness * f32(levels - 1);
	return textureSampleLevel(environment_specular, environment_sampler, r, lod).rgb;
}

fn environment(
	pixel: PbrPixel,
	geometry: PbrGeometry,
) -> vec3<f32> {	
	let e = pixel.f0 * pixel.dfg.x + pixel.f0 * pixel.dfg.y;

	var diffuse = env_diffuse(pixel.diffuse_color, geometry.n);
	let r = mix(geometry.r, geometry.n, pixel.roughness * pixel.roughness);
	var specular = env_indirect(pixel.roughness, r);

	diffuse *= 1.0 - e;
	specular *= e;

	if pixel.clearcoat > 0.0 {
		let fc = f_schlick(0.04, 1.0, geometry.clearcoat_nov) * pixel.clearcoat;
		let attenuation = 1.0 - fc;

		diffuse *= attenuation;
		specular *= attenuation;

		specular += env_indirect(pixel.clearcoat_perceptual_roughness, geometry.clearcoat_r) * fc;
	}

	diffuse *= ambient_light.color * camera.exposure;
	specular *= ambient_light.color * camera.exposure;

	return diffuse + specular;
}