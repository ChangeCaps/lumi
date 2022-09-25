#include <lumi/fullscreen.wgsl>
#include <lumi/tonemapping.wgsl>

@group(0) @binding(0)
var hdr_texture: texture_2d<f32>;

@group(0) @binding(1)
var hdr_sampler: sampler;

@fragment
fn fragment(fs: Fullscreen) -> @location(0) vec4<f32> {
	var hdr_color = textureSample(hdr_texture, hdr_sampler, fs.uv);

	hdr_color = vec4<f32>(tonemap_aces(hdr_color.rgb), hdr_color.a);

	return hdr_color;
}

