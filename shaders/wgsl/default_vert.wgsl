#include <lumi/mesh.wgsl>
#include <lumi/camera.wgsl>

@vertex
fn vertex(vertex: Vertex) -> MeshOut {
	var mesh: MeshOut;

	let p = transform * vec4<f32>(vertex.position, 1.0);
	mesh.v_position = camera.view_proj * p;
	mesh.w_position = p.xyz;

	let w_normal = transform * vec4<f32>(vertex.normal.xyz, 0.0);
	let w_tangent = transform * vec4<f32>(vertex.tangent.xyz, 0.0);
	let w_bitangent = cross(w_normal, w_tangent);

	mesh.w_normal = normalize(w_normal.xyz);
	mesh.w_tangent = normalize(w_tangent.xyz);
	mesh.w_bitangent = normalize(w_bitangent.xyz) * vertex.tangent.w;

	mesh.uv_0 = vertex.uv_0;

	return mesh;
}
