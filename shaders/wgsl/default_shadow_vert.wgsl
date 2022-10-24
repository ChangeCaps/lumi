#include <lumi/shadow_mesh.wgsl>

@group(0) @binding(0)
var<uniform> view_proj: mat4x4<f32>;

@group(0) @binding(0)
var<uniform> transform: mat4x4<f32>;

@vertex
fn vertex(vertex: Vertex) -> MeshOut {
	var mesh: MeshOut;

	mesh.v_position = view_proj * transform * vec4<f32>(vertex.position, 1.0);

	return mesh;
}
