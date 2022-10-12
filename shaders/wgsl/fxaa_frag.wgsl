#include <lumi/fullscreen.wgsl>

@group(0) @binding(0)
var source: texture_2d<f32>;

@group(0) @binding(1)
var source_sampler: sampler;

fn fxaa_luma(rgb: vec3<f32>) -> f32 {
	return rgb.g * (0.587/0.299) + rgb.r;
}

fn fxaa_tex_off(tex: texture_2d<f32>, uv: vec2<f32>, off: vec2<f32>, tex_size: vec2<f32>) -> vec3<f32> {
	let x = uv.x + off.x * tex_size.x;
	let y = uv.y + off.y * tex_size.y;
	return textureSample(tex, source_sampler, vec2<f32>(x, y)).rgb;
}

fn fxaa(uv: vec2<f32>) -> vec3<f32> {
	let fxaa_edge_threshold = 1.0 / 8.0;
	let fxaa_edge_threshold_min = 1.0 / 16.0;
	let fxaa_search_steps = 16;
	let fxaa_search_threshold = 1.0 / 4.0;
	let fxaa_subpix_cap = 3.0 / 4.0;
	let fxaa_subpix_trim = 1.0 / 4.0;

	let fxaa_subpix_trim_scale = 1.0 / (1.0 - fxaa_subpix_trim);

	let tex_size = 1.0 / vec2<f32>(textureDimensions(source));

	let rgb_n = fxaa_tex_off(source, uv, vec2<f32>( 0.0, -1.0), tex_size).rgb;
	let rgb_w = fxaa_tex_off(source, uv, vec2<f32>(-1.0,  0.0), tex_size).rgb;
	let rgb_m = fxaa_tex_off(source, uv, vec2<f32>( 0.0,  0.0), tex_size).rgb;
	let rgb_e = fxaa_tex_off(source, uv, vec2<f32>( 1.0,  0.0), tex_size).rgb;
	let rgb_s = fxaa_tex_off(source, uv, vec2<f32>( 0.0,  1.0), tex_size).rgb;

	var luma_n = fxaa_luma(rgb_n);
	let luma_w = fxaa_luma(rgb_w);
	let luma_m = fxaa_luma(rgb_m);
	let luma_e = fxaa_luma(rgb_e);
	var luma_s = fxaa_luma(rgb_s);

	let range_min = min(min(min(luma_n, luma_w), luma_e), luma_s);
	let range_max = max(max(max(luma_n, luma_w), luma_e), luma_s);

	let range = range_max - range_min;
	let threshold =  range < max(fxaa_edge_threshold_min, range_max * fxaa_edge_threshold);

	var rgb_l = rgb_n + rgb_w + rgb_m + rgb_e + rgb_s;

	let luma_l = (luma_n + luma_w + luma_e + luma_s) * 0.25;
	let range_l = abs(luma_l - luma_m);
	var blend_l = max(0.0, (range_l / range) - fxaa_subpix_trim) * fxaa_subpix_trim_scale;
	blend_l = min(fxaa_subpix_cap, blend_l);

	let rgb_nw = fxaa_tex_off(source, uv, vec2<f32>(-1.0, -1.0), tex_size).rgb;
	let rgb_ne = fxaa_tex_off(source, uv, vec2<f32>( 1.0, -1.0), tex_size).rgb;
	let rgb_sw = fxaa_tex_off(source, uv, vec2<f32>(-1.0,  1.0), tex_size).rgb;
	let rgb_se = fxaa_tex_off(source, uv, vec2<f32>( 1.0,  1.0), tex_size).rgb;
	rgb_l += rgb_nw + rgb_ne + rgb_sw + rgb_se;
	rgb_l *= 1.0 / 9.0;

	let luma_nw = fxaa_luma(rgb_nw);
	let luma_ne = fxaa_luma(rgb_ne);
	let luma_sw = fxaa_luma(rgb_sw);
	let luma_se = fxaa_luma(rgb_se);

	let edge_vert =
		abs((0.25 * luma_nw) + (-0.5 * luma_n) + (0.25 * luma_ne)) +
		abs((0.50 * luma_w ) + (-1.0 * luma_m) + (0.50 * luma_e )) +
		abs((0.25 * luma_sw) + (-0.5 * luma_s) + (0.25 * luma_se));
	let edge_horz =
		abs((0.25 * luma_nw) + (-0.5 * luma_w) + (0.25 * luma_sw)) +
		abs((0.50 * luma_n ) + (-1.0 * luma_m) + (0.50 * luma_s )) +
		abs((0.25 * luma_ne) + (-0.5 * luma_e) + (0.25 * luma_se));

	let horz_span = edge_horz >= edge_vert;
	var length_sign: f32;
	if horz_span {
		length_sign = -tex_size.y;
	} else {
		length_sign = -tex_size.x;
	}

	if !horz_span {
		luma_n = luma_w;
		luma_s = luma_e;
	}

	var gradient_n = abs(luma_n - luma_m);
	let gradient_s = abs(luma_s - luma_m);
	luma_n = (luma_n + luma_m) * 0.5;
	luma_s = (luma_s + luma_m) * 0.5;

	if gradient_n < gradient_s {
		luma_n = luma_s;
		gradient_n = gradient_s;
		length_sign = -length_sign;
	}

	var pos_n: vec2<f32>;

	if horz_span {
		pos_n = uv + vec2<f32>(0.0, length_sign * 0.5);
	} else {
		pos_n = uv + vec2<f32>(length_sign * 0.5, 0.0);
	}

	gradient_n *= fxaa_search_threshold;

	var pos_p = pos_n;
	var off_np: vec2<f32>;
	if horz_span {
		off_np = vec2<f32>(tex_size.x, 0.0);
	} else {
		off_np = vec2<f32>(0.0, tex_size.y);
	}
	var luma_end_n = luma_n;
	var luma_end_p = luma_n;
	var done_n = false;
	var done_p = false;

	pos_n += off_np * vec2<f32>(-1.0, -1.0);
	pos_p += off_np * vec2<f32>( 1.0,  1.0);

	for (var i = 0; i < fxaa_search_steps; i += 1) {
		let rgb = textureSample(source, source_sampler, pos_n).rgb;
		if !done_n {
			luma_end_n = fxaa_luma(rgb);
		}

		let rgb = textureSample(source, source_sampler, pos_p).rgb;
		if !done_p {
			luma_end_p = fxaa_luma(rgb);
		}

		done_n = done_n || abs(luma_end_n - luma_n) >= gradient_n;
		done_p = done_p || abs(luma_end_p - luma_n) >= gradient_n;

		if done_n && done_p {
			continue;
		}

		if !done_n {
			pos_n -= off_np;
		}

		if !done_p {
			pos_p += off_np;
		}
	}

	var dst_n: f32;
	var dst_p: f32;

	if horz_span {
		dst_n = uv.x - pos_n.x;
		dst_p = pos_p.x - uv.x;
	} else {
		dst_n = uv.y - pos_n.y;
		dst_p = pos_p.y - uv.y;
	}

	let direction_n = dst_n < dst_p;
	if !direction_n {
		luma_end_n = luma_end_p;
	}	

	if ((luma_m - luma_n) < 0.0) == ((luma_end_n - luma_n) < 0.0) {
		length_sign = 0.0;
	}

	let span_length = dst_n + dst_p;
	if !direction_n {
		dst_n = dst_p;
	}
	let subpixel_offset = (0.5 + (dst_n / -span_length)) * length_sign;

	var offset: vec2<f32>;
	if horz_span {
		offset = vec2<f32>(0.0, subpixel_offset);
	} else {
		offset = vec2<f32>(subpixel_offset, 0.0);
	}

	let rgb_f = textureSample(source, source_sampler, uv + offset).rgb;

	if threshold { return rgb_m; }
	return mix(rgb_l, rgb_f, blend_l);
}

@fragment
fn fragment(fs: Fullscreen) -> @location(0) vec4<f32> {
	let rgb = fxaa(fs.uv);
	return vec4<f32>(rgb, 1.0);
}
