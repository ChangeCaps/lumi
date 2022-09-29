let PI = 3.1415926535897932384626433832795;

@group(0) @binding(0)
var source: texture_2d<u32>;

@group(0) @binding(1)
var cube: texture_storage_2d_array<rgba16float, write>;

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
	var direction: vec3<f32>;

	let cube_size = vec2<f32>(textureDimensions(cube));
	let uv = vec2<f32>(global_id.xy) / cube_size.xy * 2.0 - 1.0;

	switch global_id.z {
		case 0u { direction = vec3<f32>(1.0, -uv.y, uv.x); }
		case 1u { direction = vec3<f32>(-1.0, -uv.y, -uv.x); }
		case 2u { direction = vec3<f32>(uv.x, 1.0, uv.y); }
		case 3u { direction = vec3<f32>(uv.x, -1.0, uv.y); }
		case 4u { direction = vec3<f32>(uv.x, -uv.y, 1.0); }
		case 5u { direction = vec3<f32>(-uv.x, -uv.y, -1.0); }
		default { return; }
	}	

	let source_dimensions = vec2<f32>(textureDimensions(source));
	let angles = vec2<f32>(atan2(direction.z, direction.x), acos(direction.y));
	var source_index = vec2<i32>(
		i32(angles.x * source_dimensions.x / (2.0 * PI)),
		i32(angles.y * source_dimensions.y / PI)
	);
	source_index %= textureDimensions(source);

	let color = vec4<f32>(textureLoad(source, source_index, 0)) / 65535.0;
	textureStore(cube, vec2<i32>(global_id.xy), i32(global_id.z), color);
}
