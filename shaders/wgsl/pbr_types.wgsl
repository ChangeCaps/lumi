#include <lumi/camera.wgsl>
#include <lumi/mesh.wgsl>

struct Pbr {
	frag_coord: vec4<f32>,
	w_position: vec3<f32>,
	normal: vec3<f32>,
	view: vec3<f32>,
	base_color: vec4<f32>,
	alpha_cutoff: f32,
	metallic: f32,
	roughness: f32,
	clearcoat: f32,
	clearcoat_roughness: f32,
	clearcoat_normal: vec3<f32>,
	reflectance: f32,
	emissive: vec3<f32>,
}

fn default_pbr(mesh: Mesh) -> Pbr {
	var pbr: Pbr;

	pbr.frag_coord = mesh.v_frag_coord;
	pbr.w_position = mesh.w_position;	
	pbr.normal = mesh.w_normal;
	pbr.view = normalize(camera.position - mesh.w_position);
	pbr.base_color = vec4<f32>(1.0);
	pbr.alpha_cutoff = 0.01;
	pbr.metallic = 0.01;
	pbr.roughness = 0.089;
	pbr.clearcoat = 0.0;
	pbr.clearcoat_roughness = 0.089;
	pbr.clearcoat_normal = mesh.w_normal;
	pbr.reflectance = 0.5;
	pbr.emissive = vec3<f32>(0.0);

	return pbr;
}

struct PbrGeometry {
	position: vec3<f32>,
	n: vec3<f32>,
	v: vec3<f32>,
	r: vec3<f32>,
	nov: f32,
	clearcoat_n: vec3<f32>,
	clearcoat_r: vec3<f32>,
	clearcoat_nov: f32,
}

fn compute_geometry(pbr: Pbr) -> PbrGeometry {
	var geometry: PbrGeometry;

	geometry.position = pbr.w_position;
	geometry.n = pbr.normal;
	geometry.v = pbr.view;
	geometry.r = reflect(-geometry.v, geometry.n);
	geometry.nov = max(dot(geometry.n, geometry.v), 0.0001);

	if pbr.clearcoat > 0.0 {
		geometry.clearcoat_n = pbr.clearcoat_normal;
		geometry.clearcoat_r = reflect(-geometry.v, geometry.clearcoat_n);
		geometry.clearcoat_nov = max(dot(geometry.clearcoat_n, geometry.v), 0.0001);
	}	

	return geometry;
}
