#include <lumi/fullscreen.wgsl>
#include <lumi/camera.wgsl>
#include <lumi/environment.wgsl>

@fragment
fn fragment(fs: Fullscreen) -> @location(0) vec4<f32> {
	let p = fs.uv * 2.0 - 1.0;
	let direction = vec3<f32>(p * vec2<f32>(camera.aspect_ratio, -1.0), 1.0);
	let w_direction = camera.view * vec4<f32>(normalize(direction), 0.0);
	let direction = w_direction.xyz;

	return textureSample(environment_diffuse, environment_sampler, direction);
}
