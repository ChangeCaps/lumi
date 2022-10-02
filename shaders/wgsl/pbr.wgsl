#include <lumi/light.wgsl>
#include <lumi/pbr_light.wgsl>
#include <lumi/environment.wgsl>
#include <lumi/pbr_material.wgsl>
#include <lumi/pbr_pixel.wgsl>

fn pbr_light(pbr: Pbr) -> vec4<f32> {
	if pbr.base_color.a <= pbr.alpha_cutoff {
		discard;
	}

	let geometry = compute_geometry(pbr);
	let pixel = get_pbr_pixel(pbr, geometry);

	var color = environment(pixel, geometry);

	return vec4<f32>(color, pbr.base_color.a);
}
