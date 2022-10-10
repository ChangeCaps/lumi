@group(0) @binding(0)
var input: texture_2d<f32>;

@group(0) @binding(1)
var output: texture_storage_2d<rgba16float, write>;

@group(0) @binding(2)
var<uniform> reinhard: u32;

@group(0) @binding(3)
var<uniform> count: u32;

@group(0) @binding(4)
var<uniform> axis: vec2<f32>;

@group(0) @binding(5)
var<uniform> kernel: array<vec4<f32>, 64>;

fn sample(uv: vec2<f32>) -> vec4<f32> {
	let base = vec2<i32>(floor(uv + 0.5));
	let f = fract(uv + 0.5);

	let a = textureLoad(input, base + vec2<i32>(0, 0), 0);
	let b = textureLoad(input, base + vec2<i32>(1, 0), 0);
	let c = textureLoad(input, base + vec2<i32>(0, 1), 0);
	let d = textureLoad(input, base + vec2<i32>(1, 1), 0);

	let a = a * (1.0 - f.x) + b * f.x;
	let b = c * (1.0 - f.x) + d * f.x;

	return a * (1.0 - f.y) + b * f.y;
}

fn tap(coord: vec2<f32>, weight: f32) -> vec4<f32> {
	return sample(coord) * weight;
}

fn tap_reinhard(coord: vec2<f32>, weight: f32, total_weight: ptr<function, vec4<f32>>) -> vec4<f32> {
	let s = sample(coord) * weight;
	let w = weight / (1.0 + s);
	*total_weight += w;
	return s * w;
}

@compute @workgroup_size(16, 16, 1)
fn gaussian_blur(@builtin(global_invocation_id) global_id: vec3<u32>) {
	let size_in = textureDimensions(input);
	let size_out = textureDimensions(output);
	let p_in = vec2<f32>(global_id.xy) / vec2<f32>(size_out) * vec2<f32>(size_in);
	let p_out = vec2<i32>(global_id.xy);

	var sum = vec4<f32>(0.0, 0.0, 0.0, 0.0);

	if reinhard == 1u {
		var total_weight = vec4<f32>(0.0);
		sum += tap_reinhard(p_in, kernel[0].x, &total_weight);
		var offset = axis;
		for (var i = 1u; i < count; i += 1u) {
			let k = kernel[i].x;
			var o = offset + axis * kernel[i].y;
			o *= vec2<f32>(size_in);
			sum += tap_reinhard(p_in + o, k, &total_weight);
			sum += tap_reinhard(p_in - o, k, &total_weight);
			offset += axis * 2.0;
		}
		sum /= total_weight;
	} else {
		sum += tap(p_in, kernel[0].x);
		var offset = axis;
		for (var i = 1u; i < count; i += 1u) {
			let k = kernel[i].x;
			var o = offset + axis * kernel[i].y;
			o *= vec2<f32>(size_in);
			sum += tap(p_in + o, k);
			sum += tap(p_in - o, k);
			offset += axis * 2.0;
		}
	}

	textureStore(output, p_out, sum);
}
