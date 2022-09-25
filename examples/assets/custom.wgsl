#include <lumi/mesh.wgsl>
#include <lumi/pbr.wgsl>

@fragment
fn fragment(mesh: Mesh) -> @location(0) vec4<f32> {
	var pbr = new_pbr(mesh);

	let x = dot(mesh.w_normal, vec3<f32>(1.0, 0.0, 0.0));
	let y = dot(mesh.w_normal, vec3<f32>(0.0, 1.0, 0.0));
	let z = dot(mesh.w_normal, vec3<f32>(0.0, 0.0, 1.0));
	pbr.base_color = vec4<f32>(x, y, z, 1.0);
	pbr.roughness = sin(mesh.w_position.x * 5.0) * 0.5 + 0.5;
	pbr.metallic = sin(mesh.w_position.y * 5.0) * 0.5 + 0.5;
	
	return pbr_light(pbr);
}
