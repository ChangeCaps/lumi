#include <lumi/integrated_brdf.wgsl>
#include <lumi/pbr_material.wgsl>

struct PbrPixel {
    diffuse_color: vec3<f32>,
    perceptual_roughness: f32,
    roughness: f32,
    f0: vec3<f32>,
	f90: f32,
    dfg: vec3<f32>,
    clearcoat: f32,
    clearcoat_perceptual_roughness: f32,
    clearcoat_roughness: f32,
}

fn perceptual_to_linear(perceptual_roughness: f32) -> f32 {
	return perceptual_roughness * perceptual_roughness;
}

fn compute_f0(base_color: vec3<f32>, metallic: f32, reflectance: f32) -> vec3<f32> {
	let a = 0.16 * reflectance * reflectance * (1.0 - metallic);
	let b = base_color * metallic;
	return a + b;
}

fn compute_f90(f0: vec3<f32>) -> f32 {
	return clamp(dot(f0, vec3<f32>(50.0 * 0.33)), 0.0, 1.0);
}

fn compute_diffuse_color(base_color: vec3<f32>, metallic: f32) -> vec3<f32> {
	return base_color * (1.0 - metallic);
}

fn get_pbr_pixel(pbr: Pbr, geometry: PbrGeometry) -> PbrPixel {
	var pixel: PbrPixel;

	pixel.diffuse_color = compute_diffuse_color(pbr.base_color.rgb, pbr.metallic);

	// roughness
	pixel.perceptual_roughness = clamp(pbr.roughness, 0.089, 1.0);
	pixel.roughness = perceptual_to_linear(pixel.perceptual_roughness);

	// clearcoat
	pixel.clearcoat = pbr.clearcoat;
	pixel.clearcoat_perceptual_roughness = clamp(pbr.clearcoat_roughness, 0.089, 1.0);
	pixel.clearcoat_roughness = perceptual_to_linear(pixel.clearcoat_perceptual_roughness);

	pixel.f0 = compute_f0(pbr.base_color.rgb, pbr.metallic, pbr.reflectance);
	pixel.f90 = compute_f90(pixel.f0);

	pixel.dfg = sample_brdf(pixel.perceptual_roughness, geometry.nov);

	return pixel;
}
