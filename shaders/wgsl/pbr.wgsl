#include <lumi/light.wgsl>
#include <lumi/pbr_light.wgsl>
#include <lumi/environment.wgsl>
#include <lumi/pbr_types.wgsl>
#include <lumi/camera.wgsl>

fn pbr_light(pbr: Pbr) -> vec4<f32> {
	if pbr.base_color.a <= pbr.alpha_cutoff {
		discard;
	}

	let pixel = get_pbr_pixel(pbr);

	var color = pbr_lights(pixel);

	let emissive_luminance = pow(2.0, camera.ev100 + pbr.emissive_exposure_compensation - 3.0);
	color += pbr.emissive * pbr.emissive_factor * emissive_luminance * pbr.base_color.a;

	color *= camera.exposure;
	color += environment(pixel);

	return vec4<f32>(color, pbr.base_color.a);
}
