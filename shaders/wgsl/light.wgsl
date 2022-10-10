struct AmbientLight {
	color: vec3<f32>,
}

struct PointLight {
	position: vec3<f32>,
	color: vec3<f32>,
	intensity: f32,
	range: f32,
}

struct DirectionalLight {
	direction: vec3<f32>,
	color: vec3<f32>,
	intensity: f32,
	size: f32,
	depth: f32,
	softness: f32,
	falloff: f32,
	cascade: u32,
	view_proj: mat4x4<f32>,
}

struct Light {
	color: vec3<f32>,
	intensity: f32,
	l: vec3<f32>,
	attenuation: f32,
}

@group(0) @binding(0)
var<uniform> ambient_light: AmbientLight;

@group(0) @binding(0)
var<uniform> point_light_count: u32;
@group(0) @binding(0)
var<storage, read> point_lights: array<PointLight>;

@group(0) @binding(0)
var<uniform> directional_light_count: u32;
@group(0) @binding(0)
var<storage, read> directional_lights: array<DirectionalLight>;
