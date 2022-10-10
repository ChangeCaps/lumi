struct Camera {
	position: vec3<f32>,
	aspect_ratio: f32,
	view: mat4x4<f32>,
	inverse_view: mat4x4<f32>,
	view_proj: mat4x4<f32>,
	inverse_view_proj: mat4x4<f32>,
	ev100: f32,
	exposure: f32,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

fn world_to_clip(world: vec3<f32>) -> vec4<f32> {
	return camera.view_proj * vec4<f32>(world, 1.0);
}

fn clip_to_world(clip: vec4<f32>) -> vec3<f32> {
	let world = camera.inverse_view_proj * clip;
	return world.xyz / world.w;
}
