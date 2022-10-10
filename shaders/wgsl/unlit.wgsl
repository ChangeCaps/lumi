#include <lumi/camera.wgsl>

@group(0) @binding(0)
var<uniform> transform: mat4x4<f32>;

struct Vertex {
	@builtin(position) 
	clip_position: vec4<f32>,
	@location(0)
	position: vec3<f32>,
	@location(1)
	normal: vec3<f32>,
}

@vertex
fn vertex(
	@location(0) position: vec3<f32>,
	@location(1) normal: vec3<f32>,
) -> Vertex {
	var vertex: Vertex;

	vertex.position = (transform * vec4<f32>(position, 1.0)).xyz;
	vertex.clip_position = world_to_clip(vertex.position);
	vertex.normal =(transform * vec4<f32>(normal, 0.0)).xyz;

	return vertex;
}

@group(0) @binding(0)
var<uniform> color: vec3<f32>;

@fragment
fn fragment(
	@location(0) position: vec3<f32>,
	@location(1) normal: vec3<f32>,
) -> @location(0) vec4<f32> {
	let n = normalize(normal);
	let v = normalize(camera.position - position);

	let nov = max(dot(n, v), 0.0) * 0.5 + 0.5;
	let diffuse = nov * color;

	return vec4<f32>(diffuse, 1.0);
}
