struct Fullscreen {
	@builtin(position)
	v_position: vec4<f32>,
	@location(0)
	uv: vec2<f32>,
}

fn fullscreen_triangle(index: u32) -> Fullscreen {
	var fs: Fullscreen;

	let x = f32((index << 1u) & 2u);
	let y = f32(index & 2u);

	fs.uv = vec2<f32>(x, 1.0 - y);
	fs.v_position = vec4<f32>(vec2<f32>(x, y) * 2.0 - 1.0, 0.0, 1.0);

	return fs;
}
