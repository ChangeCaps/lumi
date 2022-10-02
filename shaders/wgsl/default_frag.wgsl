#include <lumi/mesh.wgsl>
#include <lumi/pbr.wgsl>

@fragment
fn fragment(mesh: Mesh) -> @location(0) vec4<f32> {
	return pbr_light(new_pbr(mesh));
}
