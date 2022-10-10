#include <lumi/camera.wgsl>

struct Sky {
	@builtin(position)
	v_position: vec4<f32>,
	@location(0)
	uv: vec2<f32>,
	@location(1)
	normal: vec3<f32>,
}

fn fullscreen_triangle(index: u32) -> Sky {
	var sky: Sky;

	let x = f32((index << 1u) & 2u);
	let y = f32(index & 2u);

	sky.uv = vec2<f32>(x, 1.0 - y);
	sky.v_position = vec4<f32>(vec2<f32>(x, y) * 2.0 - 1.0, 0.0, 1.0);	

	return sky;
}
