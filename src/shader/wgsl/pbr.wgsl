#include <lumi/mesh.wgsl>
#include <lumi/camera.wgsl>
#include <lumi/light.wgsl>
#include <lumi/pbr_light.wgsl>
#include <lumi/environment.wgsl>

@group(0) @binding(3)
var<storage, read> point_lights: array<PointLight>;

@group(0) @binding(4)
var<storage, read> directional_lights: array<DirectionalLight>;

struct PbrInput {
	w_position: vec3<f32>,
	w_normal: vec3<f32>,
	w_tangent: vec3<f32>,
	w_bitangent: vec3<f32>,
	normal_map: vec3<f32>,
	view: vec3<f32>,
	base_color: vec4<f32>,
	metallic: f32,
	roughness: f32,
	reflectance: f32,
	emissive: vec3<f32>,
}

fn new_pbr(mesh: Mesh) -> PbrInput {
	var input: PbrInput;

	input.w_position = mesh.w_position;
	if mesh.v_front_facing {
		input.w_normal = mesh.w_normal;
		input.w_tangent = mesh.w_tangent;
		input.w_bitangent = mesh.w_bitangent;
	} else {
		input.w_normal = -mesh.w_normal;
		input.w_tangent = -mesh.w_tangent;
		input.w_bitangent = -mesh.w_bitangent;
	}
	input.normal_map = vec3<f32>(0.0, 0.0, 1.0);
	input.view = normalize(camera.position - mesh.w_position);
	input.base_color = vec4<f32>(1.0);
	input.metallic = 0.01;
	input.roughness = 0.089;
	input.reflectance = 0.5;
	input.emissive = vec3<f32>(0.0);

	return input;
}

fn pbr_light(input: PbrInput) -> vec4<f32> {
	let base_color = input.base_color;

	let perceptual_roughness = input.roughness;
	let roughness = convert_roughness(perceptual_roughness);
	let metallic = input.metallic;
	let reflectance = input.reflectance;
	
	let tbn = mat3x3<f32>(
		input.w_tangent,
		input.w_bitangent,
		input.w_normal
	);
	let normal = normalize(tbn * input.normal_map);
	let view = input.view;

	let ndotv = max(dot(normal, view), 0.0);
	let f0 = 0.16 * reflectance * reflectance * (1.0 - metallic) + base_color.rgb * metallic;
	let reflect = reflect(-view, normal);

	let diffuse_color = base_color.rgb * (1.0 - metallic);

	var light = vec3<f32>(0.0, 0.0, 0.0);

	for (var i = 0u; i < arrayLength(&point_lights); i += 1u) {
		let l = point_lights[i];
		light += point_light(input.w_position, l, roughness, ndotv, normal, view, reflect, f0, diffuse_color);
	}

	for (var i = 0u; i < arrayLength(&directional_lights); i += 1u) {
		let l = directional_lights[i];
		light += directional_light(l, roughness, ndotv, normal, view, reflect, f0, diffuse_color);
	}
		
	let env = environment(base_color.rgb, metallic, roughness, ndotv, normal, reflect, f0);

	light += env;

	var color = base_color.rgb * light + input.emissive;

	return vec4<f32>(env, 1.0);
}
