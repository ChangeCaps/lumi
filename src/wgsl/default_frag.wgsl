#include <lumi/mesh.wgsl>
#include <lumi/pbr_material.wgsl>
#include <lumi/pbr.wgsl>

@fragment
fn fragment(mesh: Mesh) -> @location(0) vec4<f32> {
	var pbr = new_pbr(mesh);

	let base_color_texture = textureSample(
		base_color_texture,
		base_color_sampler,
		mesh.uv_0
	);

	let metallic_roughness_texture = textureSample(
		metallic_roughness_texture,
		metallic_roughness_sampler,
		mesh.uv_0
	);

	pbr.normal_map = textureSample(
		normal_map,
		normal_map_sampler,
		mesh.uv_0
	).xyz * 2.0 - 1.0;	

	let emissive_map = textureSample(
		emissive_map,
		emissive_map_sampler,
		mesh.uv_0
	);

	pbr.base_color = base_color_texture * base_color;
	pbr.metallic = metallic * metallic_roughness_texture.r;
	pbr.roughness = roughness * metallic_roughness_texture.g;
	pbr.reflectance = reflectance;
	pbr.emissive = emissive_map.rgb * emissive;

	return pbr_light(pbr);
}
