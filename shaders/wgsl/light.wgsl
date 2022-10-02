struct PointLight {
	position: vec3<f32>,
	color: vec3<f32>,
	range: f32,
	radius: f32,
}

struct DirectionalLight {
	direction: vec3<f32>,
	color: vec3<f32>,
}

@group(0) @binding(3)
var<uniform> point_light_count: u32;
@group(0) @binding(4)
var<storage, read> point_lights: array<PointLight>;

@group(0) @binding(5)
var<uniform> directional_light_count: u32;
@group(0) @binding(6)
var<storage, read> directional_lights: array<DirectionalLight>;
