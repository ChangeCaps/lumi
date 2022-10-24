#include <lumi/camera.wgsl>
#include <lumi/mesh.wgsl>
#include <lumi/integrated_brdf.wgsl>

struct Pbr {
	frag_coord: vec4<f32>,
	w_position: vec3<f32>,	
	normal: vec3<f32>,
	view: vec3<f32>,
	base_color: vec4<f32>,
	alpha_cutoff: f32,
	metallic: f32,
	roughness: f32,	
	reflectance: f32,
	emissive: vec3<f32>,
	emissive_factor: f32,
	emissive_exposure_compensation: f32,

#ifdef CLEARCOAT
	clearcoat: f32,
	clearcoat_roughness: f32,
	clearcoat_normal: vec3<f32>,
#endif

#ifdef THICKNESS
	thickness: f32,
#endif

#ifdef TRANSMISSION
	transmission: f32,
	ior: f32,
	absorption: vec3<f32>,
#endif

#ifdef SUBSURFACE
	subsurface_power: f32,
	subsurface_color: vec3<f32>,
#endif
}

fn default_pbr(mesh: Mesh) -> Pbr {
	var out: Pbr;

	out.frag_coord = mesh.v_frag_coord;
	out.w_position = mesh.w_position;
	out.normal = mesh.w_normal;
	out.view = normalize(camera.position - mesh.w_position);
	out.base_color = vec4<f32>(1.0);
	out.alpha_cutoff = 0.01;
	out.metallic = 0.01;
	out.roughness = 0.089;	
	out.reflectance = 0.5;
	out.emissive = vec3<f32>(0.0);
	out.emissive_factor = 8.0;
	out.emissive_exposure_compensation = 0.0;

#ifdef CLEARCOAT
	out.clearcoat = 0.0;
	out.clearcoat_roughness = 0.089;
	out.clearcoat_normal = mesh.w_normal;
#endif

#ifdef TRANSMISSION
	out.thickness = 1.0;
	out.transmission = 1.0;
	out.ior = 1.5;
	out.absorption = vec3<f32>(0.0);
#endif

#ifdef SUBSURFACE
	out.subsurface_power = 0.0;
	out.subsurface_color = vec3<f32>(1.0);
#endif

	return out;
}

struct PbrPixel {
	frag_coord: vec4<f32>,
	position: vec3<f32>,

	n: vec3<f32>,
	v: vec3<f32>,
	r: vec3<f32>,
	nov: f32,
	nor: f32,

#ifdef CLEARCOAT
	clearcoat_n: vec3<f32>,
	clearcoat_r: vec3<f32>,
	clearcoat_nov: f32,
	clearcoat_nor: f32,
#endif

    diffuse_color: vec3<f32>,

    perceptual_roughness: f32,
    roughness: f32,

    f0: vec3<f32>,
	f90: f32,
    dfg: vec3<f32>,

#ifdef CLEARCOAT
    clearcoat: f32,
    clearcoat_perceptual_roughness: f32,
    clearcoat_roughness: f32,
#endif

#ifdef THICKNESS
	thickness: f32,
#endif

#ifdef TRANSMISSION
	transmission: f32,
	eta_ir: f32,
	eta_ri: f32,
	absorption: vec3<f32>,
#endif

#ifdef SUBSURFACE
	subsurface_power: f32,
	subsurface_color: vec3<f32>,
#endif
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

fn get_pbr_pixel(in: Pbr) -> PbrPixel {
	var pixel: PbrPixel;

	pixel.frag_coord = in.frag_coord;
	pixel.position = in.w_position;

	pixel.n = normalize(in.normal);
	pixel.v = normalize(in.view);
	pixel.r = reflect(-pixel.v, pixel.n);
	pixel.nov = max(dot(pixel.n, pixel.v), 0.001);

#ifdef CLEARCOAT
	pixel.clearcoat_n = in.clearcoat_normal;
	pixel.clearcoat_r = reflect(-pixel.v, pixel.clearcoat_n);
	pixel.clearcoat_nov = max(dot(pixel.clearcoat_n, pixel.v), 0.001);
#endif

	pixel.diffuse_color = compute_diffuse_color(in.base_color.rgb, in.metallic);

	let min_roughness = 0.089 * 0.089;
	// roughness
	pixel.roughness = clamp(in.roughness, min_roughness, 0.99);
	pixel.perceptual_roughness = linear_to_perceptual_roughness(pixel.roughness);

	// clearcoat
#ifdef CLEARCOAT
	pixel.clearcoat = in.clearcoat;
	pixel.clearcoat_roughness = clamp(in.clearcoat_roughness, min_roughness, 0.99);
	pixel.clearcoat_perceptual_roughness = linear_to_perceptual_roughness(pixel.clearcoat_roughness);
#endif

	pixel.f0 = compute_f0(in.base_color.rgb, in.metallic, in.reflectance);
	pixel.f90 = compute_f90(pixel.f0);

	pixel.dfg = sample_brdf(pixel.perceptual_roughness, pixel.nov);

#ifdef THICKNESS
	pixel.thickness = in.thickness;
#endif

#ifdef TRANSMISSION
	pixel.thickness = 1.0;
	pixel.transmission = in.transmission;
	pixel.absorption = in.absorption;

	let air_ior = 1.0;
	let ior = max(1.0, in.ior);

	pixel.eta_ir = air_ior / ior;
	pixel.eta_ri = ior / air_ior;
#endif

#ifdef SUBSURFACE
	pixel.subsurface_power = in.subsurface_power;
	pixel.subsurface_color = in.subsurface_color;
#endif

	return pixel;
}
