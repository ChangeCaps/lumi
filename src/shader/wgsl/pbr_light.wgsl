#include <lumi/light.wgsl>

let PI = 3.1415926535897932384626433832795;

fn saturate(x: f32) -> f32 {
	return clamp(x, 0.0, 1.0);
}

fn get_distance_attenuation(distance_squared: f32, inverse_range_squared: f32) -> f32 {
	let factor = distance_squared * inverse_range_squared;
	let smooth_factor = saturate(1.0 - factor * factor);
	let attenuation = smooth_factor * smooth_factor;
	return attenuation * 1.0 / max(distance_squared, 0.0001);
}

fn D_GGX(roughness: f32, ndoth: f32, h: vec3<f32>) -> f32 {
	let o = 1.0 - ndoth * ndoth;
	let a = ndoth * roughness;
	let k = roughness / (o + a * a);
	let d = k * k * (1.0 / PI);
	return d;
}

fn v_smith(roughness: f32, ndotv: f32, ndotl: f32) -> f32 {
	let a2 = roughness * roughness;
	let lv = ndotl * sqrt((ndotv - a2 * ndotv) * ndotv + a2);
	let ll = ndotv * sqrt((ndotl - a2 * ndotl) * ndotl + a2);
	return 0.5 / (lv + ll);
}

fn f_schlick3(f0: vec3<f32>, f90: f32, vdoth: f32) -> vec3<f32> {
	return f0 + (f90 - f0) * pow(1.0 - vdoth, 5.0);
}

fn f_schlick(f0: f32, f90: f32, vdoth: f32) -> f32 {
	return f0 + (f90 - f0) * pow(1.0 - vdoth, 5.0);
}

fn f_schlick_roughness3(f0: vec3<f32>, roughness: f32, vdoth: f32) -> vec3<f32> {
	let f90 = max(vec3<f32>(1.0 - roughness), f0);
	return f0 + (f90 - f0) * pow(1.0 - vdoth, 5.0);
}

fn fresnel(f0: vec3<f32>, ldoth: f32) -> vec3<f32> {
	let f90 = saturate(dot(f0, vec3<f32>(50.0 * 0.33)));
	return f_schlick3(f0, f90, ldoth);
}

fn specular(
	f0: vec3<f32>, 
	f90: f32,
	roughness: f32, 
	h: vec3<f32>, 
	ndotv: f32, 
	ndotl: f32,
	ndoth: f32,
	ldoth: f32,
	specular_intensity: f32
) -> vec3<f32> {
	let d = D_GGX(roughness, ndoth, h);
	let v = v_smith(roughness, ndotv, ndotl);
	let f = f_schlick3(f0, f90, ldoth);

	return d * v * f * specular_intensity;
}

fn fd_burley(roughness: f32, ndotv: f32, ndotl: f32, ldoth: f32) -> f32 {
	let f90 = 0.5 + 2.0 * roughness * ldoth * ldoth;
	let light_scatter = f_schlick(1.0, f90, ndotl);
	let view_scatter = f_schlick(1.0, f90, ndotv);
	return light_scatter * view_scatter * (1.0 / PI);
}

fn EnvBRDFApprox(f0: vec3<f32>, perceptual_roughness: f32, ndotv: f32) -> vec3<f32> {
    let c0 = vec4<f32>(-1.0, -0.0275, -0.572, 0.022);
    let c1 = vec4<f32>(1.0, 0.0425, 1.04, -0.04);
    let r = perceptual_roughness * c0 + c1;
    let a004 = min(r.x * r.x, exp2(-9.28 * ndotv)) * r.x + r.y;
    let AB = vec2<f32>(-1.04, 1.04) * a004 + r.zw;
    return f0 * AB.x + AB.y;
}

fn convert_roughness(perceptual_roughness: f32) -> f32 {
	let clamped_roughness = clamp(perceptual_roughness, 0.089, 1.0);
	return clamped_roughness * clamped_roughness;
}

fn point_light(
	w_position: vec3<f32>,
	light: PointLight,
	roughness: f32,
	ndotv: f32,
	normal: vec3<f32>,
	view: vec3<f32>,
	reflect: vec3<f32>,
	f0: vec3<f32>,
	f90: f32,
	diffuse: vec3<f32>,
) -> vec3<f32> {
	let light_to_frag = light.position - w_position;
	let distance_squared = dot(light_to_frag, light_to_frag);
	let inverse_range_squared = 1.0 / (light.range * light.range);
	let range_attenuation = get_distance_attenuation(distance_squared, inverse_range_squared);

	let a = roughness;
	let center_to_ray = dot(light_to_frag, reflect) * reflect -	light_to_frag;
	let closest_point = light_to_frag + center_to_ray * saturate(light.radius * inverseSqrt(dot(center_to_ray, center_to_ray)));
	let l_spec_length_inverse = inverseSqrt(dot(closest_point, closest_point));
	let normalization_factor = a / saturate(a + (light.radius * 0.5 * l_spec_length_inverse));
	let specular_intensity = normalization_factor * normalization_factor;

	var l = closest_point * l_spec_length_inverse;
	var h = normalize(l + view);
	var ndotl = saturate(dot(normal, l));
	var ndoth = saturate(dot(normal, h));
	var ldoth = saturate(dot(l, h));

	let specular_light = specular(f0, f90, roughness, h, ndotv, ndotl, ndoth, ldoth, specular_intensity);

	l = normalize(light_to_frag);
	h = normalize(l + view);
	ndotl = saturate(dot(normal, l));
	ndoth = saturate(dot(normal, h));
	ldoth = saturate(dot(l, h));

	let diffuse_light = fd_burley(roughness, ndotv, ndotl, ldoth) * diffuse;

	return (specular_light + diffuse_light) * light.color * range_attenuation * ndotl;
}

fn directional_light(
	light: DirectionalLight,
	roughness: f32,
	ndotv: f32,
	normal: vec3<f32>,
	view: vec3<f32>,
	reflect: vec3<f32>,
	f0: vec3<f32>,
	f90: f32,
	diffuse: vec3<f32>,
) -> vec3<f32> {
	let incident = -light.direction;

	let half = normalize(incident + view);
	let ndotl = saturate(dot(normal, incident));
	let ndoth = saturate(dot(normal, half));
	let ldoth = saturate(dot(incident, half));

	let diffuse_light = fd_burley(roughness, ndotv, ndotl, ldoth) * diffuse;
	let specular_intensity = 1.0;
	let specular_light = specular(f0, f90, roughness, half, ndotv, ndotl, ndoth, ldoth, specular_intensity);

	return (specular_light + diffuse_light) * light.color * ndotl;
}
