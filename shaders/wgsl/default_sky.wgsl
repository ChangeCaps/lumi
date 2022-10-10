#include <lumi/sky.wgsl>
#include <lumi/camera.wgsl>

@group(0) @binding(0)
var sky_texture: texture_cube<f32>;

@group(0) @binding(0)
var sky_sampler: sampler;

@fragment
fn fragment(sky: Sky) -> @location(0) vec4<f32> {
	let uv = sky.uv * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0);

	let clip_near = vec4<f32>(uv, 0.0, 1.0);
	let clip_far = vec4<f32>(uv, 0.5, 1.0);

	let world_near = clip_to_world(clip_near);
	let world_far = clip_to_world(clip_far);
		
	let ray = normalize(world_far - world_near);

	let color = textureSample(sky_texture, sky_sampler, ray);
	return color;
}
