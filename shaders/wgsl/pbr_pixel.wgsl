#include <lumi/integrated_brdf.wgsl>
#include <lumi/pbr_types.wgsl>

struct PbrPixel {
	frag_coord: vec4<f32>,
	thickness: f32,
	subsurface_power: f32,
	subsurface_color: vec3<f32>,
    diffuse_color: vec3<f32>,
    perceptual_roughness: f32,
    roughness: f32,
    f0: vec3<f32>,
	f90: f32,
    dfg: vec3<f32>,
    clearcoat: f32,
    clearcoat_perceptual_roughness: f32,
    clearcoat_roughness: f32,
	transmission: f32,
	eta_ir: f32,
	eta_ri: f32,
	absorption: vec3<f32>,
}

fn has_subsurface(pixel: PbrPixel) -> bool {
	return pixel.thickness < 1.0;
}

fn has_clearcoat(pixel: PbrPixel) -> bool {
	return pixel.clearcoat > 0.0;
}

fn has_refractions(pixel: PbrPixel) -> bool {
	return pixel.transmission > 0.0;
}

fn linear_to_perceptual_roughness(roughness: f32) -> f32 {
	return sqrt(roughness);
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

	pixel.frag_coord = pbr.frag_coord;

	pixel.thickness = pbr.thickness;
	pixel.subsurface_power = pbr.subsurface_power;
	pixel.subsurface_color = pbr.subsurface_color;

	pixel.diffuse_color = compute_diffuse_color(pbr.base_color.rgb, pbr.metallic);

	let min_roughness = 0.089 * 0.089;
	// roughness
	pixel.roughness = clamp(pbr.roughness, min_roughness, 0.99);
	pixel.perceptual_roughness = linear_to_perceptual_roughness(pbr.roughness);

	// clearcoat
	pixel.clearcoat = pbr.clearcoat;
	pixel.clearcoat_roughness = clamp(pbr.clearcoat_roughness, min_roughness, 0.99);
	pixel.clearcoat_perceptual_roughness = linear_to_perceptual_roughness(pbr.clearcoat_roughness);

	pixel.f0 = compute_f0(pbr.base_color.rgb, pbr.metallic, pbr.reflectance);
	pixel.f90 = compute_f90(pixel.f0);

	pixel.dfg = sample_brdf(pixel.perceptual_roughness, geometry.nov);

	pixel.transmission = pbr.transmission;	
	pixel.absorption = pbr.absorption;

	let air_ior = 1.0;
	let ior = max(1.0, pbr.ior);

	pixel.eta_ir = air_ior / ior;
	pixel.eta_ri = ior / air_ior;

	return pixel;
}
