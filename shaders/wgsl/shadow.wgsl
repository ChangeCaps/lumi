#include <lumi/poisson.wgsl>
#include <lumi/light.wgsl>

@group(0) @binding(0)
var directional_shadow_maps: texture_depth_2d_array;

@group(0) @binding(0)
var shadow_map_sampler: sampler;

struct Shadow {
	position: vec3<f32>,
	normal: vec3<f32>,
	frag_coord: vec4<f32>,
}

fn penumbra_radius_uv(z_receiver: f32, z_blocker: f32) -> f32 {
	return z_receiver - z_blocker;
}

fn plane_bias(position: vec3<f32>) -> vec2<f32> {
	var dx = dpdx(position);
	dx = dx - dx * 0.5;
	var dy = dpdy(position);
	dy = dy - dy * 0.5;

	var bias: vec2<f32>;
	bias.x = dy.y * dx.z - dx.y * dy.z;
	bias.y = dx.x * dy.z - dy.x * dx.z;
	let det = dx.x * dy.y - dx.y * dy.x;
	return bias / det;
}

fn shadow_noise(pos: vec3<f32>) -> f32 {
	let s = pos + 0.2127 + pos.x * pos.y * pos.z * 0.3713;
	let r = 4.789 * sin(489.123 * (s));
	return fract(r.x * r.y * r.z * (1.0 + s.x));
}

fn rotate(pos: vec2<f32>, trig: vec2<f32>) -> vec2<f32> {
	return vec2<f32>(
		pos.x * trig.x - pos.y * trig.y,
		pos.y * trig.x + pos.x * trig.y,
	);
}

fn valid_cascade(light_space: vec2<f32>) -> bool {	
	let m = max(abs(light_space.x), abs(light_space.y));
	return m < 8.0;
}

fn sample_cascade(light_space: vec2<f32>, index: u32) -> f32 {
	var cascade = 0u;
	let m = max(abs(light_space.x), abs(light_space.y));

	if m > 1.0 { cascade = 1u; }
	if m > 2.0 { cascade = 2u; }
	if m > 4.0 { cascade = 3u; }

	let d = pow(2.0, f32(cascade));

	let uv = light_space / d * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5, 0.5);

	let index = i32(index + cascade);
	let depth = textureSample(directional_shadow_maps, shadow_map_sampler, uv, index);

	return depth;
}

fn directional_find_blocker(
	light_space: vec2<f32>,
	index: u32,
	z0: f32,
	bias: f32,
	plane_bias: vec2<f32>,
	radius: f32,
	trig: vec2<f32>,
	sample_count: u32,
) -> vec2<f32> {
	var blocker_sum = 0.0;
	var blocker_count = 0.0;

	let biased_depth = z0 - bias;

	for (var i = 0u; i < sample_count; i += 1u) {
		var offset = poisson_disk[i] * radius;
		offset = rotate(offset, trig);

		let offset_light_space = light_space + offset;

		if !valid_cascade(offset_light_space) {
			continue;
		}

		let depth = sample_cascade(offset_light_space, index);
		let bias = dot(offset, plane_bias);

		if biased_depth + bias > depth {
			blocker_sum += depth;
			blocker_count += 1.0;
		}
	}

	let avg = blocker_sum / blocker_count;
	
	return vec2<f32>(avg, blocker_count);
}

fn directional_pcf_filter(
	light_space: vec2<f32>,
	index: u32,
	z0: f32,
	bias: f32,
	plane_bias: vec2<f32>,
	filter_radius: vec2<f32>,
	trig: vec2<f32>,
	sample_count: u32,
) -> f32 {
	var sum = 0.0;

	let biased_depth = z0 - bias;

	for (var i = 0u; i < sample_count; i += 1u) {
		var offset = poisson_disk[i] * filter_radius;
		offset = rotate(offset, trig);

		let offset_light_space = light_space + offset;

		if !valid_cascade(offset_light_space) {
			continue;
		}

		let depth = sample_cascade(offset_light_space, index);
		let bias = dot(offset, plane_bias);

		if biased_depth + bias < depth {
			sum += 1.0;
		}
	}

	return sum / f32(sample_count);
}

fn directional_pcss(
	light: DirectionalLight,
	light_space: vec2<f32>,
	index: u32,
	z: f32,
	bias: f32,
	plane_bias: vec2<f32>,
	z_vs: f32,
	trig: vec2<f32>,
) -> f32 {
	let search_radius = light.softness / light.size * 10.0;
	let blocker = directional_find_blocker(
		light_space,
		index,
		z,
		bias,
		plane_bias,
		search_radius,
		trig,
		16u,
	);

	if blocker.y < 1.0 {
		return 1.0;
	}

	let avg_z = blocker.x * light.depth;

	var penumbra = penumbra_radius_uv(z_vs, avg_z) * 0.005 * light.softness * 16.0;
	penumbra = 1.0 - pow(1.0 - penumbra, light.falloff);
	
	var filter_radius = vec2<f32>(penumbra - 0.015 * light.softness) / light.size;
	filter_radius = min(vec2<f32>(search_radius), filter_radius);

	return directional_pcf_filter(
		light_space,
		index,
		z,
		bias,
		plane_bias,
		filter_radius,
		trig,
		48u,
	);
}

fn directional_shadow(light: DirectionalLight, shadow: Shadow, view_proj: mat4x4<f32>, index: u32) -> f32 {
	let light_space = view_proj * vec4<f32>(shadow.position, 1.0);

	if light_space.w < 0.0 {
		return 1.0;
	}

	let plane_bias = plane_bias(shadow.frag_coord.xyz);
	let light_space = light_space.xyz / light_space.w;

	if light_space.z < 0.0 || light_space.z > 1.0 {
		return 1.0;
	}	

	let z = light_space.z;
	let z_vs = z * light.depth;

	let noise = shadow_noise(shadow.frag_coord.xyz);
	let angle = noise * 2.0 * 3.14159265359;
	let trig = vec2<f32>(cos(angle), sin(angle));

	let bias_scale = 0.01;
	let bias = 1.0 / light.depth * bias_scale;

	return directional_pcss(light, light_space.xy, index, z, bias, plane_bias, z_vs, trig);
}
