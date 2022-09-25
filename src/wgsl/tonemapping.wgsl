fn tonemap_aces(color: vec3<f32>) -> vec3<f32> {
	let a = color * (color + 0.0245786) - 0.000090537;
	let b = color * (0.983729 * color + 0.4329510) + 0.238081;
	return a / b;
}
