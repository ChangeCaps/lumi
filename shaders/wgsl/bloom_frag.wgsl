#include <lumi/fullscreen.wgsl>

@group(0) @binding(0)
var source_texture: texture_2d<f32>;

@group(0) @binding(1)
var source_sampler: sampler;

@group(0) @binding(2)
var source_mip_texture: texture_2d<f32>;

@group(0) @binding(3)
var source_mip_sampler: sampler;

@group(0) @binding(4)
var<uniform> scale: f32;

@group(0) @binding(5)
var<uniform> threshold: f32;

@group(0) @binding(6)
var<uniform> curve: vec3<f32>;

fn quadratic_threshold(color: vec4<f32>, threshold: f32, curve: vec3<f32>) -> vec4<f32> {
	let br = max(max(color.r, color.g), color.b);

	var rq = clamp(br - curve.x, 0.0, curve.y);
	rq = curve.z * rq * rq;

	return color * max(rq, br - threshold) / max(br, 0.0001); 
}

@fragment
fn downsample(fs: Fullscreen) -> @location(0) vec4<f32> {
	if scale == 0.0 {
		return textureSample(source_texture, source_sampler, fs.uv);
	}

	let texel_size = 1.0 / vec2<f32>(textureDimensions(source_texture));
	let scale = texel_size * scale;
	
	let a = textureSample(source_texture, source_sampler, fs.uv + vec2<f32>(-1.0, -1.0) * scale);
	let b = textureSample(source_texture, source_sampler, fs.uv + vec2<f32>( 0.0, -1.0) * scale);
	let c = textureSample(source_texture, source_sampler, fs.uv + vec2<f32>( 1.0, -1.0) * scale);
	let d = textureSample(source_texture, source_sampler, fs.uv + vec2<f32>(-0.5, -0.5) * scale);
	let e = textureSample(source_texture, source_sampler, fs.uv + vec2<f32>( 0.5, -0.5) * scale);
	let f = textureSample(source_texture, source_sampler, fs.uv + vec2<f32>(-1.0,  0.0) * scale);
	let g = textureSample(source_texture, source_sampler, fs.uv + vec2<f32>( 0.0,  0.0) * scale);
	let h = textureSample(source_texture, source_sampler, fs.uv + vec2<f32>( 1.0,  0.0) * scale);
	let i = textureSample(source_texture, source_sampler, fs.uv + vec2<f32>(-0.5,  0.5) * scale);
	let j = textureSample(source_texture, source_sampler, fs.uv + vec2<f32>( 0.5,  0.5) * scale);
	let k = textureSample(source_texture, source_sampler, fs.uv + vec2<f32>(-1.0,  1.0) * scale);
	let l = textureSample(source_texture, source_sampler, fs.uv + vec2<f32>( 0.0,  1.0) * scale);
	let m = textureSample(source_texture, source_sampler, fs.uv + vec2<f32>( 1.0,  1.0) * scale);

	let div = (1.0 / 4.0) * vec2<f32>(0.5, 0.125);

	var o = (d + e + i + j) * div.x;
	o = o + (a + b + g + f) * div.y;
	o = o + (b + c + h + g) * div.y;
	o = o + (f + g + l + k) * div.y;
	o = o + (g + h + m + l) * div.y;

	if threshold > 0.0 {
		o = quadratic_threshold(o, threshold, curve);
	}

	return o;
}

fn sample_3x3_tent(tex: texture_2d<f32>, uv: vec2<f32>, scale: vec2<f32>) -> vec4<f32> {
	let d = vec4<f32>(1.0, 1.0, -1.0, 0.0);

	var s: vec4<f32> = textureSample(tex, source_sampler, uv - d.xy * scale);
	s += textureSample(tex, source_sampler, uv - d.wy * scale) * 2.0;
	s += textureSample(tex, source_sampler, uv - d.zy * scale);
                   
	s += textureSample(tex, source_sampler, uv + d.zw * scale) * 2.0;
	s += textureSample(tex, source_sampler, uv				 ) * 4.0;
	s += textureSample(tex, source_sampler, uv + d.xw * scale) * 2.0;
                   
	s += textureSample(tex, source_sampler, uv + d.zy * scale);
	s += textureSample(tex, source_sampler, uv + d.wy * scale) * 2.0;
	s += textureSample(tex, source_sampler, uv + d.xy * scale);

	return s / 16.0;
}

@fragment
fn upsample(fs: Fullscreen) -> @location(0) vec4<f32> {
	let texel_size = 1.0 / vec2<f32>(textureDimensions(source_mip_texture));
	let color = textureSample(source_texture, source_sampler, fs.uv);
	let mip_color = sample_3x3_tent(source_mip_texture, fs.uv, texel_size * scale);

	return color + mip_color;
}
