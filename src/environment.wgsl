let PI = 3.1415926535897932384626433832795;

@group(0) @binding(0)
var eq: texture_2d<u32>;

@group(0) @binding(1)
var cube: texture_storage_2d_array<rgba16float, write>;

@group(0) @binding(2)
var<uniform> roughness: f32;

fn direction(id: vec3<u32>, dimensions: vec2<i32>) -> vec3<f32> {
	var direction: vec3<f32>;

	let uv = vec2<f32>(id.xy) / vec2<f32>(dimensions) * 2.0 - 1.0;

	switch id.z {
		case 0u { direction = vec3<f32>(1.0, -uv.y, -uv.x); }
		case 1u { direction = vec3<f32>(-1.0, -uv.y, uv.x); }
		case 2u { direction = vec3<f32>(uv.x, 1.0, uv.y); }
		case 3u { direction = vec3<f32>(uv.x, -1.0, -uv.y); }
		case 4u { direction = vec3<f32>(uv.x, -uv.y, 1.0); }
		case 5u { direction = vec3<f32>(-uv.x, -uv.y, -1.0); }
		default { return vec3<f32>(0.0); }
	}	

	return normalize(direction);
}

fn load_eq(texture: texture_2d<u32>, angles: vec2<f32>) -> vec4<f32> {
	let dimensions = textureDimensions(texture);
	var index = vec2<i32>(
		i32(angles.x * f32(dimensions.x) / (2.0 * PI)),
		i32(angles.y * f32(dimensions.y) / PI)
	);
	index %= dimensions;
	return vec4<f32>(textureLoad(texture, index, 0)) / 65535.0 * 4.0;	
}

fn load_eq_dir(texture: texture_2d<u32>, direction: vec3<f32>) -> vec4<f32> {
	let dimensions = textureDimensions(texture);
	let angles = vec2<f32>(atan2(direction.z, direction.x), acos(direction.y));
	var index = vec2<i32>(
		i32(angles.x * f32(dimensions.x) / (2.0 * PI)),
		i32(angles.y * f32(dimensions.y) / PI)
	);
	index %= dimensions;
	return vec4<f32>(textureLoad(texture, index, 0)) / 65535.0 * 4.0;
}

@compute @workgroup_size(16, 16, 1)
fn eq_to_cube(@builtin(global_invocation_id) global_id: vec3<u32>) {
	let direction = direction(global_id, textureDimensions(cube));
	let color = load_eq_dir(eq, direction);
	textureStore(cube, vec2<i32>(global_id.xy), i32(global_id.z), color);
}

fn radical_inverse_vdc(bits: u32) -> f32 {
	var bits = bits;
	bits = (bits << 16u) | (bits >> 16u);
	bits = ((bits & 0x00ff00ffu) << 8u) | ((bits & 0xff00ff00u) >> 8u);
	bits = ((bits & 0x0f0f0f0fu) << 4u) | ((bits & 0xf0f0f0f0u) >> 4u);
	bits = ((bits & 0x33333333u) << 2u) | ((bits & 0xccccccccu) >> 2u);
	bits = ((bits & 0x55555555u) << 1u) | ((bits & 0xaaaaaaaau) >> 1u);
	return f32(bits) * 2.3283064365386963e-10; // / 0x100000000
}

fn hammersley(index: u32, num_samples: u32) -> vec2<f32> {
	return vec2<f32>(f32(index) / f32(num_samples), radical_inverse_vdc(index));
}

fn importance_sample_ggx(xi: vec2<f32>, n: vec3<f32>, roughness: f32) -> vec3<f32> {
	let a = roughness * roughness;

	let phi = 2.0 * PI * xi.x;
	let cos_theta = sqrt((1.0 - xi.y) / (1.0 + (a * a - 1.0) * xi.y));
	let sin_theta = sqrt(1.0 - cos_theta * cos_theta);

	// from spherical coordinates to cartesian coordinates
	let h = vec3<f32>(
		cos(phi) * sin_theta,
		sin(phi) * sin_theta,
		cos_theta
	);

	// from tangent-space vector to world-space sample vector
	var up: vec3<f32>;
	if abs(n.z) < 0.999 {
		up = vec3<f32>(0.0, 0.0, 1.0);
	} else { 
		up = vec3<f32>(1.0, 0.0, 0.0);
	}

	let tangent = normalize(cross(up, n));
	let bitangent = cross(n, tangent);

	let sample_dir = tangent * h.x + bitangent * h.y + n * h.z;
	return normalize(sample_dir);
}

@compute @workgroup_size(16, 16, 1)
fn specular(@builtin(global_invocation_id) global_id: vec3<u32>) {
	let direction = direction(global_id, textureDimensions(cube));
	
	if roughness == 0.0 {
		textureStore(cube, vec2<i32>(global_id.xy), i32(global_id.z), load_eq_dir(eq, direction));
		return;
	}

	var n = direction;
	var r = n;
	var v = r;

	let sample_count = 1024u;
	var total_weight = 0.0;
	var prefiltered_color = vec3<f32>(0.0);
	for (var i = 0u; i < sample_count; i += 1u) {
		let xi = hammersley(i, sample_count);
		let h = importance_sample_ggx(xi, n, roughness);
		let l = normalize(2.0 * dot(v, h) * h - v);

		let n_dot_l = max(dot(n, l), 0.0);
		if n_dot_l > 0.0 {
			prefiltered_color += n_dot_l * load_eq_dir(eq, l).rgb;
			total_weight += n_dot_l;
		}
	}
	prefiltered_color = prefiltered_color / total_weight;
	textureStore(cube, vec2<i32>(global_id.xy), i32(global_id.z), vec4<f32>(prefiltered_color, 1.0));
}

@compute @workgroup_size(16, 16, 1)
fn irradiance(@builtin(global_invocation_id) global_id: vec3<u32>) {
	let direction = direction(global_id, textureDimensions(cube));

	let up = vec3<f32>(0.0, 1.0, 0.0);
	let right = cross(up, direction);
	let up = cross(direction, right);

	let sample_delta = 0.1;
	var sample_count = 0.0;

	var irradiance = vec3<f32>(0.0);

	for (var phi = 0.0; phi < 2.0 * PI; phi += sample_delta) {
		for (var theta = 0.0; theta < 0.5 * PI; theta += sample_delta) {
			let sample_angle = vec3<f32>(
				sin(theta) * cos(phi),
				cos(theta),
				sin(theta) * sin(phi)
			);
			let sample_direction = sample_angle.x * right + sample_angle.y * up + sample_angle.z * direction;
				
			irradiance += load_eq_dir(eq, sample_direction).rgb * cos(theta) * sin(theta);
			sample_count += 1.0;
		}
	}

	irradiance = irradiance / sample_count * PI;
	textureStore(cube, vec2<i32>(global_id.xy), i32(global_id.z), vec4<f32>(irradiance, 1.0));
}