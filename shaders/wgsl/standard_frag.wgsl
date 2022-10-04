#include <lumi/mesh.wgsl>
#include <lumi/pbr.wgsl>
#include <lumi/standard_material.wgsl>

@fragment
fn fragment(mesh: Mesh) -> @location(0) vec4<f32> {
	var pbr = default_pbr(mesh);

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

	let normal_map = textureSample(
		normal_map,
		normal_map_sampler,
		mesh.uv_0
	).xyz;	

	let clearcoat_normal_map = textureSample(
		clearcoat_normal_map,
		clearcoat_normal_map_sampler,
		mesh.uv_0
	).xyz;

	let emissive_map = textureSample(
		emissive_map,
		emissive_map_sampler,
		mesh.uv_0
	);

	let tbn = mat3x3<f32>(
		mesh.w_tangent,
		mesh.w_bitangent,
		mesh.w_normal
	);

	pbr.base_color = base_color_texture * base_color;
	pbr.alpha_cutoff = alpha_cutoff;
	pbr.metallic = metallic * metallic_roughness_texture.r;
	pbr.roughness = roughness * metallic_roughness_texture.g;
	pbr.reflectance = reflectance;
	pbr.clearcoat = clearcoat;
	pbr.clearcoat_roughness = clearcoat_roughness;
	pbr.emissive = emissive_map.rgb * emissive * 8.0;
	pbr.normal = normalize(tbn * (normal_map * 2.0 - 1.0));
	pbr.clearcoat_normal = normalize(tbn * (clearcoat_normal_map * 2.0 - 1.0));

	return pbr_light(pbr);
}
