struct Vertex {
	@location(0)
	position: vec3<f32>,
}

struct MeshOut {
	@builtin(position)
	v_position: vec4<f32>,
}
