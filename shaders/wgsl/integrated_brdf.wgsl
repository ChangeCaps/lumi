@group(0) @binding(7)
var integrated_brdf: texture_2d<f32>;

@group(0) @binding(8)
var integrated_brdf_sampler: sampler;

fn sample_brdf(perceptual_roughness: f32, nov: f32) -> vec3<f32> {
	let uv = vec2(nov, perceptual_roughness);
	return textureSample(integrated_brdf, integrated_brdf_sampler, uv).rgb;
}
