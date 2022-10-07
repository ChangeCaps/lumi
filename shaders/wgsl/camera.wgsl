struct Camera {
	position: vec3<f32>,
	aspect_ratio: f32,
	view: mat4x4<f32>,
	view_proj: mat4x4<f32>,
	ev100: f32,
	exposure: f32,
}

@group(0) @binding(0)
var<uniform> camera: Camera;
