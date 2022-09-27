#include <lumi/mesh.wgsl>
#include <lumi/camera.wgsl>

@vertex
fn vertex(vertex: Vertex) -> MeshOut {
	var mesh: MeshOut;

	let p = transform * vec4<f32>(vertex.position, 1.0);
	mesh.v_position = camera.view_proj * p;
	mesh.w_position = p.xyz;

	let n = transform * vec4<f32>(vertex.normal, 0.0);
	mesh.w_normal = normalize(n.xyz);

	let t = transform * vec4<f32>(vertex.tangent.xyz, 0.0);
	var b = cross(n.xyz, t.xyz) * vertex.tangent.w;
	mesh.w_tangent = normalize(t.xyz);
	mesh.w_bitangent = normalize(b);

	mesh.uv_0 = vertex.uv_0;

	return mesh;
}
