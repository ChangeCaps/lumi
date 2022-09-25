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
