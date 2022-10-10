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

	pbr.thickness = standard_material.thickness;
	pbr.subsurface_power = standard_material.subsurface_power;
	pbr.subsurface_color = standard_material.subsurface_color;
	pbr.base_color = base_color_texture * standard_material.base_color;
	pbr.alpha_cutoff = standard_material.alpha_cutoff;
	pbr.metallic = standard_material.metallic * metallic_roughness_texture.b;
	pbr.roughness = standard_material.roughness * metallic_roughness_texture.g;
	pbr.reflectance = standard_material.reflectance;
	pbr.clearcoat = standard_material.clearcoat;
	pbr.clearcoat_roughness = standard_material.clearcoat_roughness;
	pbr.emissive = emissive_map.rgb * standard_material.emissive;	
	pbr.transmission = standard_material.transmission;
	pbr.ior = standard_material.ior;
	pbr.absorption = standard_material.absorption;

	pbr.normal = normalize(tbn * (normal_map * 2.0 - 1.0));
	pbr.clearcoat_normal = normalize(tbn * (clearcoat_normal_map * 2.0 - 1.0));

	return pbr_light(pbr);
}
