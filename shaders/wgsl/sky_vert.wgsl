#include <lumi/sky.wgsl>

@vertex
fn vertex(@builtin(vertex_index) index: u32) -> Sky {
	return fullscreen_triangle(index);
}
