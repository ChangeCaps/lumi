struct Vertex {
	@location(0)
	position: vec3<f32>,
	@location(1)
	normal: vec3<f32>,
	@location(2)
	tangent: vec4<f32>,
	@location(3)
	uv_0: vec2<f32>,
}

struct MeshOut {
	@builtin(position)
	v_position: vec4<f32>,	
	@location(0)
	w_position: vec3<f32>,
	@location(1)
	w_normal: vec3<f32>,
	@location(2)
	w_tangent: vec3<f32>,
	@location(3)
	w_bitangent: vec3<f32>,
	@location(4)
	uv_0: vec2<f32>,
}

struct Mesh {
	@builtin(front_facing)
	v_front_facing: bool,
	@location(0)
	w_position: vec3<f32>,
	@location(1)
	w_normal: vec3<f32>,
	@location(2)
	w_tangent: vec3<f32>,
	@location(3)
	w_bitangent: vec3<f32>,
	@location(4)
	uv_0: vec2<f32>,
}

@group(0) @binding(1)
var<uniform> transform: mat4x4<f32>;
