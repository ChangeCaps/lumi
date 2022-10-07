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

fn prefiltered_radiance_offset(n: vec3<f32>, roughness: f32, offset: f32) -> vec3<f32> {	
	let levels = f32(textureNumLevels(environment_specular) - 1);
	let lod = (roughness + offset) / levels;
	let radiance = textureSampleLevel(environment_specular, environment_sampler, n, lod).rgb;
	return radiance;
}

fn prefiltered_radiance(n: vec3<f32>, roughness: f32) -> vec3<f32> {	
	let levels = f32(textureNumLevels(environment_specular) - 1);
	let lod = roughness * levels;
	let radiance = textureSampleLevel(environment_specular, environment_sampler, n, lod).rgb;
	return radiance;
}

fn env_indirect(perceptual_roughness: f32, r: vec3<f32>) -> vec3<f32> {
	let levels = textureNumLevels(environment_specular);
	let lod = perceptual_roughness * f32(levels - 1);
	return textureSampleLevel(environment_specular, environment_sampler, r, lod).rgb;
}

fn env_subsurface(
	pixel: PbrPixel, 
	geometry: PbrGeometry,
	irradiance: vec3<f32>,
) -> vec3<f32> {
	let view_dependent = prefiltered_radiance_offset(-geometry.v, pixel.roughness, (1.0 + pixel.thickness) / 5.0);
	let attenuation = (1.0 - pixel.thickness) / (2.0 * PI);

	return pixel.subsurface_color * (irradiance + view_dependent) * attenuation;
}

struct Refraction {
	position: vec3<f32>,
	direction: vec3<f32>,
	d: f32,
}

fn refract(r: vec3<f32>, n: vec3<f32>, ior: f32) -> vec3<f32> {
	let k = 1.0 - ior * ior * (1.0 - dot(n, r) * dot(n, r));

	if k < 0.0 {
		return vec3<f32>(0.0);
	} else {
		return ior * r - (ior * dot(n, r) + sqrt(k)) * n;
	}
}

fn refraction_solid_sphere(
	pixel: PbrPixel,
	geometry: PbrGeometry,
	r: vec3<f32>,
) -> Refraction {
	var ray: Refraction;

	let r = refract(r, geometry.n, pixel.eta_ir);	
	let nor = dot(geometry.n, r);
	let d = pixel.thickness * -nor;
	ray.position = geometry.position + r * d;
	ray.d = d;
	let n = normalize(nor * r - geometry.n * 0.5);
	ray.direction = refract(r, n, pixel.eta_ri);

	return ray;
}

fn env_refractions(
	pixel: PbrPixel, 
	geometry: PbrGeometry,
	e: vec3<f32>,
) -> vec3<f32> {
	let ray = refraction_solid_sphere(pixel, geometry, -geometry.v);
	let t = min(vec3<f32>(1.0), exp(-pixel.absorption * ray.d));

	let perceptual_roughness = mix(
		pixel.perceptual_roughness,
		0.0,
		saturate(pixel.eta_ir * 3.0 - 2.0),
	);

	var ft = prefiltered_radiance(ray.direction, perceptual_roughness);
	ft *= pixel.diffuse_color;
	ft *= 1.0 - e;
	ft *= t;

	return ft;
}

fn environment(
	pixel: PbrPixel,
	geometry: PbrGeometry,
) -> vec3<f32> {	
	let e = pixel.f0 * pixel.dfg.x + pixel.f0 * pixel.dfg.y;

	let diffuse_irradiance = env_diffuse(pixel.diffuse_color, geometry.n);
	var diffuse = diffuse_irradiance;
	let r = mix(geometry.r, geometry.n, pixel.roughness * pixel.roughness);
	var specular = env_indirect(pixel.roughness, r);

	diffuse *= 1.0 - e;
	specular *= e;

	if has_subsurface(pixel) {
		diffuse += env_subsurface(pixel, geometry, diffuse_irradiance);
	}

	if has_clearcoat(pixel) {
		let fc = f_schlick(0.04, 1.0, geometry.clearcoat_nov) * pixel.clearcoat;
		let attenuation = 1.0 - fc;

		diffuse *= attenuation;
		specular *= attenuation;

		specular += env_indirect(pixel.clearcoat_perceptual_roughness, geometry.clearcoat_r) * fc;
	}

	diffuse *= ambient_light.color * camera.exposure;
	specular *= ambient_light.color * camera.exposure;

	if has_refractions(pixel) {
		let ft = env_refractions(pixel, geometry, e) * pixel.transmission;
		diffuse *= (1.0 - pixel.transmission);
		specular += ft;
	}
	

	return diffuse + specular;
}
