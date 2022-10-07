#include <lumi/light.wgsl>
#include <lumi/pbr_types.wgsl>
#include <lumi/pbr_pixel.wgsl>

let PI = 3.1415926535897932384626433832795;

fn saturate(x : f32) -> f32 {
	return clamp(x, 0.0, 1.0);
}

fn saturate_half(x: f32) -> f32 {
	return clamp(x, 0.00006103515625, 65504.0);
}

fn get_distance_attenuation(distance_squared: f32, inverse_range_squared: f32) -> f32 {
	let factor = distance_squared * inverse_range_squared;
	let smooth_factor = saturate(1.0 - factor * factor);
	let attenuation = smooth_factor * smooth_factor;
	return attenuation * 1.0 / max(distance_squared, 0.0001);
}

fn fd_lambert() -> f32 {
	return 1.0 / PI;
}

fn d_ggx(roughness: f32, noh: f32) -> f32 {
	let o = 1.0 - noh * noh;
	let a = noh * roughness;
	let k = roughness / (o + a * a);
	let d = k * k * (1.0 / PI);
	return saturate_half(d);
}

fn v_smith(roughness: f32, nov: f32, nol: f32) -> f32 {	
	let a2 = roughness * roughness;
    let lambdaV = nol * sqrt((nov - a2 * nov) * nov + a2);
    let lambdaL = nov * sqrt((nol - a2 * nol) * nol + a2);
    let v = 0.5 / (lambdaV + lambdaL);
    return saturate_half(v);
}

fn v_kelemen(loh: f32) -> f32 {
	return saturate_half(0.25 / (loh * loh));
}

fn f_schlick3(f0: vec3<f32>, f90: f32, voh: f32) -> vec3<f32> {
	return f0 + (f90 - f0) * pow(1.0 - voh, 5.0);
}

fn f_schlick(f0: f32, f90: f32, voh: f32) -> f32 {
	return f0 + (f90 - f0) * pow(1.0 - voh, 5.0);
}

fn fresnel(f0: vec3<f32>, loh: f32) -> vec3<f32> {
	let f90 = saturate(dot(f0, vec3<f32>(50.0 * 0.33)));
	return f_schlick3(f0, f90, loh);
}

fn specular_lobe(
	pixel: PbrPixel, 
	nov: f32,
	nol: f32,
	noh: f32,
	loh: f32,
) -> vec3<f32> {
	let d = d_ggx(pixel.roughness, noh);
	let v = v_smith(pixel.roughness, nov, nol);
	let f = fresnel(pixel.f0, loh);

	return d * v * f;
}

fn clearcoat_lobe(
	pixel: PbrPixel, 
	noh: f32,
	loh: f32,
	fcc: ptr<function, f32>,
) -> f32 {
	let d = d_ggx(pixel.clearcoat_roughness, noh);
	let v = v_kelemen(loh);
	let f = f_schlick(0.04, 1.0, loh) * pixel.clearcoat;
	*fcc = f;

	return d * v * f;
}

fn fd_burley(roughness: f32, ndotv: f32, ndotl: f32, ldoth: f32) -> f32 {
	let f90 = 0.5 + 2.0 * roughness * ldoth * ldoth;
	let light_scatter = f_schlick(1.0, f90, ndotl);
	let view_scatter = f_schlick(1.0, f90, ndotv);
	return light_scatter * view_scatter * (1.0 / PI);
}

fn light_surface(
	pixel: PbrPixel,
	geom: PbrGeometry,
	light: Light,
) -> vec3<f32> {
	let h = normalize(light.l + geom.v);
	let nol = saturate(dot(geom.n, light.l));
	let noh = saturate(dot(geom.n, h));
	let loh = saturate(dot(light.l, h));

	if nol < 0.0 {
		return vec3<f32>(0.0, 0.0, 0.0);
	}

	var diffuse_light = fd_burley(pixel.roughness, geom.nov, nol, loh) * pixel.diffuse_color;
	var specular_light = specular_lobe(pixel, geom.nov, nol, noh, loh);

	if has_refractions(pixel) {
		diffuse_light *= (1.0 - pixel.transmission);
	}

	var clearcoat = vec3<f32>(0.0);
	if has_clearcoat(pixel) {
		let clearcoat_nol = saturate(dot(geom.clearcoat_n, light.l));	
		let clearcoat_noh = saturate(dot(geom.clearcoat_n, h));

		var fcc: f32;
		let clearcoat_specular = clearcoat_lobe(pixel, clearcoat_noh, loh, &fcc);
		let attenuation = 1.0 - fcc;

		diffuse_light *= attenuation;
		specular_light *= attenuation;

		clearcoat = vec3<f32>(clearcoat_specular) * clearcoat_nol;
	}

	var color = (diffuse_light + specular_light) * nol;

	color += clearcoat;
	
	if has_subsurface(pixel) {
		let scatter_voh = saturate(dot(geom.v, -light.l));
		let forward_scatter = exp2(scatter_voh * pixel.subsurface_power - pixel.subsurface_power);
		let back_scatter = saturate(nol * pixel.thickness + (1.0 - pixel.thickness)) * 0.5;
		let subsurface = mix(back_scatter, 1.0, forward_scatter) * (1.0 - pixel.thickness);
		color += pixel.subsurface_color * subsurface * fd_lambert();
	}

	return color * light.color * light.intensity * light.attenuation;
}

fn point_light(	
	point_light: PointLight,
	pixel: PbrPixel,
	geom: PbrGeometry,
) -> vec3<f32> {
	let light_to_frag = point_light.position - geom.position;
	let distance_squared = dot(light_to_frag, light_to_frag);
	let inverse_range_squared = 1.0 / (point_light.range * point_light.range);
	let range_attenuation = get_distance_attenuation(distance_squared, inverse_range_squared);	

	var light: Light;
	light.color = point_light.color;
	light.intensity = point_light.intensity * camera.exposure;
	light.l = normalize(light_to_frag);
	light.attenuation = range_attenuation;
	return light_surface(pixel, geom, light);
}

fn directional_light(
	directionl_light: DirectionalLight,
	pixel: PbrPixel,
	geom: PbrGeometry,
) -> vec3<f32> {
	var light: Light;
	light.color = directionl_light.color;
	light.intensity = directionl_light.intensity * camera.exposure;
	light.l = -directionl_light.direction;
	light.attenuation = 1.0;
	return light_surface(pixel, geom, light);
}

fn pbr_lights(
	pixel: PbrPixel,
	geom: PbrGeometry,
) -> vec3<f32> {
	var color = vec3<f32>(0.0);	

	for (var i = 0u; i < point_light_count; i = i + 1u) {
		color += point_light(point_lights[i], pixel, geom);
	}

	for (var i = 0u; i < directional_light_count; i = i + 1u) {
		color += directional_light(directional_lights[i], pixel, geom);
	}

	return color;
}
