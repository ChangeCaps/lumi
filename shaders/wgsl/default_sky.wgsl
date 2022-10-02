#include <lumi/fullscreen.wgsl>
#include <lumi/camera.wgsl>

@fragment
fn fragment(fs: Fullscreen) -> @location(0) vec4<f32> {
	return vec4<f32>(fs.uv, 0.0, 1.0);
}
