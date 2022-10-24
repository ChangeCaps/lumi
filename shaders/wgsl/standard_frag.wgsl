#include <lumi/mesh.wgsl>
#include <lumi/pbr.wgsl>
#include <lumi/standard_material.wgsl>

@fragment
fn fragment(mesh: Mesh) -> @location(0) vec4<f32> {
	var pbr = default_pbr(mesh);

	pbr.base_color = standard_material.base_color;
	pbr.alpha_cutoff = standard_material.alpha_cutoff;
	pbr.metallic = standard_material.metallic;
	pbr.roughness = standard_material.roughness;
	pbr.reflectance = standard_material.reflectance;	
	pbr.emissive = standard_material.emissive;

#ifdef CLEARCOAT
	pbr.clearcoat = standard_material.clearcoat;
	pbr.clearcoat_roughness = standard_material.clearcoat_roughness;
#endif

#ifdef THICKNESS
	pbr.thickness = standard_material.thickness;
#endif

#ifdef SUBSURFACE
	pbr.subsurface_power = standard_material.subsurface_power;
	pbr.subsurface_color = standard_material.subsurface_color;
#endif

#ifdef TRANSMISSION
	pbr.transmission = standard_material.transmission;
	pbr.ior = standard_material.ior;
	pbr.absorption = standard_material.absorption;
#endif

#ifdef BASE_COLOR_TEXTURE
	let base_color_texture = textureSample(
		base_color_texture,
		base_color_sampler,
		mesh.uv_0
	);
	pbr.base_color *= base_color_texture;
#endif

#ifdef METALLIC_ROUGHNESS_TEXTURE
	let metallic_roughness_texture = textureSample(
		metallic_roughness_texture,
		metallic_roughness_sampler,
		mesh.uv_0
	);
	pbr.metallic *= metallic_roughness_texture.b;
	pbr.roughness *= metallic_roughness_texture.g;
#endif

#ifdef EMISSIVE_MAP
	let emissive_map = textureSample(
		emissive_map,
		emissive_map_sampler,
		mesh.uv_0
	);
	pbr.emissive *= emissive_map.rgb;
#endif

#ifdef NORMAL_MAP
	let normal_map = textureSample(
		normal_map,
		normal_map_sampler,
		mesh.uv_0
	).xyz;

	let tbn = mat3x3<f32>(
		mesh.w_tangent,
		mesh.w_bitangent,
		mesh.w_normal
	);	

	pbr.normal = normalize(tbn * (normal_map * 2.0 - 1.0));
#endif

#ifndef NORMAL_MAP
	pbr.normal = mesh.w_normal;
#endif

#ifdef CLEACOAT
#ifdef CLEARCOAT_NORMAL_MAP
	let clearcoat_normal_map = textureSample(
		clearcoat_normal_map,
		clearcoat_normal_map_sampler,
		mesh.uv_0
	).xyz;

	pbr.clearcoat_normal = normalize(tbn * (clearcoat_normal_map * 2.0 - 1.0));
#endif

#ifndef CLEARCOAT_NORMAL_MAP
	pbr.clearcoat_normal = mesh.w_normal;
#endif
#endif

	return pbr_light(pbr);
}
