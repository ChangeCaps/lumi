#include <lumi/shadow_mesh.wgsl>

struct Transform {
	@location(1)
	transform_0: vec4<f32>,
	@location(2)
	transform_1: vec4<f32>,
	@location(3)
	transform_2: vec4<f32>,
	@location(4)
	transform_3: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> view_proj: mat4x4<f32>;

@vertex
fn vertex(vertex: Vertex, transform: Transform) -> MeshOut {
	var mesh: MeshOut;

	let transform = mat4x4<f32>(
		transform.transform_0,
		transform.transform_1,
		transform.transform_2,
		transform.transform_3,
	);

	mesh.v_position = view_proj * transform * vec4<f32>(vertex.position, 1.0);

	return mesh;
}
