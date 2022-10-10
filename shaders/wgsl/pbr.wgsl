#include <lumi/light.wgsl>
#include <lumi/pbr_light.wgsl>
#include <lumi/environment.wgsl>
#include <lumi/pbr_types.wgsl>
#include <lumi/pbr_pixel.wgsl>
#include <lumi/camera.wgsl>

fn pbr_light(pbr: Pbr) -> vec4<f32> {
	if pbr.base_color.a <= pbr.alpha_cutoff {
		discard;
	}

	let geometry = compute_geometry(pbr);
	let pixel = get_pbr_pixel(pbr, geometry);

	var color = environment(pixel, geometry);
	color += pbr_lights(pixel, geometry);

	let emissive_luminance = pow(2.0, camera.ev100 + pbr.emissive_exposure_compensation - 3.0);
	color += pbr.emissive * emissive_luminance * camera.exposure * pbr.base_color.a;

	return vec4<f32>(color, pbr.base_color.a);
}
